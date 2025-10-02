//! Comprehensive example demonstrating workflow integration with agents and tools
//!
//! This example demonstrates:
//! - Using agents directly as workflow steps with createStep()
//! - Invoking agents from within step execute functions using .generate()
//! - Using tools directly as workflow steps with createStep()  
//! - Invoking tools from within step execute functions using .execute()
//! - Converting workflows into tools for reuse
//! - Using workflows within agents
//! - Mapping workflow input to agent-compatible prompts
//! - Composing complex business logic with agents, tools, and workflows

use generic_ai_agent::{
    workflow::{
        WorkflowBuilder, WorkflowContext, StepSchema,
        WorkflowDecision, WorkflowStep, MapperFn
    },
    agent::{Agent, AgentConfig},
    tools::{Tool, ToolConfig, ToolInput, ToolResult},
    llm::{LLMClient, user_message},
    error::Result,
};
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::{json, Value};
use tokio;

/// Custom agent for natural language processing tasks
pub struct TextAnalysisAgent {
    pub name: String,
    pub llm_client: Arc<dyn LLMClient>,
    pub config: AgentConfig,
}

impl TextAnalysisAgent {
    pub fn new(name: &str, llm_client: Arc<dyn LLMClient>) -> Self {
        Self {
            name: name.to_string(),
            llm_client,
            config: AgentConfig::default(),
        }
    }
}

#[async_trait]
impl Agent for TextAnalysisAgent {
    async fn generate(&self, input: &str) -> Result<String> {
        println!("  🤖 Agent '{}' processing: {}", self.name, input);
        
        let messages = vec![user_message(input)];
        let response = self.llm_client.generate(&messages).await?;
        
        println!("  ✅ Agent '{}' generated response", self.name);
        Ok(response)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Custom tool for data transformation
pub struct DataTransformTool {
    pub name: String,
    pub transformation_type: String,
}

impl DataTransformTool {
    pub fn new(name: &str, transformation_type: &str) -> Self {
        Self {
            name: name.to_string(),
            transformation_type: transformation_type.to_string(),
        }
    }
}

#[async_trait]
impl Tool for DataTransformTool {
    async fn execute(&self, input: &ToolInput) -> Result<ToolResult> {
        println!("  🔧 Tool '{}' executing {} transformation", self.name, self.transformation_type);
        
        // Simulate different transformations based on type
        let result = match self.transformation_type.as_str() {
            "uppercase" => {
                let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
                json!({"result": text.to_uppercase(), "transformation": "uppercase"})
            }
            "word_count" => {
                let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let word_count = text.split_whitespace().count();
                json!({"result": word_count, "transformation": "word_count", "original_text": text})
            }
            "reverse" => {
                let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let reversed = text.chars().rev().collect::<String>();
                json!({"result": reversed, "transformation": "reverse"})
            }
            _ => json!({"error": "unknown transformation type"})
        };
        
        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        println!("  ✅ Tool '{}' completed transformation", self.name);
        Ok(ToolResult::new(result))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &format!("Performs {} transformation on text data", self.transformation_type)
    }
    
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": {"type": "string", "description": "Text to transform"}
            },
            "required": ["text"]
        })
    }
}

/// Mock LLM client for demonstration
pub struct MockLLMClient;

#[async_trait]
impl LLMClient for MockLLMClient {
    async fn generate(&self, messages: &[Value]) -> Result<String> {
        // Simulate LLM processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let last_message = messages.last()
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("");
        
        // Generate a mock response based on the input
        if last_message.contains("sentiment") {
            Ok("The sentiment of the text is positive with confidence score 0.85".to_string())
        } else if last_message.contains("summary") {
            Ok("This text discusses workflow integration patterns with agents and tools, demonstrating composable architecture.".to_string())
        } else if last_message.contains("translate") {
            Ok("Translated text: [Mock translation result]".to_string())
        } else {
            Ok(format!("Processed: {}", last_message))
        }
    }
    
    async fn generate_structured(&self, _messages: &[Value], _schema: &Value) -> Result<Value> {
        Ok(json!({"mock": "structured response"}))
    }
}

/// Step that uses an agent directly within its execute function
pub struct AgentInvokingStep {
    pub name: String,
    pub agent: Arc<dyn Agent + Send + Sync>,
    pub prompt_template: String,
}

impl AgentInvokingStep {
    pub fn new(name: &str, agent: Arc<dyn Agent + Send + Sync>, prompt_template: &str) -> Self {
        Self {
            name: name.to_string(),
            agent,
            prompt_template: prompt_template.to_string(),
        }
    }
}

#[async_trait]
impl WorkflowStep for AgentInvokingStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        println!("  📝 Step '{}' invoking agent", self.name);
        
        // Get input from context and format prompt
        let input_text = context.metadata.get("current_text")
            .cloned()
            .unwrap_or_else(|| "default input".to_string());
        
        let formatted_prompt = self.prompt_template.replace("{input}", &input_text);
        
        // Invoke agent with formatted prompt
        let response = self.agent.generate(&formatted_prompt).await?;
        
        // Store result in context
        context.metadata.insert(
            format!("{}_response", self.name),
            response
        );
        
        println!("  ✅ Step '{}' completed agent invocation", self.name);
        Ok(WorkflowDecision::Continue)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Step that uses a tool directly within its execute function
