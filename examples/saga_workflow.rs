//! Comprehensive saga pattern example
//!
//! This example demonstrates a complete saga workflow for an e-commerce order:
//! 1. Reserve inventory
//! 2. Charge payment
//! 3. Send confirmation email
//!
//! If any step fails, compensations execute in reverse order.

use the_agency::{
    AgentError, Result, SagaContext, SagaOrchestrator, SagaResult, SagaStep, WorkflowContext,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("Saga Pattern Workflow Example\n");
    println!("Scenario: E-commerce Order Processing\n");

    // Simulate a successful order
    println!("=== Test 1: Successful Order ===");
    run_successful_order().await?;

    println!("\n=== Test 2: Failed Payment (with compensation) ===");
    run_failed_payment().await?;

    println!("\n=== Test 3: Failed Confirmation (with compensation) ===");
    run_failed_confirmation().await?;

    Ok(())
}

/// Simulate a successful order processing
async fn run_successful_order() -> Result<()> {
    let orchestrator = create_order_saga(false, false);
    let context = WorkflowContext::new(10);
    let saga_context = SagaContext::new("successful-order".to_string(), context);

    match orchestrator.execute(saga_context).await? {
        SagaResult::Completed(result) => {
            println!("✅ Order completed successfully!");
            println!("   Final result: {}", result);
        }
        _ => println!("❌ Unexpected result"),
    }

    Ok(())
}

/// Simulate a payment failure
async fn run_failed_payment() -> Result<()> {
    let orchestrator = create_order_saga(true, false);
    let context = WorkflowContext::new(10);
    let saga_context = SagaContext::new("failed-payment-order".to_string(), context);

    match orchestrator.execute(saga_context).await? {
        SagaResult::Compensated {
            error,
            compensated_steps,
        } => {
            println!("⚠️  Order failed but successfully compensated");
            println!("   Error: {}", error);
            println!("   Compensated steps: {:?}", compensated_steps);
        }
        _ => println!("❌ Unexpected result"),
    }

    Ok(())
}

/// Simulate a confirmation failure
async fn run_failed_confirmation() -> Result<()> {
    let orchestrator = create_order_saga(false, true);
    let context = WorkflowContext::new(10);
    let saga_context = SagaContext::new("failed-confirmation-order".to_string(), context);

    match orchestrator.execute(saga_context).await? {
        SagaResult::Compensated {
            error,
            compensated_steps,
        } => {
            println!("⚠️  Order failed but successfully compensated");
            println!("   Error: {}", error);
            println!("   Compensated steps: {:?}", compensated_steps);
        }
        _ => println!("❌ Unexpected result"),
    }

    Ok(())
}

/// Create an order processing saga
fn create_order_saga(fail_payment: bool, fail_confirmation: bool) -> SagaOrchestrator {
    // Step 1: Reserve Inventory
    let reserve_inventory = SagaStep::new(
        "reserve_inventory",
        "Reserve Inventory",
        |_ctx| {
            println!("   📦 Reserving inventory for order...");
            std::thread::sleep(std::time::Duration::from_millis(100));
            println!("      → Reserved 5 units of product SKU-123");
            Ok(serde_json::json!({
                "order_id": "ORD-12345",
                "product_sku": "SKU-123",
                "quantity": 5,
                "reservation_id": "RES-67890"
            }))
        },
        |_ctx, result| {
            println!("   🔄 Compensating inventory reservation...");
            if let Some(reservation_id) = result.get("reservation_id") {
                println!("      → Released reservation: {}", reservation_id);
            }
            Ok(())
        },
    )
    .with_retries(2);

    // Step 2: Charge Payment
    let charge_payment = SagaStep::new(
        "charge_payment",
        "Charge Payment",
        move |_ctx| {
            println!("   💳 Processing payment...");
            std::thread::sleep(std::time::Duration::from_millis(100));

            if fail_payment {
                println!("      ❌ Payment failed: Insufficient funds");
                return Err(AgentError::Workflow(
                    "Payment declined: Insufficient funds".to_string(),
                ));
            }

            println!("      → Charged $99.99 to card ending in 1234");
            Ok(serde_json::json!({
                "transaction_id": "TXN-ABCDEF",
                "amount": 99.99,
                "currency": "USD",
                "card_last4": "1234"
            }))
        },
        |_ctx, result| {
            println!("   🔄 Compensating payment...");
            if let Some(transaction_id) = result.get("transaction_id") {
                println!("      → Refunded transaction: {}", transaction_id);
            }
            Ok(())
        },
    )
    .with_retries(1);

    // Step 3: Send Confirmation
    let send_confirmation = SagaStep::new(
        "send_confirmation",
        "Send Confirmation Email",
        move |_ctx| {
            println!("   📧 Sending confirmation email...");
            std::thread::sleep(std::time::Duration::from_millis(100));

            if fail_confirmation {
                println!("      ❌ Email service unavailable");
                return Err(AgentError::Network(
                    "Email service temporarily unavailable".to_string(),
                ));
            }

            println!("      → Sent confirmation to customer@example.com");
            Ok(serde_json::json!({
                "email_sent": true,
                "recipient": "customer@example.com",
                "email_id": "EMAIL-XYZ123"
            }))
        },
        |_ctx, result| {
            println!("   🔄 Compensating email confirmation...");
            if let Some(email_id) = result.get("email_id") {
                println!("      → Sent cancellation email (ref: {})", email_id);
            }
            Ok(())
        },
    )
    .with_retries(2);

    // Build the orchestrator
    SagaOrchestrator::new()
        .add_step(reserve_inventory)
        .add_step(charge_payment)
        .add_step(send_confirmation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_successful_saga() {
        let result = run_successful_order().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_failed_payment_compensation() {
        let result = run_failed_payment().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_failed_confirmation_compensation() {
        let result = run_failed_confirmation().await;
        assert!(result.is_ok());
    }
}
