# API Examples Guide

This guide demonstrates how to use The Agency REST API from different programming languages.

## Quick Start

### 1. Start the Daemon

```bash
# Build and run the daemon
cargo build --bin agency-daemon
./target/debug/agency-daemon
```

The API server will start on `http://127.0.0.1:8080`

### 2. Run All Examples

The easiest way to see all examples in action:

```bash
./scripts/run-api-examples.sh
```

This will:

- Start the daemon automatically
- Run curl examples
- Run Python examples
- Run JavaScript/Node.js examples
- Stop the daemon when done

## Language-Specific Examples

### Run Individual Language Examples

```bash
# Only curl examples
./scripts/run-api-examples.sh --curl

# Only Python examples
./scripts/run-api-examples.sh --python

# Only JavaScript examples
./scripts/run-api-examples.sh --javascript

# Keep daemon running after examples
./scripts/run-api-examples.sh --keep-running

# Stop a running daemon
./scripts/run-api-examples.sh --stop
```

## Manual API Usage

### cURL

```bash
# Health Check
curl http://127.0.0.1:8080/health | jq

# Process Message
curl -X POST http://127.0.0.1:8080/api/v1/agent/process \
  -H "Content-Type: application/json" \
  -d '{"message": "What is Rust?", "max_steps": 5}' | jq

# Create Workflow
curl -X POST http://127.0.0.1:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d '{
    "workflow_id": "my-workflow-123",
    "initial_message": "Process this task",
    "max_steps": 20
  }' | jq

# List Snapshots
curl http://127.0.0.1:8080/api/v1/workflows/snapshots | jq

# Resume Workflow
curl -X POST http://127.0.0.1:8080/api/v1/workflows/resume \
  -H "Content-Type: application/json" \
  -d '{"snapshot_id": "550e8400-e29b-41d4-a716-446655440000"}' | jq

# Get OpenAPI Spec
curl http://127.0.0.1:8080/api-docs/openapi.json | jq
```

### Python

Save as `client.py`:

```python
#!/usr/bin/env python3
import requests
from typing import Dict, Any, Optional

class AgencyClient:
    """Client for The Agency REST API"""
    
    def __init__(self, base_url: str = "http://127.0.0.1:8080"):
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update({"Content-Type": "application/json"})
    
    def health(self) -> Dict[str, Any]:
        """Check API health"""
        response = self.session.get(f"{self.base_url}/health")
        response.raise_for_status()
        return response.json()
    
    def process(self, message: str, max_steps: Optional[int] = None) -> Dict[str, Any]:
        """Process a message through the agent"""
        data = {"message": message}
        if max_steps:
            data["max_steps"] = max_steps
        
        response = self.session.post(
            f"{self.base_url}/api/v1/agent/process",
            json=data
        )
        response.raise_for_status()
        return response.json()
    
    def create_workflow(self, workflow_id: str, initial_message: str, 
                       max_steps: int = 20) -> Dict[str, Any]:
        """Create a new workflow"""
        data = {
            "workflow_id": workflow_id,
            "initial_message": initial_message,
            "max_steps": max_steps
        }
        response = self.session.post(
            f"{self.base_url}/api/v1/workflows",
            json=data
        )
        response.raise_for_status()
        return response.json()
    
    def list_snapshots(self) -> list:
        """List all workflow snapshots"""
        response = self.session.get(
            f"{self.base_url}/api/v1/workflows/snapshots"
        )
        response.raise_for_status()
        return response.json()
    
    def resume_workflow(self, snapshot_id: str) -> Dict[str, Any]:
        """Resume a workflow from a snapshot"""
        data = {"snapshot_id": snapshot_id}
        response = self.session.post(
            f"{self.base_url}/api/v1/workflows/resume",
            json=data
        )
        response.raise_for_status()
        return response.json()


# Example usage
if __name__ == "__main__":
    client = AgencyClient()
    
    # Check health
    health = client.health()
    print(f"API Status: {health['status']}, Version: {health['version']}")
    
    # Process a message
    result = client.process("What is Python?", max_steps=5)
    print(f"Response: {result['response']}")
    print(f"Steps: {result['steps_executed']}, Completed: {result['completed']}")
    
    # Create a workflow
    workflow = client.create_workflow(
        "python-workflow",
        "Analyze data",
        max_steps=20
    )
    print(f"Workflow created: {workflow['workflow_id']}")
```

Run with:
```bash
pip3 install requests
python3 client.py
```

### JavaScript/Node.js

Save as `client.js`:

```javascript
#!/usr/bin/env node

class AgencyClient {
    constructor(baseUrl = 'http://127.0.0.1:8080') {
        this.baseUrl = baseUrl;
    }

    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const response = await fetch(url, {
            ...options,
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            }
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        return response.json();
    }

    async health() {
        return this.request('/health');
    }

    async process(message, maxSteps = null) {
        const body = { message };
        if (maxSteps) body.max_steps = maxSteps;
        
        return this.request('/api/v1/agent/process', {
            method: 'POST',
            body: JSON.stringify(body)
        });
    }

    async createWorkflow(workflowId, initialMessage, maxSteps = 20) {
        return this.request('/api/v1/workflows', {
            method: 'POST',
            body: JSON.stringify({
                workflow_id: workflowId,
                initial_message: initialMessage,
                max_steps: maxSteps
            })
        });
    }

    async listSnapshots() {
        return this.request('/api/v1/workflows/snapshots');
    }

    async resumeWorkflow(snapshotId) {
        return this.request('/api/v1/workflows/resume', {
            method: 'POST',
            body: JSON.stringify({ snapshot_id: snapshotId })
        });
    }
}

// Example usage
async function main() {
    const client = new AgencyClient();

    try {
        // Check health
        const health = await client.health();
        console.log(`API Status: ${health.status}, Version: ${health.version}`);

        // Process a message
        const result = await client.process('What is JavaScript?', 5);
        console.log(`Response: ${result.response}`);
        console.log(`Steps: ${result.steps_executed}, Completed: ${result.completed}`);

        // Create a workflow
        const workflow = await client.createWorkflow(
            'js-workflow',
            'Process data',
            20
        );
        console.log(`Workflow created: ${workflow.workflow_id}`);

    } catch (error) {
        console.error('Error:', error.message);
        process.exit(1);
    }
}

main();
```

Run with:
```bash
node client.js
```

### TypeScript

For TypeScript, add type definitions:

```typescript
interface HealthResponse {
    status: string;
    version: string;
}

interface ProcessRequest {
    message: string;
    max_steps?: number;
}

interface ProcessResponse {
    response: string;
    steps_executed: number;
    completed: boolean;
}

interface CreateWorkflowRequest {
    workflow_id: string;
    initial_message: string;
    max_steps: number;
}

interface CreateWorkflowResponse {
    workflow_id: string;
    status: string;
}

class AgencyClient {
    constructor(private baseUrl: string = 'http://127.0.0.1:8080') {}

    async health(): Promise<HealthResponse> {
        const response = await fetch(`${this.baseUrl}/health`);
        return response.json();
    }

    async process(message: string, maxSteps?: number): Promise<ProcessResponse> {
        const body: ProcessRequest = { message };
        if (maxSteps) body.max_steps = maxSteps;

        const response = await fetch(`${this.baseUrl}/api/v1/agent/process`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        return response.json();
    }

    // ... other methods
}
```

## API Endpoints

### Agent Operations

- `POST /api/v1/agent/process` - Process a message through the agent

### Workflow Management

- `POST /api/v1/workflows` - Create a new workflow
- `POST /api/v1/workflows/{id}/suspend` - Suspend a running workflow
- `POST /api/v1/workflows/resume` - Resume a suspended workflow

### Workflow Snapshots

- `GET /api/v1/workflows/snapshots` - List all snapshots
- `GET /api/v1/workflows/snapshots/{id}` - Get a specific snapshot
- `DELETE /api/v1/workflows/snapshots/{id}` - Delete a snapshot

### System

- `GET /health` - Health check
- `GET /api-docs/openapi.json` - OpenAPI specification

## Interactive Documentation

View the full API documentation with Swagger UI:

```bash
# Open in browser
open docs/swagger-ui.html

# Or visit
# https://editor.swagger.io/
# and import: http://127.0.0.1:8080/api-docs/openapi.json
```

## Error Handling

All endpoints return standard HTTP status codes:

- `200 OK` - Success
- `400 Bad Request` - Invalid request
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error
- `502 Bad Gateway` - Network error

Error responses include:
```json
{
  "error": "Error message",
  "details": "Additional details"
}
```

## Best Practices

1. **Connection Pooling**: Reuse HTTP connections for better performance
2. **Error Handling**: Always handle connection errors and timeouts
3. **Timeouts**: Set appropriate request timeouts
4. **Retries**: Implement retry logic for transient failures
5. **API Keys**: Use authentication when deploying to production

## Further Reading

- [API Documentation](./API_DOCUMENTATION.md) - Complete API reference
- [Deployment Guide](./DEPLOYMENT.md) - Production deployment
- [Configuration Guide](../README.md) - Agent configuration options