pub struct ToolInvokingStep {
    pub name: String,
    pub tool: Arc<dyn Tool + Send + Sync>,
}

impl ToolInvokingStep {
    pub fn new(name: &str, tool: Arc<dyn Tool + Send + Sync>) -> Self {
        Self {
            name: name.to_string(),
            tool,
        }
    }
}

#[async_trait]
impl WorkflowStep for ToolInvokingStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        println!("  🔧 Step '{}' invoking tool", self.name);
        
        // Get input from context
        let input_text = context.metadata.get("current_text")
            .cloned()
            .unwrap_or_else(|| "default input".to_string());
        
        // Create tool input
        let tool_input = ToolInput::new(json!({"text": input_text}));
        
        // Invoke tool
        let result = self.tool.execute(&tool_input).await?;
        
        // Store result in context
        context.metadata.insert(
            format!("{}_result", self.name),
            result.data().to_string()
        );
        
        println!("  ✅ Step '{}' completed tool invocation", self.name);
        Ok(WorkflowDecision::Continue)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Helper function to create a workflow step from an agent
pub fn create_agent_step(name: &str, agent: Arc<dyn Agent + Send + Sync>) -> Box<dyn WorkflowStep + Send + Sync> {
    Box::new(AgentInvokingStep::new(name, agent, "{input}"))
}

/// Helper function to create a workflow step from a tool
pub fn create_tool_step(name: &str, tool: Arc<dyn Tool + Send + Sync>) -> Box<dyn WorkflowStep + Send + Sync> {
    Box::new(ToolInvokingStep::new(name, tool))
}

