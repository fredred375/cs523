# CS523 Anvil Admission Controller

## Requirements
Before setting up, ensure you have the following tools installed. The listed versions are what worked during testing but may vary based on your environment.

- `kubectl` (v1.32.2) – Kubernetes CLI
- `kind` (v0.27.0) – Running Kubernetes in Docker
- `Docker` (27.2.1) – Container runtime

## Setup

Follow these steps to set up and test the example admission controller.



1. Start a Kubernetes Cluster

You can use any Kubernetes cluster, but this setup assumes you're using **kind**.
```bash
kind create cluster
```

2. Build the Docker Image

Build the container image for the admission controller, ensuring it is tagged as `admission_controller:v1`:
```bash
docker build -t admission_controller:v1 -f docker/Dockerfile .
```

3. Load the image into `kind`

Since kind runs Kubernetes inside Docker, the image must be explicitly loaded into the cluster:
```bash
kind load docker-image admission_controller:v1
```

4. Run the setup script

The setup script generates TLS certificates and applies the necessary Kubernetes configurations.
```bash
./setup.sh
```

5.  Verify the Admission Controller

You can test the admission controller using the provided manifests:
```bash
kubectl apply -f manifests/admission_ok.yaml # should succeed and add a label
kubectl apply -f manifests/admission_reject.yaml # should fail
```
