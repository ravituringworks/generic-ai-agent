//! Example demonstrating workflow suspend and resume functionality
//! 
//! This example shows how to:
//! - Configure workflows with suspend/resume capability
//! - Create suspendable workflow steps
//! - Handle different suspend scenarios
//! - Resume workflows from snapshots

use generic_ai_agent::{
    workflow::{
        WorkflowEngine, WorkflowContext, WorkflowSuspendConfig, FileSnapshotStorage,
        HumanApprovalStep, RateLimitedApiStep, EnhancedMemoryRetrievalStep, SuspendReason
    },
    llm::user_message,
};
use std::path::PathBuf;
use tokio;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::init();

    println!("🚀 Workflow Suspend & Resume Demo");
    println!("==================================\n");

    // Setup snapshot storage
    let storage_dir = PathBuf::from("./examples/snapshots");
    let snapshot_storage = FileSnapshotStorage::new(&storage_dir);
    
    // Configure suspend/resume settings
    let suspend_config = WorkflowSuspendConfig {
        auto_checkpoint: true,
        checkpoint_interval: 2,
        max_snapshots: 5,
        snapshot_retention: chrono::Duration::days(1),
    };

    // Create workflow engine with suspend/resume capability
    let engine = WorkflowEngine::new()
        .with_suspend_config(suspend_config)
        .with_snapshot_storage(Box::new(snapshot_storage))
        .add_step(Box::new(EnhancedMemoryRetrievalStep))
        .add_step(Box::new(HumanApprovalStep::new(
            "Do you approve this action?".to_string()
        )))
        .add_step(Box::new(RateLimitedApiStep::new(
            "external_api".to_string(),
            2 // 2 calls per minute
        )));

    // Demo 1: Basic workflow suspension
    println!("📋 Demo 1: Human Approval Workflow");
    println!("-----------------------------------");
    
    let mut context = WorkflowContext::new(10);
    context.add_message(user_message("Please process this important request"));
    
    let result = engine.execute(context).await?;
    
    if !result.completed {
        println!("✅ Workflow suspended as expected");
        println!("   Response: {}", result.response);
        
        // List available snapshots
        let snapshots = engine.list_snapshots(None).await?;
        if let Some(snapshot) = snapshots.first() {
            println!("   Snapshot ID: {}", snapshot.id);
            println!("   Suspended at: {}", snapshot.created_at);
            println!("   Reason: {:?}", snapshot.suspend_reason);
            
            // Demo 2: Resume with approval
            println!("\n📋 Demo 2: Resume with Approval");
            println!("-------------------------------");
            
            // Simulate human approval by modifying the snapshot
            let mut resumed_context = snapshot.context.clone();
            resumed_context.metadata.insert("human_approval".to_string(), "granted".to_string());
            
            // Create a new snapshot with the approval
            let approved_snapshot = engine.create_snapshot(
                &resumed_context, 
                snapshot.current_step, 
                SuspendReason::Manual
            ).await?;
            
            // Store the approved snapshot
            engine.store_snapshot(&approved_snapshot).await?;
            
            let resumed_result = engine.resume_from_snapshot(approved_snapshot.id).await?;
            println!("✅ Workflow resumed and completed");
            println!("   Final response: {}", resumed_result.response);
        }
    }

    // Demo 3: Rate limiting scenario
    println!("\n📋 Demo 3: Rate Limiting Scenario");
    println!("----------------------------------");
    
    let mut context = WorkflowContext::new(10);
    context.add_message(user_message("Make multiple API calls"));
    
    // Simulate multiple rapid API calls
    for i in 1..=3 {
        println!("API call attempt #{}", i);
        
        let result = engine.execute(context.clone()).await?;
        
        if result.response.contains("rate limit") || result.response.contains("suspended") {
            println!("⏸️  Rate limit hit, workflow suspended");
            
            // Wait a moment and try to resume
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            
            let snapshots = engine.list_snapshots(None).await?;
            if let Some(latest_snapshot) = snapshots.first() {
                println!("🔄 Attempting to resume...");
                let resumed_result = engine.resume_from_snapshot(latest_snapshot.id).await?;
                
                if resumed_result.completed {
                    println!("✅ Successfully resumed after rate limit");
                } else {
                    println!("⏸️  Still suspended (may need more time)");
                }
            }
            break;
        } else {
            println!("✅ API call #{} successful", i);
        }
    }

    // Demo 4: Snapshot management
    println!("\n📋 Demo 4: Snapshot Management");
    println!("------------------------------");
    
    let all_snapshots = engine.list_snapshots(None).await?;
    println!("Total snapshots: {}", all_snapshots.len());
    
    for (i, snapshot) in all_snapshots.iter().enumerate() {
        println!("  {}. ID: {}", i + 1, snapshot.id);
        println!("     Created: {}", snapshot.created_at);
        println!("     Reason: {:?}", snapshot.suspend_reason);
        println!("     Step: {}", snapshot.current_step);
        println!();
    }
    
    // Clean up old snapshots
    let cleanup_count = engine.cleanup_snapshots().await.unwrap_or(0);
    println!("🧹 Cleaned up {} old snapshots", cleanup_count);

    // Demo 5: Complex workflow with checkpoints
    println!("\n📋 Demo 5: Complex Workflow with Auto-Checkpoints");
    println!("--------------------------------------------------");
    
    let complex_engine = WorkflowEngine::new()
        .with_suspend_config(WorkflowSuspendConfig {
            auto_checkpoint: true,
            checkpoint_interval: 1, // Checkpoint after every step
            max_snapshots: 3,
            snapshot_retention: chrono::Duration::hours(1),
        })
        .with_snapshot_storage(Box::new(FileSnapshotStorage::new(&storage_dir)))
        .add_step(Box::new(EnhancedMemoryRetrievalStep))
        .add_step(Box::new(RateLimitedApiStep::new("step1_api".to_string(), 10)))
        .add_step(Box::new(RateLimitedApiStep::new("step2_api".to_string(), 10)))
        .add_step(Box::new(HumanApprovalStep::new("Final approval required".to_string())));

    let mut complex_context = WorkflowContext::new(20);
    complex_context.add_message(user_message("Execute complex multi-step workflow"));
    
    let complex_result = complex_engine.execute(complex_context).await?;
    
    if !complex_result.completed {
        println!("✅ Complex workflow suspended (auto-checkpoints created)");
        
        let complex_snapshots = complex_engine.list_snapshots(None).await?;
        println!("   Created {} checkpoints", complex_snapshots.len());
        
        for snapshot in &complex_snapshots {
            if matches!(snapshot.suspend_reason, SuspendReason::Scheduled) {
                println!("   📸 Checkpoint at step {}", snapshot.current_step);
            }
        }
    }

    println!("\n🎉 Suspend & Resume demo completed!");
    println!("Check the ./examples/snapshots directory for saved workflow states.");

    Ok(())
}

/// Helper function to demonstrate workflow state inspection
async fn inspect_workflow_state(engine: &WorkflowEngine, snapshot_id: Uuid) -> anyhow::Result<()> {
    if let Ok(snapshots) = engine.list_snapshots(None).await {
        if let Some(snapshot) = snapshots.iter().find(|s| s.id == snapshot_id) {
            println!("🔍 Workflow State Inspection");
            println!("   ID: {}", snapshot.id);
            println!("   Created: {}", snapshot.created_at);
            println!("   Current Step: {}", snapshot.current_step);
            println!("   Reason: {:?}", snapshot.suspend_reason);
            println!("   Messages: {}", snapshot.context.messages.len());
            println!("   Memories: {}", snapshot.context.memories.len());
            println!("   Tool Results: {}", snapshot.context.tool_results.len());
            println!("   Metadata: {:?}", snapshot.metadata);
            println!("   Step State: {:?}", snapshot.step_state);
        }
    }
    Ok(())
}