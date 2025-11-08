# Saga Workflows Guide

The Agency framework provides comprehensive support for **saga patterns** - a design pattern for managing distributed transactions and complex multi-step workflows with automatic compensation and rollback capabilities.

## What are Saga Patterns?

Saga patterns are used to maintain data consistency in distributed systems by coordinating multiple operations that must either all succeed or all be compensated (rolled back) if any operation fails.

### Key Concepts

- **Forward Operations**: The main business logic steps
- **Compensation Operations**: Rollback actions for each forward step
- **Automatic Rollback**: When a step fails, all previous steps are compensated in reverse order
- **State Persistence**: Saga execution state is tracked and can be resumed

## Basic Saga Workflow

### Example: E-commerce Order Processing

```rust
use the_agency::saga::{SagaOrchestrator, SagaStep, SagaContext, SagaResult};
use the_agency::WorkflowContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Reserve inventory
    let reserve_inventory = SagaStep::new(
        "reserve_inventory",
        "Reserve Inventory",
        |_ctx| {
            println!("📦 Reserving inventory...");
            // Reserve inventory logic
            Ok(serde_json::json!({
                "order_id": "ORD-12345",
                "quantity": 5
            }))
        },
        |_ctx, result| {
            println!("🔄 Releasing inventory reservation...");
            // Compensation: release reserved inventory
            Ok(())
        },
    );

    // Step 2: Process payment
    let process_payment = SagaStep::new(
        "process_payment",
        "Process Payment",
        |_ctx| {
            println!("💳 Processing payment...");
            // Payment processing logic
            Ok(serde_json::json!({
                "transaction_id": "TXN-ABC123",
                "amount": 99.99
            }))
        },
        |_ctx, result| {
            println!("🔄 Refunding payment...");
            // Compensation: refund payment
            Ok(())
        },
    );

    // Step 3: Send confirmation
    let send_confirmation = SagaStep::new(
        "send_confirmation",
        "Send Confirmation",
        |_ctx| {
            println!("📧 Sending confirmation email...");
            // Email sending logic
            Ok(serde_json::json!({
                "email_id": "EMAIL-XYZ789"
            }))
        },
        |_ctx, result| {
            println!("🔄 Sending cancellation email...");
            // Compensation: send cancellation email
            Ok(())
        },
    );

    // Create orchestrator
    let orchestrator = SagaOrchestrator::new()
        .add_step(reserve_inventory)
        .add_step(process_payment)
        .add_step(send_confirmation);

    // Execute saga
    let workflow_ctx = WorkflowContext::new(10);
    let saga_ctx = SagaContext::new("order-processing".to_string(), workflow_ctx);

    match orchestrator.execute(saga_ctx).await? {
        SagaResult::Completed(result) => {
            println!("✅ Order completed successfully!");
        }
        SagaResult::Compensated { error, compensated_steps } => {
            println!("⚠️ Order failed but was compensated: {}", error);
            println!("Compensated steps: {:?}", compensated_steps);
        }
        SagaResult::CompensationFailed { error, compensation_error, failed_at_step } => {
            println!("❌ Order failed and compensation also failed: {}", error);
        }
    }

    Ok(())
}
```

### Running the Example

```bash
cargo run --example saga_workflow
```

This demonstrates:
- ✅ **Successful execution**: All steps complete
- ⚠️ **Failure with compensation**: Payment fails, inventory reservation is released
- ⚠️ **Confirmation failure**: Email fails, payment is refunded and inventory released

## Advanced Saga: LLM Integration

The Agency framework supports **LLM-integrated sagas** where each workflow step uses AI language models, with intelligent compensation for AI-generated content.

### Example: AI Research Workflow

