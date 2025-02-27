# CS523 Anvil Admission Controller

## Requirements

- Kind
- Docker

## Setup

1. Start your Kubernetes cluster of choice. In my setup, I use `kind`.
```bash
kind create cluster
```

2. Build the image. The image should have the tag `admission_controller:v1`.
```bash
docker build -t admission_controller:v1 -f docker/Dockerfile .
```

3. Load the image into `kind`.
```bash
kind load docker-image admission_controller:v1
```

4. Run the setup script. The script creates TLS certificates and applies Kubernetes configs.
```bash
./setup.sh
```

5. You can confirm the correctness of the admission controller through `manifests/admission_ok.yaml` and `manifests/admission_reject.yaml`.
```bash
kubectl apply -f manifests/admission_ok.yaml # should succeed and add a label
kubectl apply -f manifests/admission_reject.yaml # should fail
```
