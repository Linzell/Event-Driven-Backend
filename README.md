# Event-Driven Backend

Event-sourced management system using CQRS/ES with AWS.

## Architecture

```
User → API Lambda → DynamoDB Event Log → Publisher Lambda → Kinesis Stream → Projectors
```

## Prerequisites

```bash
# Required
cargo --version           # Rust
cargo-lambda --version    # Lambda builder
terraform --version       # Infrastructure
docker --version          # LocalStack

# Install if missing
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
brew install cargo-make docker
brew tap hashicorp/tap   
brew install hashicorp/tap/terraform
```

## Quick Start

### 1. Setup Environment

```bash
cp .env.example .env
# Add your LocalStack auth token to .env:
# LOCALSTACK_AUTH_TOKEN=your-token-here
```

### 2. Start LocalStack Pro

```bash
docker-compose up -d
```

### 3. Build Lambdas

**For local (Mac ARM64):**
```bash
cargo make lambda-build
```

**For AWS deployment (x86_64):**
```bash
# Update Makefile.toml to use --x86-64 instead of --arm64, then:
cargo make lambda-build
```

### 4. Deploy Infrastructure

```bash
cd infra/local
cargo make tf init
cargo make tf apply --auto-approve
```

### 6. Test with Bruno

Open `docs/bruno` in Bruno REST client and run the requests in order:
1. Create Dispense
2. Add Patient
3. Add Drugs
4. Complete Dispense

## Testing

### Check Infrastructure

```bash
# DynamoDB Tables
aws --endpoint-url=http://localhost:4566 dynamodb list-tables

# Kinesis Stream
aws --endpoint-url=http://localhost:4566 kinesis list-streams

# Lambda Functions
aws --endpoint-url=http://localhost:4566 lambda list-functions

# S3 Bucket
aws --endpoint-url=http://localhost:4566 s3 ls
```

### Test API (Direct Lambda Invocation)

**Note:** LocalStack Freemium doesn't support API Gateway V2. Use direct Lambda invocation:

#### 1. Create Dispense
```bash
curl -X POST "http://localhost:4566/2015-03-31/functions/dispensary-local-api/invocations" \
  -H "Content-Type: application/json" \
  -d '{
    "version": "2.0",
    "routeKey": "POST /dispenses",
    "rawPath": "/dispenses",
    "requestContext": {"http": {"method": "POST", "path": "/dispenses"}},
    "headers": {"content-type": "application/json"},
    "body": "{}",
    "isBase64Encoded": false
  }'
```

#### 2. Get Dispense (replace {id})
```bash
curl -X POST "http://localhost:4566/2015-03-31/functions/dispensary-local-api/invocations" \
  -H "Content-Type: application/json" \
  -d '{
    "version": "2.0",
    "routeKey": "GET /dispenses/{id}",
    "rawPath": "/dispenses/{id}",
    "pathParameters": {"id": "{id}"},
    "requestContext": {"http": {"method": "GET", "path": "/dispenses/{id}"}},
    "isBase64Encoded": false
  }'
```

**Recommended:** Use the Bruno collection in `docs/bruno` for easier testing.

## Dispense Workflow States

1. **pending** - Dispense created
2. **analyzing** - Prescription uploaded, AI analyzing
3. **ready** - Analysis complete, ready for patient/drugs
4. **complete** - Dispense finalized

## Events Published

- `Dispense:Started`
- `Dispense:PrescriptionUploaded`
- `Dispense:PrescriptionAnalyzed`
- `Dispense:PatientAdded`
- `Dispense:DrugsAdded`
- `Dispense:Completed`
- `Dispense:Cancelled`

## Troubleshooting

### LocalStack not starting
```bash
docker-compose down
docker-compose up -d
docker logs -f dispensary-localstack
```

### Terraform errors
```bash
cd infra/local
rm -rf .terraform terraform.tfstate*
terraform init
```

### Lambda build issues
```bash
cargo clean
cargo make lambda-build
```

## Cleanup

```bash
# Destroy infrastructure
cd infra/local
terraform destroy

# Stop LocalStack
docker-compose down

# Clean builds
cargo clean
```

## Development

```bash
# Check code
cargo check

# Format
cargo fmt

# Lint
cargo clippy

# Test
cargo test
```

## LocalStack Web Interface

Access the LocalStack web interface at https://app.localstack.cloud to:
- View API Gateway resources
- Inspect Lambda functions
- Monitor DynamoDB tables
- Check Kinesis streams
- Browse S3 buckets

Your auth token from `.env` is required for access.

## AWS Deployment

**Build and deploy:**
```bash
# Update Makefile.toml to use --x86-64 instead of --arm64
# Then build all lambdas:
cargo make lambda-build

# Deploy infrastructure
cd infra/aws
terraform init -backend-config="bucket=your-tfstate-bucket" \
               -backend-config="key=dispensary/terraform.tfstate"
terraform apply -var-file=dev.tfvars
```

**Note:** The `infra/aws` module uses `x86_64` by default (no `lambda_architecture` override needed).
