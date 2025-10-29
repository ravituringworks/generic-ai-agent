//! Comprehensive example demonstrating pause execution functionality
//!
//! This example demonstrates:
//! - sleep(): Pause for a set number of milliseconds
//! - sleepUntil(): Pause until a specific timestamp
//! - waitForEvent(): Pause until an external event is received
//! - sendEvent(): Send an event to resume a waiting workflow

use the_agency::{
    workflow::{
        WorkflowEngine, WorkflowContext, WorkflowSuspendConfig, FileSnapshotStorage,
        EventBus, WorkflowEvent, SleepStep, SleepUntilStep, WaitForEventStep,
        ConditionalPauseStep, PauseType, EnhancedMemoryRetrievalStep
    },
    llm::user_message,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio;
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::init();

    println!("🚀 Pause Execution Demo");
    println!("========================\n");

    // Setup infrastructure
    let storage_dir = PathBuf::from("./examples/pause_snapshots");
    let snapshot_storage = FileSnapshotStorage::new(&storage_dir);
    let event_bus = Arc::new(EventBus::new(50));
    
    let suspend_config = WorkflowSuspendConfig {
        auto_checkpoint: false, // Disable for this demo
        checkpoint_interval: 10,
        max_snapshots: 20,
        snapshot_retention: chrono::Duration::hours(1),
    };

    // Demo 1: Basic sleep() functionality
    println!("📋 Demo 1: Basic Sleep Functionality");
    println!("------------------------------------");
    
    let engine = WorkflowEngine::new()
        .with_suspend_config(suspend_config.clone())
        .with_snapshot_storage(Box::new(FileSnapshotStorage::new(&storage_dir)))
        .with_event_bus(Arc::clone(&event_bus));

    println!("⏱️  Testing direct sleep() method...");
    let start = std::time::Instant::now();
    engine.sleep(500).await?; // Sleep for 500ms
    let elapsed = start.elapsed();
    println!("✅ Slept for {}ms (expected ~500ms)", elapsed.as_millis());

    // Demo 2: sleepUntil() functionality
    println!("\n📋 Demo 2: Sleep Until Timestamp");
    println!("--------------------------------");
    
    let future_time = Utc::now() + Duration::seconds(1);
    println!("⏱️  Testing sleepUntil() until {}", future_time);
    
    let start = std::time::Instant::now();
    engine.sleep_until(future_time).await?;
    let elapsed = start.elapsed();
    println!("✅ Slept until timestamp - elapsed: {}ms", elapsed.as_millis());

    // Demo 3: Workflow with SleepStep
    println!("\n📋 Demo 3: Workflow with Sleep Step");
    println!("-----------------------------------");
    
    let sleep_engine = WorkflowEngine::new()
        .with_suspend_config(suspend_config.clone())
        .with_snapshot_storage(Box::new(FileSnapshotStorage::new(&storage_dir)))
        .with_event_bus(Arc::clone(&event_bus))
        .add_step(Box::new(SleepStep::new(200)));

    let mut context = WorkflowContext::new(10);
    context.add_message(user_message("Execute sleep workflow"));
    
    let result = sleep_engine.execute(context).await?;
    println!("🔄 Workflow result: {}", result.response);
    println!("   Completed: {}", result.completed);
    
    if !result.completed {
        // List snapshots to find the suspended workflow
        let snapshots = sleep_engine.list_snapshots(None).await?;
        if let Some(snapshot) = snapshots.first() {
            println!("   📸 Snapshot created: {}", snapshot.id);
            println!("   💤 Suspend reason: {:?}", snapshot.suspend_reason);
            
            // For demo purposes, we'll manually resume after a short wait
            println!("   ⏳ Waiting 300ms then resuming...");
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            
            let resumed_result = sleep_engine.resume_from_snapshot(snapshot.id).await?;
            println!("   ✅ Resumed - Completed: {}", resumed_result.completed);
        }
    }

    // Demo 4: sleepUntil with SleepUntilStep
    println!("\n📋 Demo 4: Workflow with Sleep Until Step");
    println!("-----------------------------------------");
    
    let wake_time = Utc::now() + Duration::seconds(1);
    let sleep_until_engine = WorkflowEngine::new()
        .with_suspend_config(suspend_config.clone())
        .with_snapshot_storage(Box::new(FileSnapshotStorage::new(&storage_dir)))
        .with_event_bus(Arc::clone(&event_bus))
        .add_step(Box::new(SleepUntilStep::new(wake_time)));

    let mut context = WorkflowContext::new(10);
    context.add_message(user_message("Sleep until specific time"));
    
    let result = sleep_until_engine.execute(context).await?;
    println!("🔄 Sleep until result: {}", result.response);

    // Demo 5: Event-based workflow control
    println!("\n📋 Demo 5: Event-Based Workflow Control");
    println!("---------------------------------------");
    
    let event_engine = WorkflowEngine::new()
        .with_suspend_config(suspend_config.clone())
        .with_snapshot_storage(Box::new(FileSnapshotStorage::new(&storage_dir)))
        .with_event_bus(Arc::clone(&event_bus))
        .add_step(Box::new(WaitForEventStep::new("user_approval".to_string(), Some(5000))));

    let mut context = WorkflowContext::new(10);
    context.add_message(user_message("Wait for user approval"));
    
    // Start the workflow in a separate task
    let event_engine_clone = WorkflowEngine::new()
        .with_suspend_config(suspend_config.clone())
        .with_snapshot_storage(Box::new(FileSnapshotStorage::new(&storage_dir)))
        .with_event_bus(Arc::clone(&event_bus))
        .add_step(Box::new(WaitForEventStep::new("user_approval".to_string(), Some(5000))));
        
    let workflow_handle = tokio::spawn(async move {
        event_engine_clone.execute(context).await
    });
    
    // Wait a moment, then send the approval event
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    println!("📤 Sending user_approval event...");
    let approval_event = WorkflowEvent {
        id: "approval-123".to_string(),
        event_type: "user_approval".to_string(),
        payload: serde_json::json!({
            "approved": true,
            "user": "demo_user",
            "timestamp": Utc::now()
        }),
        timestamp: Utc::now(),
        target_workflow_id: None,
    };
    
    let sent_count = event_bus.send_event(approval_event)?;
    println!("✅ Event sent to {} subscribers", sent_count);
    
    let workflow_result = workflow_handle.await??;
    println!("🔄 Event workflow result: {}", workflow_result.response);

    // Demo 6: Conditional pause workflow
    println!("\n📋 Demo 6: Conditional Pause Workflow");
    println!("-------------------------------------");
    
    let conditional_engine = WorkflowEngine::new()
        .with_suspend_config(suspend_config.clone())
        .with_snapshot_storage(Box::new(FileSnapshotStorage::new(&storage_dir)))
        .with_event_bus(Arc::clone(&event_bus))
        .add_step(Box::new(EnhancedMemoryRetrievalStep))
        .add_step(Box::new(ConditionalPauseStep::new(
            "need_pause".to_string(),
            PauseType::Sleep(100)
        )));

    let mut context = WorkflowContext::new(10);
    context.add_message(user_message("Process with conditional pause"));
    
    // First execution - no pause condition
    let result = conditional_engine.execute(context.clone()).await?;
    println!("🔄 First execution (no pause): Completed = {}", result.completed);
    
    // Second execution - with pause condition
    context.metadata.insert("need_pause".to_string(), "true".to_string());
    let result = conditional_engine.execute(context).await?;
    println!("🔄 Second execution (with pause): {}", result.response);

    // Demo 7: Complex multi-step workflow with various pause types
    println!("\n📋 Demo 7: Complex Multi-Step Workflow");
    println!("--------------------------------------");
    
    let complex_engine = WorkflowEngine::new()
        .with_suspend_config(suspend_config.clone())
        .with_snapshot_storage(Box::new(FileSnapshotStorage::new(&storage_dir)))
        .with_event_bus(Arc::clone(&event_bus))
        .add_step(Box::new(SleepStep::new(50)))  // Quick pause
        .add_step(Box::new(ConditionalPauseStep::new(
            "wait_for_data".to_string(),
            PauseType::WaitForEvent {
                event_id: "data_ready".to_string(),
                timeout_ms: Some(2000)
            }
        )))
        .add_step(Box::new(SleepStep::new(50))); // Another quick pause

    let mut context = WorkflowContext::new(20);
    context.add_message(user_message("Complex workflow execution"));
    context.metadata.insert("wait_for_data".to_string(), "true".to_string());
    
    // Start complex workflow
    let complex_handle = tokio::spawn(async move {
        complex_engine.execute(context).await
    });
    
    // Send data ready event after a delay
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    println!("📤 Sending data_ready event for complex workflow...");
    
    let data_event = WorkflowEvent {
        id: "data-456".to_string(),
        event_type: "data_ready".to_string(),
        payload: serde_json::json!({
            "data": [1, 2, 3, 4, 5],
            "source": "external_api"
        }),
        timestamp: Utc::now(),
        target_workflow_id: None,
    };
    
    event_bus.send_event(data_event)?;
    
    let complex_result = complex_handle.await??;
    println!("🔄 Complex workflow result: {}", complex_result.response);

    // Demo 8: Event Bus operations
    println!("\n📋 Demo 8: Advanced Event Bus Operations");
    println!("----------------------------------------");
    
    // Test multiple subscribers
    println!("🔗 Setting up multiple event subscribers...");
    let mut receiver1 = event_bus.subscribe("broadcast_test");
    let mut receiver2 = event_bus.subscribe("broadcast_test");
    
    println!("📊 Subscriber count for 'broadcast_test': {}", event_bus.subscriber_count("broadcast_test"));
    
    // Send broadcast event
    let broadcast_event = WorkflowEvent {
        id: "broadcast-789".to_string(),
        event_type: "broadcast_test".to_string(),
        payload: serde_json::json!({"message": "Hello to all subscribers!"}),
        timestamp: Utc::now(),
        target_workflow_id: None,
    };
    
    let sent_count = event_bus.send_event(broadcast_event)?;
    println!("📤 Broadcast event sent to {} subscribers", sent_count);
    
    // Receive on both subscribers
    let event1 = receiver1.recv().await?;
    let event2 = receiver2.recv().await?;
    println!("📥 Receiver 1 got: {}", event1.payload["message"]);
    println!("📥 Receiver 2 got: {}", event2.payload["message"]);

    // Demo 9: Pause with timeout
    println!("\n📋 Demo 9: Event Wait with Timeout");
    println!("----------------------------------");
    
    println!("⏳ Testing waitForEvent with 1000ms timeout...");
    let start = std::time::Instant::now();
    let timeout_result = event_bus.wait_for_event("timeout_test", Some(1000)).await?;
    let elapsed = start.elapsed();
    
    match timeout_result {
        Some(event) => println!("📥 Received event: {}", event.id),
        None => println!("⏰ Timeout occurred after {}ms (expected ~1000ms)", elapsed.as_millis()),
    }

    // Demo 10: Workflow state inspection
    println!("\n📋 Demo 10: Workflow State Management");
    println!("------------------------------------");
    
    let snapshots = event_engine.list_snapshots(None).await?;
    println!("📸 Total snapshots created: {}", snapshots.len());
    
    for (i, snapshot) in snapshots.iter().enumerate().take(3) {
        println!("   {}. ID: {} | Created: {} | Reason: {:?}", 
                 i + 1, 
                 snapshot.id, 
                 snapshot.created_at.format("%H:%M:%S"),
                 snapshot.suspend_reason);
    }

    // Cleanup demo
    println!("\n🧹 Cleaning up snapshots...");
    let cleanup_count = event_engine.cleanup_snapshots().await.unwrap_or(0);
    println!("   Cleaned up {} old snapshots", cleanup_count);

    println!("\n🎉 Pause execution demo completed!");
    println!("📁 Check ./examples/pause_snapshots for workflow state files");
    println!("\n💡 Key takeaways:");
    println!("   • sleep() - Direct millisecond-based pausing");
    println!("   • sleepUntil() - Pause until specific timestamp");
    println!("   • waitForEvent() - Event-driven workflow control");
    println!("   • sendEvent() - External workflow triggering");
    println!("   • All pause types integrate with suspend/resume system");
    
    Ok(())
}