```rust
use the_agency::{Agent, AgentBuilder, AgentConfig};
use the_agency::saga::{SagaOrchestrator, SagaStep};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize LLM agent
    let agent = Arc::new(Mutex::new(
        AgentBuilder::new()
            .with_config(AgentConfig::default())
            .with_system_prompt("You are an expert AI research assistant.")
            .build()
            .await?
    ));

    // Step 1: Generate research plan
    let agent_clone1 = Arc::clone(&agent);
    let generate_plan = SagaStep::new(
        "generate_plan",
        "Generate Research Plan",
        move |_ctx| {
            let agent = Arc::clone(&agent_clone1);
            async move {
                println!("🧠 Generating research plan using LLM...");
                let prompt = "Create a research plan for transformer architectures in NLP";

                let mut agent_guard = agent.lock().await;
                let response = agent_guard.process(prompt).await?;

                Ok(serde_json::json!({
                    "plan_content": response,
                    "research_topic": "Transformer Architectures"
                }))
            }
        },
        |_ctx, result| {
            async move {
                println!("🔄 Archiving research plan...");
                // Compensation: save plan for later reference
                Ok(())
            }
        },
    );

    // Step 2: Conduct analysis
    let agent_clone2 = Arc::clone(&agent);
    let conduct_analysis = SagaStep::new(
        "conduct_analysis",
        "Conduct Research Analysis",
        move |_ctx| {
            let agent = Arc::clone(&agent_clone2);
            async move {
                println!("🔬 Conducting research analysis...");
                let prompt = "Analyze current transformer architecture trends";

                let mut agent_guard = agent.lock().await;
                let response = agent_guard.process(prompt).await?;

                Ok(serde_json::json!({
                    "analysis_content": response,
                    "findings": "Transformers outperform RNNs by 15%"
                }))
            }
        },
        |_ctx, result| {
            async move {
                println!("🔄 Deleting analysis results...");
                // Compensation: remove analysis data
                Ok(())
            }
        },
    );

    // Step 3: Generate report
    let agent_clone3 = Arc::clone(&agent);
    let generate_report = SagaStep::new(
        "generate_report",
        "Generate Final Report",
        move |_ctx| {
            let agent = Arc::clone(&agent_clone3);
            async move {
                println!("📄 Generating final report...");
                let prompt = "Generate comprehensive research report";

                let mut agent_guard = agent.lock().await;
                let response = agent_guard.process(prompt).await?;

                Ok(serde_json::json!({
                    "report_content": response,
                    "status": "completed"
                }))
            }
        },
        |_ctx, result| {
            async move {
                println!("🔄 Removing draft report...");
                // Compensation: delete report draft
                Ok(())
            }
        },
    );

    // Create and execute saga
    let orchestrator = SagaOrchestrator::new()
        .add_step(generate_plan)
        .add_step(conduct_analysis)
        .add_step(generate_report);

    let workflow_ctx = WorkflowContext::new(10);
    let saga_ctx = SagaContext::new("ai-research".to_string(), workflow_ctx);

    match orchestrator.execute(saga_ctx).await? {
        SagaResult::Completed(result) => {
            println!("✅ AI research workflow completed!");
        }
        SagaResult::Compensated { error, compensated_steps } => {
            println!("⚠️ Research failed but compensated: {}", error);
        }
        _ => println!("❌ Unexpected result"),
    }

    Ok(())
}
```

### Running the LLM Saga Example

```bash
# Ensure Ollama is running
ollama serve

# Pull required models
ollama pull llama3.2
ollama pull nomic-embed-text

# Run the example
cargo run --example saga_llm_workflow
```

## Saga Features

### Retry Logic
```rust
let step = SagaStep::new(id, name, action, compensation)
    .with_retries(3)  // Retry up to 3 times on failure
    .with_retry_delay(Duration::from_secs(1));
```

### Conditional Failure
```rust
let step = SagaStep::new(id, name, action, compensation)
    .non_retryable();  // Don't retry this step on failure
```

### State Persistence
```rust
// Saga state is automatically tracked
let context = SagaContext::new("workflow-name".to_string(), workflow_ctx);

// State includes:
// - Current step execution status
// - Results from completed steps
// - Retry counts per step
// - Compensation status
```

## Use Cases

### E-commerce
- Inventory reservation → Payment processing → Order fulfillment
- Compensation: Release inventory, refund payment, cancel order

### Financial Services
- Account validation → Fund transfer → Transaction recording
- Compensation: Reverse transfer, restore balances, void transaction

### AI/ML Pipelines
- Data validation → Model training → Model deployment
- Compensation: Delete training data, remove model artifacts, rollback deployment

### Content Management
- Content creation → Review process → Publication
- Compensation: Archive drafts, remove reviews, unpublish content

### Research Workflows
- Hypothesis generation → Experiment design → Data analysis → Report writing
- Compensation: Archive hypotheses, delete experimental data, remove draft reports

## Best Practices

### Step Design
- **Idempotent Operations**: Steps should be safe to retry
- **Minimal Scope**: Each step should do one thing well
- **Clear Boundaries**: Steps should have well-defined inputs/outputs

### Compensation Design
- **Reverse Operations**: Compensations should undo the forward operation
- **Data Preservation**: Consider archiving rather than deleting important data
- **Notification**: Inform stakeholders of compensation actions

### Error Handling
- **Specific Errors**: Use descriptive error messages
- **Retryable vs Non-retryable**: Distinguish between transient and permanent failures
- **Timeout Management**: Set appropriate timeouts for long-running operations

### Monitoring
- **Logging**: Log all step executions and compensations
- **Metrics**: Track success rates, compensation frequency, execution times
- **Alerts**: Monitor for frequent compensations indicating systemic issues

## Integration with Workflows

