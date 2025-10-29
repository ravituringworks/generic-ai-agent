//! Simplified example demonstrating workflow integration patterns
//!
//! This example demonstrates:
//! - Using agent-like functionality within workflow steps
//! - Using tool-like functionality within workflow steps
//! - Creating reusable workflow components
//! - Mapping workflow input for different processing patterns
//! - Composing complex business logic with reusable components

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use the_agency::{
    error::Result,
    workflow::{
        MapperFn, StepSchema, WorkflowBuilder, WorkflowContext, WorkflowDecision, WorkflowStep,
    },
};
use tokio;

/// Mock agent-like functionality within a workflow step
pub struct TextAnalysisStep {
    pub name: String,
    pub analysis_type: String,
}

impl TextAnalysisStep {
    pub fn new(name: &str, analysis_type: &str) -> Self {
        Self {
            name: name.to_string(),
            analysis_type: analysis_type.to_string(),
        }
    }

    /// Mock LLM-like text generation
    async fn generate_analysis(&self, input: &str) -> Result<String> {
        // Simulate LLM processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let analysis = match self.analysis_type.as_str() {
            "sentiment" => {
                if input.contains("amazing")
                    || input.contains("great")
                    || input.contains("excellent")
                {
                    "Sentiment: POSITIVE (confidence: 0.95)"
                } else if input.contains("bad")
                    || input.contains("terrible")
                    || input.contains("awful")
                {
                    "Sentiment: NEGATIVE (confidence: 0.90)"
                } else {
                    "Sentiment: NEUTRAL (confidence: 0.75)"
                }
            }
            "summary" => {
                let word_count = input.split_whitespace().count();
                &format!("Summary: Input contains {} words. Key themes detected based on content analysis.", word_count)
            }
            "language" => {
                if input.chars().any(|c| c.is_ascii_alphabetic()) {
                    "Language: English (confidence: 0.98)"
                } else {
                    "Language: Unknown (confidence: 0.10)"
                }
            }
            _ => "Analysis: Generic text processing completed",
        };

        Ok(analysis.to_string())
    }
}

#[async_trait]
impl WorkflowStep for TextAnalysisStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        println!(
            "  🤖 Agent-like step '{}' performing {} analysis",
            self.name, self.analysis_type
        );

        // Get input from context
        let input_text = context
            .metadata
            .get("current_text")
            .cloned()
            .unwrap_or_else(|| "default input".to_string());

        // Perform agent-like analysis
        let analysis_result = self.generate_analysis(&input_text).await?;

        // Store result in context
        context
            .metadata
            .insert(format!("{}_result", self.name), analysis_result.clone());

        // Update current text for next step
        context
            .metadata
            .insert("current_text".to_string(), analysis_result.clone());

        println!(
            "  ✅ Agent-like step '{}' completed: {}",
            self.name, analysis_result
        );
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Mock tool-like functionality within a workflow step
pub struct DataTransformStep {
    pub name: String,
    pub transform_type: String,
}

impl DataTransformStep {
    pub fn new(name: &str, transform_type: &str) -> Self {
        Self {
            name: name.to_string(),
            transform_type: transform_type.to_string(),
        }
    }

    /// Mock tool execution
    async fn execute_tool(&self, input: &str) -> Result<Value> {
        // Simulate tool processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(75)).await;

        let result = match self.transform_type.as_str() {
            "uppercase" => {
                json!({
                    "result": input.to_uppercase(),
                    "transformation": "uppercase",
                    "original_length": input.len()
                })
            }
            "word_count" => {
                let word_count = input.split_whitespace().count();
                json!({
                    "result": word_count,
                    "transformation": "word_count",
                    "original_text": input
                })
            }
            "extract_keywords" => {
                let words: Vec<&str> = input
                    .split_whitespace()
                    .filter(|w| w.len() > 4) // Simple keyword extraction
                    .collect();
                json!({
                    "result": words,
                    "transformation": "extract_keywords",
                    "keyword_count": words.len()
                })
            }
            "reverse" => {
                let reversed: String = input.chars().rev().collect();
                json!({
                    "result": reversed,
                    "transformation": "reverse",
                    "original": input
                })
            }
            _ => json!({
                "error": "unknown transformation type",
                "requested": self.transform_type
            }),
        };

        Ok(result)
    }
}

#[async_trait]
impl WorkflowStep for DataTransformStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        println!(
            "  🔧 Tool-like step '{}' executing {} transformation",
            self.name, self.transform_type
        );

        // Get input from context
        let input_text = context
            .metadata
            .get("current_text")
            .cloned()
            .unwrap_or_else(|| "default input".to_string());

        // Execute tool-like transformation
        let tool_result = self.execute_tool(&input_text).await?;

        // Store detailed result in context
        context
            .metadata
            .insert(format!("{}_result", self.name), tool_result.to_string());

        // Update current text with the result for next step
        if let Some(result_value) = tool_result.get("result") {
            let result_text = match result_value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Array(arr) => format!("{:?}", arr),
                _ => result_value.to_string(),
            };
            context
                .metadata
                .insert("current_text".to_string(), result_text);
        }

        println!(
            "  ✅ Tool-like step '{}' completed transformation",
            self.name
        );
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Helper function to create agent-like steps (analogous to createStep(agent))
pub fn create_analysis_step(
    name: &str,
    analysis_type: &str,
) -> Box<dyn WorkflowStep + Send + Sync> {
    Box::new(TextAnalysisStep::new(name, analysis_type))
}

