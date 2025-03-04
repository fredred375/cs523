use kube::core::{
    admission::{AdmissionRequest, AdmissionResponse, AdmissionReview},
    DynamicObject, ResourceExt,
};
use std::{convert::Infallible, error::Error};
use tracing::*;
use warp::{reply, Filter, Reply};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let routes = warp::path("validate")
        .and(warp::body::json())
        .and_then(validate_handler)
        .with(warp::trace::request());

    // You must generate a certificate for the service / url,
    // encode the CA in the ValidatingWebhookConfiguration, and terminate TLS here.
    // See admission_setup.sh + admission_controller.yaml.tpl for how to do this.
    // let addr = format!("{}:8443", std::env::var("ADMISSION_PRIVATE_IP").unwrap());
    warp::serve(warp::post().and(routes))
        .tls()
        .cert_path("/certs/tls.crt")
        .key_path("/certs/tls.key")
        .run(([0, 0, 0, 0], 8443)) // in-cluster
        // .run(addr.parse::<std::net::SocketAddr>().unwrap())
        .await;
}

// A general /validate handler, handling errors from the underlying business logic
async fn validate_handler(body: AdmissionReview<DynamicObject>) -> Result<impl Reply, Infallible> {
    // Parse incoming webhook AdmissionRequest first
    let req: AdmissionRequest<_> = match body.try_into() {
        Ok(req) => req,
        Err(err) => {
            error!("invalid request: {}", err.to_string());
            return Ok(reply::json(
                &AdmissionResponse::invalid(err.to_string()).into_review(),
            ));
        }
    };

    // Then construct a AdmissionResponse
    let mut res = AdmissionResponse::from(&req);
    // req.Object always exists for us, but could be None if extending to DELETE events
    if let Some(obj) = req.object {
        let name = obj.name_any(); // apiserver may not have generated a name yet
        res = match validate_state(res.clone(), &obj) {
            Ok(res) => {
                info!("accepted: {:?} on resource {}", req.operation, name);
                res
            }
            Err(err) => {
                warn!("denied: {:?} on {} ({})", req.operation, name, err);
                res.deny(err.to_string())
            }
        };
    };
    // Wrap the AdmissionResponse wrapped in an AdmissionReview
    Ok(reply::json(&res.into_review()))
}

// The main validation handler implementing the spec requirements
fn validate_state(res: AdmissionResponse, obj: &DynamicObject) -> Result<AdmissionResponse, Box<dyn Error>> {

    let spec = obj.data.get("spec").ok_or("Missing spec field")?;

    // 1. Validate replicas is non-negative
    if let Some(replicas) = spec.get("replicas") {
        if replicas.as_i64().map_or(true, |r| r < 0) {
            return Err("Spec replicas must be non-negative".into());
        }
    }

    // 2. Validate selector exists and match_labels is not empty
    let selector = spec.get("selector").ok_or("Missing spec.selector field")?;
    let match_labels = selector.get("matchLabels").ok_or("Missing spec.selector.matchLabels field")?;

    if !match_labels.is_object() || match_labels.as_object().unwrap().is_empty() {
        return Err("spec.selector.matchLabels must be non-empty".into());
    }

    // 3. Validate template, metadata, and spec exist
    let template = spec.get("template").ok_or("Missing spec.template field")?;
    let template_metadata = template.get("metadata").ok_or("Missing spec.template.metadata field")?;
    let template_spec = template.get("spec").ok_or("Missing spec.template.spec field")?;
    // if !template.get("spec").is_some() {
    //     return Err("Missing spec.template.spec field".into());
    // }
    // log the value of spec
    info!("spec: {:?}", template_spec);

    // 4. Validate selector matches template's metadata labels
    let template_labels = template_metadata.get("labels");
    if template_labels.is_none() {
        return Err("spec.template.metadata.labels is required to match selector".into());
    }

    let template_labels = template_labels.unwrap().as_object().ok_or("Template labels must be an object")?;
    let match_labels = match_labels.as_object().unwrap();

    // Check if all matchLabels exist in template labels with the same values
    for (key, value) in match_labels {
        match template_labels.get(key) {
            Some(template_value) if template_value == value => {
            },
            _ => return Err(format!("Selector matchLabel '{}' not found in template labels or value doesn't match", key).into())
        }
    }

    Ok(res)
}