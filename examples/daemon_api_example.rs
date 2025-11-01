//! Example demonstrating the Agency daemon API and saga pattern
//!
//! This example shows:
//! - How to interact with the REST API
//! - Creating and executing workflows
//! - Using the saga pattern for distributed transactions
//!
//! To run this example:
//! 1. Start the daemon: cargo run --bin agency-daemon
//! 2. In another terminal: cargo run --example daemon_api_example

use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Agency Daemon API Example\n");

    let client = Client::new();
    let base_url = "http://127.0.0.1:8080";

    // 1. Health check
    println!("1. Checking API health...");
    let response = client.get(format!("{}/health", base_url)).send().await?;
    println!("   Status: {}", response.status());
    println!("   Response: {}\n", response.text().await?);

    // 2. Process a simple message
    println!("2. Processing a message...");
    let response = client
        .post(format!("{}/api/v1/agent/process", base_url))
        .json(&json!({
            "message": "What is Rust programming language?",
            "max_steps": 5
        }))
        .send()
        .await?;
    println!("   Status: {}", response.status());
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("   Response: {}\n", serde_json::to_string_pretty(&result)?);
    } else {
        println!("   Error: {}\n", response.text().await?);
    }

    // 3. Create a workflow
    println!("3. Creating a workflow...");
    let response = client
        .post(format!("{}/api/v1/workflows", base_url))
        .json(&json!({
            "workflow_id": "example-workflow-001",
            "initial_message": "Process long-running task",
            "max_steps": 20
        }))
        .send()
        .await?;
    println!("   Status: {}", response.status());
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("   Response: {}\n", serde_json::to_string_pretty(&result)?);
    } else {
        println!("   Error: {}\n", response.text().await?);
    }

    // 4. List workflow snapshots
    println!("4. Listing workflow snapshots...");
    let response = client
        .get(format!("{}/api/v1/workflows/snapshots", base_url))
        .send()
        .await?;
    println!("   Status: {}", response.status());
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!(
            "   Found snapshots: {}\n",
            result.as_array().map(|a| a.len()).unwrap_or(0)
        );
    } else {
        println!("   Error: {}\n", response.text().await?);
    }

    // 5. Demonstrate saga pattern conceptually
    println!("5. Saga Pattern Example (conceptual):");
    println!("   Saga patterns are used for distributed transactions:");
    println!("   - Step 1: Reserve Inventory → Compensation: Release Inventory");
    println!("   - Step 2: Process Payment → Compensation: Refund Payment");
    println!("   - Step 3: Send Confirmation → Compensation: Send Cancellation");
    println!("   If any step fails, compensations run in reverse order.\n");

    println!("Example completed!");
    println!("\nTo explore more:");
    println!("  - Check docs/DEPLOYMENT.md for full API documentation");
    println!("  - See examples/saga_workflow.rs for saga pattern code");
    println!("  - Review src/saga.rs for implementation details");

    Ok(())
}
