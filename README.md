# word-manager-rust-be

Rust backend for Word Manager, deployed as an AWS Lambda function behind API Gateway.

Infrastructure is managed by Terraform in
[full-stack-k8s/serverless/infra](https://github.com/i-am-wizard/full-stack-k8s/tree/main/serverless/infra).

## Prerequisites

- [Rust](https://rustup.rs/) (for local development, not needed for building)
- Container runtime (e.g. [Docker](https://docs.docker.com/get-docker/) or [Rancher Desktop](https://rancherdesktop.io/))
- [AWS CLI v2](https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2.html) configured with credentials

## Manual Deploy to AWS Lambda

The Terraform infra creates the Lambda function (`word-manager-backend`) with a placeholder zip.
These steps replace it with the real compiled binary.

### 1. Build

Build inside the cargo-lambda Docker container (no local install needed):

```bash
docker run --rm -v "$PWD":/app -w /app ghcr.io/cargo-lambda/cargo-lambda \
  cargo lambda build --release --arm64
```

Output: `target/lambda/word-manager-backend/bootstrap`

### 2. Deploy

```bash
# Zip the bootstrap binary
cd target/lambda/word-manager-backend
zip -j bootstrap.zip bootstrap
cd -

# Upload new code
aws lambda update-function-code \
  --function-name word-manager-backend \
  --zip-file fileb://target/lambda/word-manager-backend/bootstrap.zip \
  --architectures arm64 \
  --region eu-west-2

# Wait for the update to finish
aws lambda wait function-updated \
  --function-name word-manager-backend \
  --region eu-west-2

# Publish a new version
VERSION=$(aws lambda publish-version \
  --function-name word-manager-backend \
  --region eu-west-2 \
  --query 'Version' --output text)

# Point the "live" alias to the new version
aws lambda update-alias \
  --function-name word-manager-backend \
  --name live \
  --function-version "$VERSION" \
  --region eu-west-2
```

> The `live` alias is what API Gateway targets. Uploading code alone isn't enough,
> you must publish a version and update the alias.

### 3. Verify

```bash
# Health check
curl https://<API_ID>.execute-api.eu-west-2.amazonaws.com/api

# Get word
curl https://<API_ID>.execute-api.eu-west-2.amazonaws.com/api/word

# Update word
curl -X PUT https://<API_ID>.execute-api.eu-west-2.amazonaws.com/api/word \
  -H 'Content-Type: application/json' \
  -d '{"word":"Hello"}'
```

Get `<API_ID>` from Terraform: `cd serverless/infra/main && terraform output api_gateway_endpoint`

## API Endpoints

| Method | Path        | Description      |
|--------|-------------|------------------|
| GET    | `/api`      | Health check     |
| GET    | `/api/word` | Get current word |
| PUT    | `/api/word` | Update the word  |

## Local Development

See [testing-word-manager-locally.md](testing-word-manager-locally.md).