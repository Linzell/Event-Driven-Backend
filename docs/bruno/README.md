# Dispensary API - Bruno Collection

API testing collection for the Dispensary Event-Driven Backend.

## Setup

1. Install [Bruno](https://www.usebruno.com/)
2. Open Bruno → Open Collection → Select `dispensary-backend/docs/bruno`
3. Select "Local" environment in top-right dropdown

## Important Note

The `dispenseId` variable is **automatically set** after running "Create Dispense".

You must run requests in order:
1. Create Dispense (sets dispenseId)
2. All other requests use {{dispenseId}}

## Request Order

Execute in sequence:
1. **Create Dispense** - Creates new dispense, saves ID to environment
2. **Get Dispense** - Retrieves the created dispense
3. **Add Patient** - Adds patient information
4. **Add Drugs** - Adds medications
5. **Complete Dispense** - Finalizes workflow
6. **Upload Prescription URL** - Gets S3 presigned URL

## Lambda Response Format

Since we invoke Lambda directly, responses are wrapped:

```json
{
  "statusCode": 201,
  "body": "{...actual response...}",
  "headers": {...}
}
```

The post-response scripts automatically parse `body` and extract data.

## Manual Testing

If Bruno isn't working, test directly with curl:

```bash
# Create dispense
curl -X POST "http://localhost:4566/2015-03-31/functions/dispensary-local-api/invocations" \
  -H "Content-Type: application/json" \
  -d '{"version":"2.0","routeKey":"POST /dispenses","rawPath":"/dispenses","requestContext":{"http":{"method":"POST","path":"/dispenses"}},"body":"{}","isBase64Encoded":false}'

# Copy the "id" from response and use in next requests
```

## Troubleshooting

### dispenseId not set
- Make sure you ran "Create Dispense" first
- Check Console tab in Bruno for script errors
- Verify response has `body.id` field

### Connection refused
- Check LocalStack is running: `docker ps | grep dispensary`
- Verify endpoint: `http://localhost:4566`

### Lambda not found
- Verify deployment: `cd infra/local && terraform output`
- Check Lambda exists: `curl http://localhost:4566/_localstack/health`
