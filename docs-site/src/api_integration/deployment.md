# Agency Daemon Deployment Guide

This guide covers deploying the Agency platform as a long-running daemon/service with REST API access.

## Features

- **REST API Server**: HTTP endpoints for agent operations, workflow management, and A2A communication
- **Long-Running Workflows**: Support for suspend/resume workflows with state persistence
- **Saga Pattern**: Distributed transaction support with automatic compensation
- **System Service**: Run as a macOS launchd service or Unix daemon

## Quick Start

### 1. Build the Daemon

```bash
cargo build --release --bin agency-daemon
```

The binary will be located at `target/release/agency-daemon`.

### 2. Configuration

Create or edit `config.toml`:

```toml
[llm]
model = "qwen3-coder:480b-cloud"
base_url = "http://localhost:11434"

[memory]
database_url = "sqlite:memory.db"
embedding_model = "nomic-embed-text"

[mcp]
server_url = "http://localhost:8080"
timeout_seconds = 30
```

### 3. Run the Daemon

**Foreground mode:**
```bash
./target/release/agency-daemon --config config.toml --host 127.0.0.1 --port 8080
```

**Background daemon mode (Unix/macOS):**
```bash
./target/release/agency-daemon \
  --daemon \
  --config config.toml \
  --pid-file /var/run/agency.pid \
  --log-file /var/log/agency/agency.log
```

## API Endpoints

### Health Check
```bash
curl http://localhost:8080/health
```

### Process Message
```bash
curl -X POST http://localhost:8080/api/v1/agent/process \
  -H "Content-Type: application/json" \
  -d '{"message": "What is Rust?", "max_steps": 10}'
```

### Create Workflow
```bash
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -d '{
    "workflow_id": "my-workflow",
    "initial_message": "Process this data",
    "max_steps": 20
  }'
```

### Resume Workflow
```bash
curl -X POST http://localhost:8080/api/v1/workflows/resume \
  -H "Content-Type: application/json" \
  -d '{"snapshot_id": "550e8400-e29b-41d4-a716-446655440000"}'
```

### List Snapshots
```bash
curl http://localhost:8080/api/v1/workflows/snapshots
```

### Delete Snapshot
```bash
curl -X DELETE http://localhost:8080/api/v1/workflows/snapshots/550e8400-e29b-41d4-a716-446655440000
```

## macOS System Service (launchd)

### Installation

1. **Build and install the binary:**
```bash
cargo build --release --bin agency-daemon
sudo cp target/release/agency-daemon /usr/local/bin/
sudo chmod +x /usr/local/bin/agency-daemon
```

2. **Create directories:**
```bash
sudo mkdir -p /usr/local/etc/agency
sudo mkdir -p /usr/local/var/lib/agency
sudo mkdir -p /usr/local/var/log/agency
```

3. **Install configuration:**
```bash
sudo cp config.toml /usr/local/etc/agency/
```

4. **Install launchd plist:**
```bash
sudo cp com.theagency.daemon.plist /Library/LaunchDaemons/
sudo chown root:wheel /Library/LaunchDaemons/com.theagency.daemon.plist
sudo chmod 644 /Library/LaunchDaemons/com.theagency.daemon.plist
```

### Service Management

**Load and start:**
```bash
sudo launchctl load /Library/LaunchDaemons/com.theagency.daemon.plist
```

**Stop:**
```bash
sudo launchctl stop com.theagency.daemon
```

**Unload:**
```bash
sudo launchctl unload /Library/LaunchDaemons/com.theagency.daemon.plist
```

**Check status:**
```bash
sudo launchctl list | grep agency
```

**View logs:**
```bash
tail -f /usr/local/var/log/agency/stdout.log
tail -f /usr/local/var/log/agency/stderr.log
```

## Saga Pattern Usage

The Agency platform supports the Saga pattern for distributed transactions with automatic compensation.

### Example Saga

