# Uzima Cloud Health API Integration (scaffold)

This folder contains a scaffolded integration service for connecting Uzima to cloud healthcare providers (Google Cloud Healthcare, Azure Health Data Services, AWS HealthLake). The code here uses in-memory mock connectors and simple FHIR/DICOM transformers to allow local testing and serve as extension points for full production implementations.

- Mock connectors: `src/connectors/google.js`, `src/connectors/azure.js`, `src/connectors/aws.js`
- Transformer utilities: `src/transform/fhir.js`, `src/transform/dicom.js`
- Orchestrator: `src/orchestrator.js` — performs cross-cloud sync and simulates realtime events
- Entry point: `src/index.js`
- Unit tests: `tests/` (Jest)
- Dockerfile for containerizing the service (example)

What is included
- Mock connectors: `src/connectors/google.js`, `src/connectors/azure.js`, `src/connectors/aws.js` (google supports real mode)
- Transformer utilities: `src/transform/fhir.js`, `src/transform/dicom.js`, `src/transform/fhir_validator.js`
- Orchestrator: `src/orchestrator.js` — performs cross-cloud sync and simulates realtime events
- Entry point: `src/index.js`
- Unit tests: `tests/` (Jest)
- Dockerfile for containerizing the service (example)
- GitHub Actions workflow: `.github/workflows/cloud_health_api.yml`
- Mock connectors: `src/connectors/google.js`, `src/connectors/azure.js`, `src/connectors/aws.js`
- Transformer utilities: `src/transform/fhir.js`, `src/transform/dicom.js`
- Orchestrator: `src/orchestrator.js` — performs cross-cloud sync and simulates realtime events
- Entry point: `src/index.js`
- Unit tests: `tests/` (Jest)
- Dockerfile for containerizing the service (example)

Design notes and assumptions
- This scaffold intentionally uses mock connectors and in-memory storage so tests and local runs don't require cloud credentials.
- To switch to production cloud SDKs, replace the methods in each connector with SDK calls and use environment-based credentials.
- Real-time synchronization is represented by `simulateRealtimeEvents`. In production, use webhooks, Pub/Sub, Event Grid, or Kinesis and an authenticated listener.

How to run locally (quick)

1. cd into this folder

```bash
cd integrations/cloud_health_api
npm install
npm test
node src/index.js
```

Testing acceptance criteria: follow the "Testing checklist" below.

Testing checklist (step-by-step)
1. Unit tests: `npm test` — verifies FHIR transformer and orchestrator flows using mocks.
2. Start the mock orchestrator: `node src/index.js` — observe console logs showing cross-cloud syncs.
3. Real Google Cloud Healthcare mode (optional):

Set environment variables or configure the connector with the real mode options. Required environment variables when enabling real Google mode (or pass equivalent config to the connector):

```bash
export UZIMA_USE_GOOGLE_REAL=1
export GCP_PROJECT=your-gcp-project-id
export GCP_LOCATION=your-location
export GCP_DATASET=your-dataset-id
export GCP_FHIR_STORE=your-fhir-store-id
```

Note: The service uses Application Default Credentials for authentication. Ensure you have run `gcloud auth application-default login` or provide a service account JSON via `GOOGLE_APPLICATION_CREDENTIALS`.

In code you can enable real mode by constructing the GoogleConnector with a config object, for example in `src/index.js` use:

```js
const google = new GoogleConnector({ useReal: true, projectId: process.env.GCP_PROJECT, location: process.env.GCP_LOCATION, datasetId: process.env.GCP_DATASET, fhirStoreId: process.env.GCP_FHIR_STORE });
```

3. Validate cross-cloud data: The orchestrator logs sync events; connectors maintain in-memory stores that can be inspected by modifying the code to dump `connector.store` or by adding simple debug endpoints.
4. To integrate real clouds: add SDK clients and set environment variables (GCP credentials, Azure credentials, AWS credentials). See connector files for extension points.

Next steps to production
- Implement SDK clients in connectors and secure credential management (Vault, KMS).
- Implement idempotency and conflict resolution policies for cross-cloud syncs.
- Add message queue integration for reliable async sync and retry/backoff logic.
- Add automated E2E tests using test cloud accounts and synthetic FHIR/DICOM data.
 - Add automated E2E tests using test cloud accounts and synthetic FHIR/DICOM data.

Deployment and integration with infra

Docker (build and run locally):

```bash
# From the integration folder
docker build -t uzima-cloud-health-api:local .
docker run -e UZIMA_USE_GOOGLE_REAL=0 -it --rm uzima-cloud-health-api:local
```

Helm (example):

```bash
# Package and install the chart to your cluster (example)
cd integrations/cloud_health_api/helm
helm install uzima-cloud-health-api . --set image.repository=uzima-cloud-health-api,image.tag=latest
```

Terraform (example):

```bash
cd integrations/cloud_health_api/terraform
# Configure your kubeconfig/environment for the kubernetes provider
terraform init
terraform apply
```

Kubernetes example manifest

There's also a simple k8s deployment example under `k8s/cloud-health-api-deployment.yaml` which can be applied with:

```bash
kubectl apply -f integrations/cloud_health_api/k8s/cloud-health-api-deployment.yaml
```

Notes on production readiness
- Secure credentials with your cloud secret manager (GCP Secret Manager, AWS Secrets Manager, Azure Key Vault).
- Use a CI pipeline to build images and push to your container registry, and deploy via Helm or your GitOps workflow.
- Add horizontal pod autoscaling and readiness/liveness probes before production launch.