Sagas integrate seamlessly with the Agency workflow engine:

```rust
use the_agency::workflow::SagaWorkflowStep;

// Convert saga to workflow step
let saga_step = SagaWorkflowStep::new("research-saga".to_string(), orchestrator);

// Add to workflow
let workflow = WorkflowBuilder::new()
    .add_step(Box::new(saga_step))
    .add_step(Box::new(notification_step))
    .build();
```

## Testing Sagas

### Unit Testing
```rust
#[tokio::test]
async fn test_saga_success() {
    let orchestrator = create_test_saga();
    let context = SagaContext::new("test".to_string(), WorkflowContext::new(10));

    let result = orchestrator.execute(context).await.unwrap();
    assert!(matches!(result, SagaResult::Completed(_)));
}

#[tokio::test]
async fn test_saga_compensation() {
    let orchestrator = create_failing_saga();
    let context = SagaContext::new("test".to_string(), WorkflowContext::new(10));

    let result = orchestrator.execute(context).await.unwrap();
    assert!(matches!(result, SagaResult::Compensated { .. }));
}
```

### Integration Testing
```rust
#[tokio::test]
async fn test_llm_saga_integration() {
    // Test with real LLM calls
    let agent = create_test_agent().await;
    let orchestrator = create_llm_saga(agent);

    let result = orchestrator.execute(context).await.unwrap();
    assert!(matches!(result, SagaResult::Completed(_)));
}
```

## Performance Considerations

### Concurrent Execution
- Saga steps can run concurrently when independent
- Use `tokio::join!` for parallel compensation

### Resource Management
- Limit concurrent LLM calls to avoid rate limits
- Use connection pooling for external services
- Implement circuit breakers for unreliable services

### State Management
- Persist saga state for long-running workflows
- Use efficient serialization (e.g., JSON, MessagePack)
- Implement state compression for large contexts

## Troubleshooting

### Common Issues

**Compensation Failures**
```
Cause: Compensation logic errors or external service unavailability
Solution: Make compensations idempotent and handle external failures gracefully
```

**Deadlocks**
```
Cause: Circular dependencies between saga steps
Solution: Design acyclic step dependencies and use timeouts
```

**State Corruption**
```
Cause: Concurrent access to shared state
Solution: Use proper locking and atomic operations
```

**Memory Leaks**
```
Cause: Large contexts not cleaned up after compensation
Solution: Implement proper cleanup in compensation handlers
```

## Advanced Patterns

### Nested Sagas
```rust
// Parent saga contains child sagas
let child_saga = create_child_saga();
let parent_step = SagaStep::new(
    "child_process",
    "Execute Child Saga",
    move |ctx| {
        // Execute child saga
        let child_result = child_saga.execute(ctx.clone()).await?;
        Ok(child_result)
    },
    // Compensation handles child saga rollback
    move |ctx, result| { /* ... */ },
);
```

### Dynamic Sagas
```rust
// Build saga based on runtime conditions
let mut orchestrator = SagaOrchestrator::new();

if needs_validation {
    orchestrator = orchestrator.add_step(validation_step);
}

if use_premium_service {
    orchestrator = orchestrator.add_step(premium_processing_step);
}
```

### Saga Orchestration
```rust
// Coordinate multiple independent sagas
let sagas = vec![saga1, saga2, saga3];

for saga in sagas {
    let result = saga.execute(context.clone()).await?;
    match result {
        SagaResult::Completed(_) => continue,
        SagaResult::Compensated { .. } => {
            // Handle partial failure
            break;
        }
        _ => return Err("Saga failed".into()),
    }
}
```

## Migration Guide

### From Basic Workflows
```rust
// Before: Basic workflow
let workflow = WorkflowBuilder::new()
    .add_step(step1)
    .add_step(step2)
    .build();

// After: Saga workflow
let saga = SagaOrchestrator::new()
    .add_step(step1_with_compensation)
    .add_step(step2_with_compensation);

let saga_step = SagaWorkflowStep::new("saga".to_string(), saga);
let workflow = WorkflowBuilder::new()
    .add_step(Box::new(saga_step))
    .build();
```

### From Manual Compensation
```rust
// Before: Manual error handling
match do_operation() {
    Ok(result) => {
        match do_next_operation() {
            Ok(_) => Ok(()),
            Err(e) => {
                undo_operation();
                Err(e)
            }
        }
    }
    Err(e) => Err(e)
}

// After: Saga pattern
let step1 = SagaStep::new("op1", "Operation 1", do_operation, undo_operation);
let step2 = SagaStep::new("op2", "Operation 2", do_next_operation, undo_next_operation);

let saga = SagaOrchestrator::new()
    .add_step(step1)
    .add_step(step2);

saga.execute(context).await?;
```