/// Helper function to demonstrate workflow event handling patterns
async fn demonstrate_event_patterns(event_bus: &EventBus) -> anyhow::Result<()> {
    println!("🔄 Event Pattern Demonstrations");
    
    // Pattern 1: Request-Response
    println!("   Pattern 1: Request-Response");
    let response_receiver = event_bus.subscribe("response_123");
    
    let request_event = WorkflowEvent {
        id: "request_123".to_string(),
        event_type: "api_request".to_string(),
        payload: serde_json::json!({"endpoint": "/users", "method": "GET"}),
        timestamp: Utc::now(),
        target_workflow_id: None,
    };
    
    event_bus.send_event(request_event)?;
    
    // Simulate response after delay
    tokio::spawn({
        let event_bus = Arc::new(EventBus::new(10)); // Create owned instance for task
        async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let response_event = WorkflowEvent {
                id: "response_123".to_string(),
                event_type: "response_123".to_string(),
                payload: serde_json::json!({"status": 200, "data": {"users": []}}),
                timestamp: Utc::now(),
                target_workflow_id: None,
            };
            let _ = event_bus.send_event(response_event);
        }
    });
    
    // Pattern 2: Workflow coordination
    println!("   Pattern 2: Workflow Coordination");
    let coordination_events = vec!["step_1_complete", "step_2_complete", "step_3_complete"];
    
    for event_type in coordination_events {
        let event = WorkflowEvent {
            id: format!("coord_{}", event_type),
            event_type: event_type.to_string(),
            payload: serde_json::json!({"workflow_id": "main_workflow", "step": event_type}),
            timestamp: Utc::now(),
            target_workflow_id: Some("main_workflow".to_string()),
        };
        event_bus.send_event(event)?;
    }
    
    Ok(())
}