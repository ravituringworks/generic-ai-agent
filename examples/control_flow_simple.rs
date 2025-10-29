//! Simplified working example demonstrating workflow control flow functionality
//!
//! This example demonstrates:
//! - .then() - Sequential step chaining with automatic data passing
//! - .parallel() - Simultaneous step execution  
//! - .branch() - Conditional logic based on context
//! - .dowhile() and .foreach() - Loop constructs
//! - .map() - Data transformation between steps
//! - Input/output schemas and validation

use the_agency::{
    workflow::{
        WorkflowBuilder, WorkflowContext, StepSchema,
        ParallelExecutionStep, ConditionFn, MapperFn, ItemsExtractorFn,
        WorkflowDecision, WorkflowStep
    },
    error::Result,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio;

/// Custom step for demonstration
pub struct DataProcessingStep {
    pub name: String,
    pub processing_time_ms: u64,
}

impl DataProcessingStep {
    pub fn new(name: &str, processing_time_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            processing_time_ms,
        }
    }
}

#[async_trait]
impl WorkflowStep for DataProcessingStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        println!("  🔄 Processing step: {}", self.name);
        
        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(self.processing_time_ms)).await;
        
        // Add some result data to context
        context.metadata.insert(
            format!("{}_result", self.name),
            format!("processed_data_{}", chrono::Utc::now().timestamp_millis())
        );
        
        // For demo purposes, increment a counter
        if self.name == "increment_counter" {
            let current = context.metadata.get("loop_iteration")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            context.metadata.insert("loop_iteration".to_string(), (current + 1).to_string());
            println!("    Counter incremented to: {}", current + 1);
        }
        
        println!("  ✅ Completed step: {}", self.name);
        Ok(WorkflowDecision::Continue)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Workflow Control Flow Demo");
    println!("==============================\n");

    // Demo 1: Sequential execution with .then()
    println!("📋 Demo 1: Sequential Step Chaining (.then())");
    println!("----------------------------------------------");
    
    let sequential_workflow = WorkflowBuilder::new("sequential_demo")
        .with_input_schema(
            StepSchema::new_object()
                .add_property("input", "string")
                .add_required("input")
        )
        .then(Box::new(DataProcessingStep::new("step1", 100)))
        .then(Box::new(DataProcessingStep::new("step2", 150)))
        .then(Box::new(DataProcessingStep::new("step3", 75)))
        .with_initial_data(serde_json::json!({"input": "demo_data"}))
        .build();

    let context = WorkflowContext::new(20);
    let result = sequential_workflow.execute(context).await?;
    println!("Sequential workflow result: Steps executed = {}\n", result.steps_executed);

    // Demo 2: Parallel execution with .parallel()
    println!("📋 Demo 2: Parallel Step Execution (.parallel())");
    println!("-------------------------------------------------");
    
    let parallel_steps: Vec<Box<dyn WorkflowStep + Send + Sync>> = vec![
        Box::new(DataProcessingStep::new("parallel_1", 200)),
        Box::new(DataProcessingStep::new("parallel_2", 150)),
        Box::new(DataProcessingStep::new("parallel_3", 100)),
    ];
    
    let parallel_workflow = WorkflowBuilder::new("parallel_demo")
        .parallel(parallel_steps)
        .then(Box::new(DataProcessingStep::new("after_parallel", 50)))
        .build();

    let context = WorkflowContext::new(20);
    let start_time = std::time::Instant::now();
    let result = parallel_workflow.execute(context).await?;
    let elapsed = start_time.elapsed();
    println!("Parallel workflow completed in {}ms (should be ~200ms for parallel + 50ms sequential)\n", elapsed.as_millis());

    // Demo 3: Conditional branching with .branch()
    println!("📋 Demo 3: Conditional Logic (.branch())");
    println!("-----------------------------------------");
    
    // Condition based on context metadata
    let condition: ConditionFn = Arc::new(|context, _| {
        context.metadata.get("should_take_fast_path") == Some(&"true".to_string())
    });
    
    let branching_workflow = WorkflowBuilder::new("branch_demo")
        .then(Box::new(DataProcessingStep::new("setup", 50)))
        .branch(
            condition,
            Box::new(DataProcessingStep::new("fast_path", 100)),  // if true
            Some(Box::new(DataProcessingStep::new("slow_path", 300))) // if false
        )
        .build();

    // Test false condition (slow path)
    let mut context = WorkflowContext::new(20);
    context.metadata.insert("should_take_fast_path".to_string(), "false".to_string());
    let result = branching_workflow.execute(context).await?;
    println!("Branch workflow (slow path): Steps executed = {}", result.steps_executed);
    
    // Test true condition (fast path)  
    let mut context = WorkflowContext::new(20);
    context.metadata.insert("should_take_fast_path".to_string(), "true".to_string());
    let result = branching_workflow.execute(context).await?;
    println!("Branch workflow (fast path): Steps executed = {}\n", result.steps_executed);

    // Demo 4: Do-while loop with .dowhile()
    println!("📋 Demo 4: Do-While Loop (.dowhile())");
    println!("--------------------------------------");
    
    let loop_condition: ConditionFn = Arc::new(|context, _| {
        let iteration = context.metadata.get("loop_iteration")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        println!("    Loop condition check: iteration = {}", iteration);
        iteration < 3
    });
    
    // Step that increments counter
    let counter_step = DataProcessingStep::new("increment_counter", 75);
    
    let dowhile_workflow = WorkflowBuilder::new("dowhile_demo")
        .dowhile(Box::new(counter_step), loop_condition)
        .build();

    let mut context = WorkflowContext::new(20);
    context.metadata.insert("loop_iteration".to_string(), "0".to_string());
    let result = dowhile_workflow.execute(context).await?;
    println!("Do-while workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 5: For-each loop with .foreach()
    println!("📋 Demo 5: For-Each Loop (.foreach())");
    println!("-------------------------------------");
    
    let items_extractor: ItemsExtractorFn = Arc::new(|_context| {
        vec![
            serde_json::json!({"id": 1, "name": "Item A"}),
            serde_json::json!({"id": 2, "name": "Item B"}), 
            serde_json::json!({"id": 3, "name": "Item C"}),
        ]
    });
    
    let foreach_workflow = WorkflowBuilder::new("foreach_demo")
        .foreach(
            Box::new(DataProcessingStep::new("process_item", 100)),
            items_extractor
        )
        .then(Box::new(DataProcessingStep::new("aggregate_results", 50)))
        .build();

    let context = WorkflowContext::new(20);
    let result = foreach_workflow.execute(context).await?;
    println!("For-each workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 6: Data mapping with .map()
    println!("📋 Demo 6: Data Transformation (.map())");
    println!("---------------------------------------");
    
    let data_mapper: MapperFn = Arc::new(|context, input_data| {
        println!("    Transforming data: {:?}", input_data);
        
        // Transform data by adding processing metadata
        let mut output = input_data.clone();
        output["transformed"] = serde_json::json!(true);
        output["timestamp"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
        output["context_metadata_count"] = serde_json::json!(context.metadata.len());
        
        println!("    Transformed to: {:?}", output);
        output
    });
    
    let mapping_workflow = WorkflowBuilder::new("mapping_demo")
        .then(Box::new(DataProcessingStep::new("generate_data", 100)))
        .map(data_mapper)
        .then(Box::new(DataProcessingStep::new("use_transformed_data", 75)))
        .build();

    let mut context = WorkflowContext::new(20);
    context.metadata.insert("source_data".to_string(), "original_value".to_string());
    let result = mapping_workflow.execute(context).await?;
    println!("Mapping workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 7: Schema validation
    println!("📋 Demo 7: Input/Output Schema Validation");
    println!("-----------------------------------------");
    
    let input_schema = StepSchema::new_object()
        .add_property("user_id", "string")
        .add_property("action", "string")
        .add_property("timestamp", "number")
        .add_required("user_id")
        .add_required("action");
    
    let output_schema = StepSchema::new_object()
        .add_property("result", "string")
        .add_property("processed_at", "string")
        .add_property("success", "boolean");
    
    // Valid input
    let valid_input = serde_json::json!({
        "user_id": "user123",
        "action": "login", 
        "timestamp": 1634567890
    });
    
    // Invalid input (missing required field)
    let invalid_input = serde_json::json!({
        "action": "login",
        "timestamp": 1634567890
    });
    
    println!("Input schema validation:");
    println!("  Valid input: {}", input_schema.validates(&valid_input));
    println!("  Invalid input: {}", input_schema.validates(&invalid_input));
    
    let schema_workflow = WorkflowBuilder::new("schema_demo")
        .with_input_schema(input_schema)
        .with_output_schema(output_schema)
        .then(Box::new(DataProcessingStep::new("validate_and_process", 100)))
        .build();

    let context = WorkflowContext::new(20);
    let result = schema_workflow.execute(context).await?;
    println!("Schema workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 8: Workflow cloning and customization
    println!("📋 Demo 8: Workflow Cloning and Customization");
    println!("---------------------------------------------");
    
    let base_workflow = WorkflowBuilder::new("base_template")
        .with_input_schema(StepSchema::new_object().add_property("data", "string"))
        .with_initial_data(serde_json::json!({"template": "base"}))
        .then(Box::new(DataProcessingStep::new("template_step", 100)));
    
    // Clone and customize for different use cases
    let customized_workflow_1 = base_workflow.clone_workflow("customized_v1")
        .then(Box::new(DataProcessingStep::new("custom_step_1", 75)))
        .with_initial_data(serde_json::json!({"template": "v1"}))
        .build();
    
    let customized_workflow_2 = base_workflow.clone_workflow("customized_v2")
        .then(Box::new(DataProcessingStep::new("custom_step_2", 125)))
        .with_initial_data(serde_json::json!({"template": "v2"}))
        .build();
    
    println!("Cloned workflow 1:");
    let context = WorkflowContext::new(10);
    let result = customized_workflow_1.execute(context).await?;
    println!("  Steps executed: {}", result.steps_executed);
    
    println!("Cloned workflow 2:");
    let context = WorkflowContext::new(10);
    let result = customized_workflow_2.execute(context).await?;
    println!("  Steps executed: {}", result.steps_executed);

    // Demo 9: Workflow State and Metadata
    println!("\n📋 Demo 9: Workflow State and Metadata");
    println!("--------------------------------------");
    
    let metadata_workflow = WorkflowBuilder::new("metadata_demo")
        .then(Box::new(DataProcessingStep::new("collect_metadata", 50)))
        .map(Arc::new(|context, input_data| {
            println!("Context metadata during workflow:");
            for (key, value) in &context.metadata {
                println!("  {}: {}", key, value);
            }
            
            let mut output = input_data.clone();
            output["metadata_keys"] = serde_json::json!(context.metadata.keys().collect::<Vec<_>>());
            output["step_count"] = serde_json::json!(context.step_count);
            output
        }))
        .then(Box::new(DataProcessingStep::new("use_metadata", 75)))
        .build();

    let mut context = WorkflowContext::new(15);
    context.metadata.insert("environment".to_string(), "demo".to_string());
    context.metadata.insert("version".to_string(), "1.0.0".to_string());
    let result = metadata_workflow.execute(context).await?;
    println!("Metadata workflow completed: Steps executed = {}\n", result.steps_executed);

    println!("🎉 Control flow demo completed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • .then() - Sequential step chaining with data passing");
    println!("   • .parallel() - Concurrent step execution");
    println!("   • .branch() - Conditional execution based on context");
    println!("   • .dowhile() - Loop constructs with conditions");
    println!("   • .foreach() - Iteration over data collections");
    println!("   • .map() - Data transformation between steps");
    println!("   • Schema validation for inputs and outputs");
    println!("   • Workflow cloning and customization");
    println!("   • Context metadata management and state tracking");

    Ok(())
}