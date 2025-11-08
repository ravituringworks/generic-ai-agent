# API Documentation

The Agency Platform provides a comprehensive REST API with OpenAPI (Swagger) documentation.

## Accessing the API Documentation

Once the Agency daemon is running, you can access the OpenAPI specification at:

```
http://127.0.0.1:8080/api-docs/openapi.json
```

> **Note on Swagger UI**: The service currently uses Axum 0.8, while `utoipa-swagger-ui` v8 requires Axum 0.7. 
> Until `utoipa-swagger-ui` adds Axum 0.8 support, use one of the methods below to view the interactive documentation.

### Viewing Swagger UI

You can view the interactive API documentation using:

1. **Local Swagger UI (Easiest)**
   - Open `docs/swagger-ui.html` in your browser
   - Make sure the Agency daemon is running first
   - The page will automatically load the API spec from `http://127.0.0.1:8080/api-docs/openapi.json`

2. **Online Swagger Editor**
   - Go to https://editor.swagger.io/
   - Click "File" → "Import URL"
   - Enter: `http://127.0.0.1:8080/api-docs/openapi.json`

3. **Swagger UI Docker**
   ```bash
   docker run -p 8081:8080 -e SWAGGER_JSON_URL=http://host.docker.internal:8080/api-docs/openapi.json swaggerapi/swagger-ui
   ```
   Then access: http://localhost:8081

4. **NPM Swagger UI Watcher**
   ```bash
   npm install -g swagger-ui-watcher
   swagger-ui-watcher http://127.0.0.1:8080/api-docs/openapi.json
   ```

## API Endpoints

The Agency API is organized into the following sections:

### Health Check
- `GET /health` - Check service health and version

### Agent Operations
- `POST /api/v1/agent/process` - Process a message through the agent

### Workflow Management
- `POST /api/v1/workflows` - Create a new workflow
- `POST /api/v1/workflows/{id}/suspend` - Suspend a running workflow
- `POST /api/v1/workflows/resume` - Resume a suspended workflow

### Workflow Snapshots
- `GET /api/v1/workflows/snapshots` - List all workflow snapshots
- `GET /api/v1/workflows/snapshots/{id}` - Get a specific snapshot
- `DELETE /api/v1/workflows/snapshots/{id}` - Delete a snapshot

## Example Usage

### Process a Message

```bash
curl -X POST http://127.0.0.1:8080/api/v1/agent/process \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What is Rust?",
    "max_steps": 5
  }'
```

**Response:**
```json
{
  "response": "Rust is a systems programming language...",
  "steps_executed": 1,
  "completed": true
}
```

### Create a Workflow

```bash
curl -X POST http://127.0.0.1:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d '{
    "workflow_id": "workflow-123",
    "initial_message": "Process this task",
    "max_steps": 20
  }'
```

**Response:**
```json
{
  "workflow_id": "workflow-123",
  "status": "created"
}
```

### Resume a Workflow

```bash
curl -X POST http://127.0.0.1:8080/api/v1/workflows/resume \
  -H "Content-Type: application/json" \
  -d '{"snapshot_id": "550e8400-e29b-41d4-a716-446655440000"}'
```

### List Workflow Snapshots

```bash
curl http://127.0.0.1:8080/api/v1/workflows/snapshots
```

### Health Check

```bash
curl http://127.0.0.1:8080/health
```

**Response:**
```json
{
  "status": "ok",
  "version": "0.2.0"
}
```

## Schema Definitions

All request and response types include:
- **Field descriptions** - Clear explanation of each field
- **Examples** - Sample values for each field
- **Validation rules** - Required fields, types, constraints
- **Default values** - Where applicable

### Key Schemas

#### ProcessRequest
```json
{
  "message": "What is Rust?",      // Required: Message to process
  "max_steps": 10                   // Optional: Max execution steps (default: unlimited)
}
```

#### ProcessResponse
```json
{
  "response": "Rust is a systems programming language...",
  "steps_executed": 1,
  "completed": true
}
```

#### CreateWorkflowRequest
```json
{
  "workflow_id": "workflow-123",
  "initial_message": "Process this data",
  "max_steps": 20
}
```

#### ResumeWorkflowRequest
```json
{
  "snapshot_id": "550e8400-e29b-41d4-a716-446655440000"  // UUID format
}
```

#### ErrorResponse
```json
{
  "error": "Invalid request",
  "details": "Additional error information"
}
```

## HTTP Status Codes

The API uses standard HTTP status codes:

- **200 OK** - Request succeeded
- **400 Bad Request** - Invalid request parameters
- **404 Not Found** - Resource not found
- **500 Internal Server Error** - Server error
- **502 Bad Gateway** - Network or upstream service error

## Authentication

Currently, the API does not require authentication. In production deployments, consider adding:
- API keys
- JWT tokens
- OAuth 2.0
- mTLS

## Rate Limiting

No rate limiting is currently implemented. For production use, consider implementing:
- Request throttling
- IP-based limits
- User/API key quotas

## CORS

CORS is configured to be permissive for development. In production, configure appropriate CORS policies in the middleware layer.