/// Workflow that can be converted to a tool
pub fn create_text_processing_workflow(llm_client: Arc<dyn LLMClient>) -> WorkflowBuilder {
    let uppercase_tool = Arc::new(DataTransformTool::new("uppercase_tool", "uppercase"));
    let word_count_tool = Arc::new(DataTransformTool::new("word_count_tool", "word_count"));
    let sentiment_agent = Arc::new(TextAnalysisAgent::new("sentiment_agent", llm_client.clone()));
    
    WorkflowBuilder::new("text_processing_workflow")
        .with_input_schema(
            StepSchema::new_object()
                .add_property("text", "string")
                .add_required("text")
        )
        .then(create_tool_step("transform_uppercase", uppercase_tool))
        .then(create_tool_step("count_words", word_count_tool))
        .then(create_agent_step("analyze_sentiment", sentiment_agent))
        .with_initial_data(json!({"current_text": "Hello world! This is a test."}))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Workflow-Agent-Tool Integration Demo");
    println!("=======================================\n");

    // Create shared LLM client
    let llm_client: Arc<dyn LLMClient> = Arc::new(MockLLMClient);

    // Demo 1: Using agents directly as workflow steps
    println!("📋 Demo 1: Agent as Direct Workflow Step");
    println!("----------------------------------------");
    
    let sentiment_agent = Arc::new(TextAnalysisAgent::new("sentiment_analyzer", llm_client.clone()));
    let summary_agent = Arc::new(TextAnalysisAgent::new("text_summarizer", llm_client.clone()));
    
    let agent_workflow = WorkflowBuilder::new("agent_integration_demo")
        .with_initial_data(json!({"current_text": "This is an amazing product that exceeds expectations!"}))
        .then(create_agent_step("sentiment_analysis", sentiment_agent))
        .then(create_agent_step("text_summary", summary_agent))
        .build();

    let context = WorkflowContext::new(10);
    let result = agent_workflow.execute(context).await?;
    println!("Agent workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 2: Using tools directly as workflow steps
    println!("📋 Demo 2: Tool as Direct Workflow Step");
    println!("---------------------------------------");
    
    let uppercase_tool = Arc::new(DataTransformTool::new("text_transformer", "uppercase"));
    let word_count_tool = Arc::new(DataTransformTool::new("word_counter", "word_count"));
    let reverse_tool = Arc::new(DataTransformTool::new("text_reverser", "reverse"));
    
    let tool_workflow = WorkflowBuilder::new("tool_integration_demo")
        .with_initial_data(json!({"current_text": "Hello World from Rust!"}))
        .then(create_tool_step("uppercase_transform", uppercase_tool))
        .then(create_tool_step("count_words", word_count_tool))
        .then(create_tool_step("reverse_text", reverse_tool))
        .build();

    let context = WorkflowContext::new(10);
    let result = tool_workflow.execute(context).await?;
    println!("Tool workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 3: Mixed workflow with agents and tools
    println!("📋 Demo 3: Mixed Agent-Tool Workflow");
    println!("------------------------------------");
    
    let translate_agent = Arc::new(TextAnalysisAgent::new("translator", llm_client.clone()));
    let word_count_tool_2 = Arc::new(DataTransformTool::new("counter", "word_count"));
    
    let mixed_workflow = WorkflowBuilder::new("mixed_agent_tool_demo")
        .with_initial_data(json!({"current_text": "The quick brown fox jumps over the lazy dog"}))
        .then(create_tool_step("analyze_words", word_count_tool_2))
        .then(create_agent_step("translate_text", translate_agent))
        .then(Box::new(AgentInvokingStep::new(
            "custom_analysis",
            Arc::new(TextAnalysisAgent::new("analyzer", llm_client.clone())),
            "Analyze the following text for key themes: {input}"
        )))
        .build();

    let context = WorkflowContext::new(15);
    let result = mixed_workflow.execute(context).await?;
    println!("Mixed workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 4: Workflow with input mapping for agents
    println!("📋 Demo 4: Workflow with Input Mapping");
    println!("--------------------------------------");
    
    // Mapper function to transform workflow input to agent-compatible prompt
    let prompt_mapper: MapperFn = Arc::new(|context, input_data| {
        let original_text = input_data.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("default text");
        
        // Transform to a structured prompt for sentiment analysis
        let prompt = format!(
            "Please analyze the sentiment of the following text and provide a confidence score:\n\nText: '{}'\n\nProvide your analysis in a structured format.",
            original_text
        );
        
        // Update context with the formatted prompt
        context.metadata.insert("formatted_prompt".to_string(), prompt.clone());
        
        json!({"prompt": prompt, "original": original_text})
    });
    
    let mapped_workflow = WorkflowBuilder::new("mapped_input_demo")
        .with_input_schema(
            StepSchema::new_object()
                .add_property("text", "string")
                .add_required("text")
        )
        .with_initial_data(json!({"text": "I absolutely love this new feature! It's incredible."}))
        .map(prompt_mapper)
        .then(create_agent_step("sentiment_with_mapping", 
            Arc::new(TextAnalysisAgent::new("sentiment_mapper", llm_client.clone()))
        ))
        .build();

    let context = WorkflowContext::new(10);
    let result = mapped_workflow.execute(context).await?;
    println!("Mapped workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 5: Reusable workflow as a tool
    println!("📋 Demo 5: Workflow as Reusable Tool");
    println!("------------------------------------");
    
    let text_processing_workflow = create_text_processing_workflow(llm_client.clone()).build();
    
    // This workflow can now be exposed as a tool to other workflows or agents
    println!("Text processing workflow created and ready for reuse");
    
    let context = WorkflowContext::new(15);
    let result = text_processing_workflow.execute(context).await?;
    println!("Reusable workflow completed: Steps executed = {}\n", result.steps_executed);

    // Demo 6: Complex nested workflow with multiple patterns
    println!("📋 Demo 6: Complex Nested Workflow");
    println!("----------------------------------");
    
    // Create a complex workflow that combines all patterns
    let preprocessing_tool = Arc::new(DataTransformTool::new("preprocessor", "uppercase"));
    let analysis_agent = Arc::new(TextAnalysisAgent::new("comprehensive_analyzer", llm_client.clone()));
    let postprocessing_tool = Arc::new(DataTransformTool::new("postprocessor", "word_count"));
    
    let complex_workflow = WorkflowBuilder::new("complex_nested_demo")
        .with_input_schema(
            StepSchema::new_object()
                .add_property("documents", "array")
                .add_property("analysis_type", "string")
        )
        .with_initial_data(json!({
            "documents": ["Doc 1: Great product!", "Doc 2: Needs improvement.", "Doc 3: Excellent service!"],
            "analysis_type": "sentiment_batch",
            "current_text": "Batch processing multiple documents for comprehensive analysis"
        }))
        // Preprocessing phase
        .then(create_tool_step("preprocess", preprocessing_tool))
        // Analysis phase with agents
        .then(create_agent_step("analyze", analysis_agent))
        // Post-processing phase
        .then(create_tool_step("postprocess", postprocessing_tool))
        // Custom integration step
        .then(Box::new(ToolInvokingStep::new(
            "final_transform",
            Arc::new(DataTransformTool::new("finalizer", "reverse"))
        )))
        .build();

    let context = WorkflowContext::new(20);
    let result = complex_workflow.execute(context).await?;
    println!("Complex workflow completed: Steps executed = {}", result.steps_executed);

    println!("\n🎉 Workflow-Agent-Tool Integration Demo Completed!");
    println!("\n💡 Key Integration Patterns Demonstrated:");
    println!("   • createStep(agent) - Direct agent integration as workflow step");
    println!("   • agent.generate() - Invoking agents from within step execution");
    println!("   • createStep(tool) - Direct tool integration as workflow step");
    println!("   • tool.execute() - Invoking tools from within step execution");
    println!("   • .map() - Input transformation for agent compatibility");
    println!("   • Workflow as Tool - Reusable workflow components");
    println!("   • Mixed workflows - Combining agents, tools, and custom logic");
    println!("   • Nested composition - Complex multi-phase processing");

    Ok(())
}