/// Helper function to create tool-like steps (analogous to createStep(tool))
pub fn create_transform_step(
    name: &str,
    transform_type: &str,
) -> Box<dyn WorkflowStep + Send + Sync> {
    Box::new(DataTransformStep::new(name, transform_type))
}

/// Create a reusable text processing workflow (analogous to workflow-as-tool pattern)
pub fn create_text_processing_workflow() -> WorkflowBuilder {
    WorkflowBuilder::new("text_processing_workflow")
        .with_input_schema(
            StepSchema::new_object()
                .add_property("text", "string")
                .add_required("text"),
        )
        // Tool-like preprocessing
        .then(create_transform_step("preprocess", "extract_keywords"))
        // Agent-like analysis
        .then(create_analysis_step("analyze", "sentiment"))
        // Tool-like post-processing
        .then(create_transform_step("postprocess", "word_count"))
        .with_initial_data(
            json!({"current_text": "This is an amazing product with excellent features!"}),
        )
}

/// Complex step that uses both agent-like and tool-like functionality
pub struct HybridProcessingStep {
    pub name: String,
}

impl HybridProcessingStep {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl WorkflowStep for HybridProcessingStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        println!(
            "  🔄 Hybrid step '{}' performing combined processing",
            self.name
        );

        let input_text = context
            .metadata
            .get("current_text")
            .cloned()
            .unwrap_or_else(|| "default input".to_string());

        // Tool-like functionality: Transform text
        let transformed = input_text.to_uppercase();

        // Agent-like functionality: Analyze the transformed text
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        let analysis = if transformed.contains("AMAZING") || transformed.contains("EXCELLENT") {
            "Analysis: Highly positive content detected with strong emotional indicators"
        } else if transformed.contains("BAD") || transformed.contains("TERRIBLE") {
            "Analysis: Negative content detected requiring attention"
        } else {
            "Analysis: Neutral content with standard processing requirements"
        };

        // Combine results
        let combined_result = format!("Transformed: {} | {}", transformed, analysis);

        context
            .metadata
            .insert(format!("{}_result", self.name), combined_result.clone());
        context
            .metadata
            .insert("current_text".to_string(), combined_result);

        println!(
            "  ✅ Hybrid step '{}' completed combined processing",
            self.name
        );
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

    println!("🚀 Workflow Integration Patterns Demo");
    println!("=====================================\n");

    // Demo 1: Agent-like functionality within workflow steps
    println!("📋 Demo 1: Agent-like Steps (Natural Language Processing)");
    println!("----------------------------------------------------------");

    let agent_workflow = WorkflowBuilder::new("agent_like_demo")
        .with_initial_data(
            json!({"current_text": "This amazing product has excellent features that work great!"}),
        )
        .then(create_analysis_step("sentiment_analyzer", "sentiment"))
        .then(create_analysis_step("language_detector", "language"))
        .then(create_analysis_step("summarizer", "summary"))
        .build();

    let context = WorkflowContext::new(10);
    let result = agent_workflow.execute(context).await?;
    println!(
        "Agent-like workflow completed: Steps executed = {}\n",
        result.steps_executed
    );

    // Demo 2: Tool-like functionality within workflow steps
    println!("📋 Demo 2: Tool-like Steps (Data Transformation)");
    println!("-------------------------------------------------");

    let tool_workflow = WorkflowBuilder::new("tool_like_demo")
        .with_initial_data(json!({"current_text": "Hello World from the amazing Rust programming language ecosystem"}))
        .then(create_transform_step("extractor", "extract_keywords"))
        .then(create_transform_step("counter", "word_count"))
        .then(create_transform_step("transformer", "uppercase"))
        .then(create_transform_step("reverser", "reverse"))
        .build();

    let context = WorkflowContext::new(10);
    let result = tool_workflow.execute(context).await?;
    println!(
        "Tool-like workflow completed: Steps executed = {}\n",
        result.steps_executed
    );

    // Demo 3: Mixed workflow with agent-like and tool-like steps
    println!("📋 Demo 3: Mixed Agent-Tool Workflow");
    println!("------------------------------------");

    let mixed_workflow = WorkflowBuilder::new("mixed_demo")
        .with_initial_data(json!({"current_text": "The quick brown fox jumps over the lazy dog in an amazing way"}))
        // Start with tool-like processing
        .then(create_transform_step("keyword_extract", "extract_keywords"))
        // Follow with agent-like analysis
        .then(create_analysis_step("sentiment_check", "sentiment"))
        // More tool processing
        .then(create_transform_step("word_counter", "word_count"))
        // Final agent-like summary
        .then(create_analysis_step("final_summary", "summary"))
        .build();