```rust
use the_agency::{SagaStep, SagaOrchestrator, SagaContext, WorkflowContext};

// Define saga steps
let reserve_inventory = SagaStep::new(
    "reserve_inventory",
    "Reserve Inventory",
    |ctx| {
        // Forward action: reserve items
        Ok(serde_json::json!({"order_id": "123", "items": 5}))
    },
    |ctx, result| {
        // Compensation: release reservation
        println!("Releasing inventory reservation");
        Ok(())
    },
);

let charge_payment = SagaStep::new(
    "charge_payment",
    "Charge Payment",
    |ctx| {
        // Forward action: charge customer
        Ok(serde_json::json!({"transaction_id": "txn_456"}))
    },
    |ctx, result| {
        // Compensation: refund payment
        println!("Refunding payment");
        Ok(())
    },
);

let send_confirmation = SagaStep::new(
    "send_confirmation",
    "Send Confirmation",
    |ctx| {
        // Forward action: send email
        Ok(serde_json::json!({"email_sent": true}))
    },
    |ctx, result| {
        // Compensation: send cancellation email
        println!("Sending cancellation email");
        Ok(())
    },
);

// Create orchestrator
let orchestrator = SagaOrchestrator::new()
    .add_step(reserve_inventory)
    .add_step(charge_payment)
    .add_step(send_confirmation);

// Execute saga
let workflow_ctx = WorkflowContext::new(10);
let saga_ctx = SagaContext::new("order-processing".to_string(), workflow_ctx);

match orchestrator.execute(saga_ctx).await {
    Ok(SagaResult::Completed(result)) => {
        println!("Saga completed: {:?}", result);
    }
    Ok(SagaResult::Compensated { error, compensated_steps }) => {
        println!("Saga failed and compensated {} steps", compensated_steps.len());
    }
    Ok(SagaResult::CompensationFailed { error, .. }) => {
        println!("Saga compensation failed: {}", error);
    }
    Err(e) => {
        println!("Saga error: {}", e);
    }
}
```

## Long-Running Workflows

Workflows can be suspended and resumed, making them ideal for long-running processes:

```rust
use the_agency::{WorkflowEngine, WorkflowContext, SuspendReason};

// Create workflow with snapshot storage
let engine = WorkflowEngine::new()
    .with_snapshot_storage(Box::new(sqlite_storage))
    .with_default_steps();

// Execute workflow
let context = WorkflowContext::new(100);
let result = engine.execute(context).await?;

// If workflow is suspended, it can be resumed later
if !result.completed {
    let snapshot_id = /* get from result */;
    
    // Later, resume from snapshot
    let resumed_result = engine.resume_from_snapshot(snapshot_id).await?;
}
```

## Environment Variables

- `RUST_LOG`: Set logging level (e.g., `info`, `debug`, `the_agency=debug`)
- `CONFIG_PATH`: Override default config file path

## Security Considerations

1. **API Access**: The API server binds to 127.0.0.1 by default. For production, consider:
   - Using a reverse proxy (nginx, Caddy)
   - Adding authentication middleware
   - Enabling HTTPS/TLS

2. **File Permissions**: Ensure proper permissions on:
   - Config files (may contain sensitive data)
   - Log files
   - Database files

3. **Service User**: Consider running as a dedicated user instead of root

## Troubleshooting

### Service won't start
- Check logs: `tail -f /usr/local/var/log/agency/*.log`
- Verify config file exists and is valid
- Ensure required directories exist with proper permissions

### API not responding
- Verify the service is running: `sudo launchctl list | grep agency`
- Check if port is already in use: `lsof -i :8080`
- Review error logs

### Workflow persistence issues
- Ensure database file is writable
- Check disk space
- Verify SQLite is properly configured

## Performance Tuning

1. **Concurrency**: Adjust Tokio runtime threads in code
2. **Database**: Use connection pooling for SQLite
3. **Memory**: Monitor workflow context size
4. **Snapshots**: Configure retention policies to prevent storage bloat

## Monitoring

Consider integrating with:
- Prometheus (add metrics endpoint)
- Health check monitoring
- Log aggregation (ELK, Loki)
- APM tools (DataDog, New Relic)

## Next Steps

- Add authentication to API endpoints
- Implement rate limiting
- Add metrics and monitoring
- Create systemd service file for Linux
- Build Docker container
