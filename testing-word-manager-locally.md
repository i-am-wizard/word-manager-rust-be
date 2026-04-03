

You need two things running locally: **DynamoDB Local** (Docker) and **`cargo lambda watch`** (emulates API Gateway + Lambda). Both run via Docker Compose.

### 1. Start everything

```bash
cd backend-rust
docker compose up -d
```

This starts:
- **DynamoDB Local** on port 8000 (with `-sharedDb` - single shared database for all clients)
- **cargo-lambda** on port 9000 (compilation is lazy - triggered by first request)

### 2. Create the table + seed data (once)

The `AWS_ACCESS_KEY_ID=local` prefix prevents the CLI from accidentally hitting real AWS:

```bash
AWS_ACCESS_KEY_ID=local AWS_SECRET_ACCESS_KEY=local \
aws dynamodb create-table \
  --endpoint-url http://localhost:8000 \
  --table-name word-manager-table \
  --attribute-definitions \
    AttributeName=PK,AttributeType=S \
    AttributeName=SK,AttributeType=S \
    AttributeName=GSI1PK,AttributeType=S \
    AttributeName=GSI1SK,AttributeType=S \
  --key-schema AttributeName=PK,KeyType=HASH AttributeName=SK,KeyType=RANGE \
  --global-secondary-indexes \
    '[{"IndexName":"GSI1","KeySchema":[{"AttributeName":"GSI1PK","KeyType":"HASH"},{"AttributeName":"GSI1SK","KeyType":"RANGE"}],"Projection":{"ProjectionType":"ALL"}}]' \
  --billing-mode PAY_PER_REQUEST \
  --region eu-west-2

AWS_ACCESS_KEY_ID=local AWS_SECRET_ACCESS_KEY=local \
aws dynamodb put-item \
  --endpoint-url http://localhost:8000 \
  --table-name word-manager-table \
  --item '{"PK":{"S":"WORD#1"},"SK":{"S":"WORD#1"},"word":{"S":"World"},"GSI1PK":{"S":"WORDS"},"GSI1SK":{"S":"WORD#1"}}' \
  --region eu-west-2
```

### 3. Test with curl

```bash
# Health check
curl http://localhost:9000/api

# Get word
curl http://localhost:9000/api/word

# Update word
curl -X PUT http://localhost:9000/api/word \
  -H 'Content-Type: application/json' \
  -d '{"word":"Hello"}'

# Verify the update
curl http://localhost:9000/api/word
```

> **Note:** The first request triggers compilation inside the container, so it will be slow. Subsequent requests are fast.