    let context = WorkflowContext::new(15);
    let result = mixed_workflow.execute(context).await?;
    println!(
        "Mixed workflow completed: Steps executed = {}\n",
        result.steps_executed
    );

    // Demo 4: Workflow with input mapping (like .map() for agent compatibility)
    println!("📋 Demo 4: Input Mapping for Agent Compatibility");
    println!("-----------------------------------------------");

    // Mapper to format input for agent-like processing
    let agent_prompt_mapper: MapperFn = Arc::new(|_context, input_data| {
        let original_text = input_data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("I absolutely love this new feature! It's incredible and works perfectly.");

        // Format as a structured prompt for analysis
        let formatted_prompt = format!(
            "Please perform comprehensive analysis on the following text:\n\nInput: '{}'\n\nProvide sentiment, language detection, and summary.",
            original_text
        );

        // Return the original text so the workflow can process it
        json!({"current_text": original_text})
    });

    let mapped_workflow = WorkflowBuilder::new("mapped_input_demo")
        .with_input_schema(
            StepSchema::new_object()
                .add_property("text", "string")
                .add_required("text")
        )
        .with_initial_data(json!({"text": "I absolutely love this new feature! It's incredible and works perfectly."}))
        .map(agent_prompt_mapper)
        .then(create_analysis_step("comprehensive_analysis", "sentiment"))
        .then(create_analysis_step("language_analysis", "language"))
        .build();

    let context = WorkflowContext::new(10);
    let result = mapped_workflow.execute(context).await?;
    println!(
        "Mapped workflow completed: Steps executed = {}\n",
        result.steps_executed
    );

    // Demo 5: Reusable workflow as a component (workflow-as-tool pattern)
    println!("📋 Demo 5: Reusable Workflow Component");
    println!("--------------------------------------");

    let reusable_workflow = create_text_processing_workflow().build();

    println!("Executing reusable text processing workflow...");
    let context = WorkflowContext::new(15);
    let result = reusable_workflow.execute(context).await?;
    println!(
        "Reusable workflow completed: Steps executed = {}\n",
        result.steps_executed
    );

    // Demo 6: Hybrid step combining agent and tool functionality
    println!("📋 Demo 6: Hybrid Agent-Tool Processing");
    println!("---------------------------------------");

    let hybrid_workflow = WorkflowBuilder::new("hybrid_demo")
        .with_initial_data(json!({"current_text": "This excellent system provides amazing results with great performance"}))
        .then(Box::new(HybridProcessingStep::new("hybrid_processor")))
        .then(create_analysis_step("final_analysis", "summary"))
        .build();

    let context = WorkflowContext::new(10);
    let result = hybrid_workflow.execute(context).await?;
    println!(
        "Hybrid workflow completed: Steps executed = {}\n",
        result.steps_executed
    );

    // Demo 7: Complex nested workflow with multiple integration patterns
    println!("📋 Demo 7: Complex Multi-Pattern Workflow");
    println!("-----------------------------------------");

    let complex_workflow = WorkflowBuilder::new("complex_integration_demo")
        .with_input_schema(
            StepSchema::new_object()
                .add_property("documents", "array")
                .add_property("processing_mode", "string")
        )
        .with_initial_data(json!({
            "documents": [
                "Document 1: Great product with amazing features!",
                "Document 2: This needs some improvement but has potential.",
                "Document 3: Excellent service and outstanding support!"
            ],
            "processing_mode": "comprehensive",
            "current_text": "Multi-document analysis workflow processing comprehensive text evaluation"
        }))
        // Preprocessing phase (tool-like)
        .then(create_transform_step("preprocess_keywords", "extract_keywords"))
        // Analysis phase (agent-like)
        .then(create_analysis_step("document_sentiment", "sentiment"))
        // Hybrid processing
        .then(Box::new(HybridProcessingStep::new("hybrid_analysis")))
        // Final tool processing
        .then(create_transform_step("final_count", "word_count"))
        // Final agent summary
        .then(create_analysis_step("comprehensive_summary", "summary"))
        .build();

    let context = WorkflowContext::new(20);
    let result = complex_workflow.execute(context).await?;
    println!(
        "Complex workflow completed: Steps executed = {}",
        result.steps_executed
    );

    println!("\n🎉 Workflow Integration Patterns Demo Completed!");
    println!("\n💡 Key Integration Patterns Demonstrated:");
    println!("   • Agent-like Steps - Natural language processing within workflows");
    println!("   • Tool-like Steps - Data transformation and structured processing");
    println!("   • Mixed Workflows - Combining natural language and data processing");
    println!("   • Input Mapping - Transforming data for step compatibility");
    println!("   • Reusable Components - Workflow-as-tool pattern for modularity");
    println!("   • Hybrid Processing - Steps that combine multiple processing types");
    println!("   • Complex Composition - Multi-pattern workflows for comprehensive tasks");

    println!("\n🔧 Architecture Benefits:");
    println!("   • Composable - Steps can be mixed and matched");
    println!("   • Reusable - Workflows become building blocks for larger systems");
    println!("   • Flexible - Support for both structured data and natural language");
    println!("   • Scalable - Complex logic built from simple, testable components");

    Ok(())
}
