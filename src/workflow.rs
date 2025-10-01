//! Workflow engine for orchestrating agent behavior

use crate::error::{AgentError, Result};
use crate::llm::{Message, Role};
use crate::mcp::{ToolCall, ToolResult};
use crate::memory::SearchResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Context passed between workflow steps
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    /// Current conversation messages
    pub messages: Vec<Message>,
    
    /// Retrieved memories
    pub memories: Vec<SearchResult>,
    
    /// Available tools
    pub available_tools: Vec<String>,
    
    /// Tool call results
    pub tool_results: HashMap<String, ToolResult>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    
    /// Step counter
    pub step_count: usize,
    
    /// Maximum steps allowed
    pub max_steps: usize,
}

impl WorkflowContext {
    pub fn new(max_steps: usize) -> Self {
        Self {
            messages: Vec::new(),
            memories: Vec::new(),
            available_tools: Vec::new(),
            tool_results: HashMap::new(),
            metadata: HashMap::new(),
            step_count: 0,
            max_steps,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn add_tool_result(&mut self, tool_call_id: String, result: ToolResult) {
        self.tool_results.insert(tool_call_id, result);
    }

    pub fn should_continue(&self) -> bool {
        self.step_count < self.max_steps
    }

    pub fn increment_step(&mut self) {
        self.step_count += 1;
    }
}

/// A single step in the workflow
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    /// Execute this step
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision>;
    
    /// Get step name for debugging
    fn name(&self) -> &str;
}

/// Decision made by a workflow step
#[derive(Debug, Clone)]
pub enum WorkflowDecision {
    /// Continue to next step
    Continue,
    
    /// Complete the workflow with final response
    Complete(String),
    
    /// Jump to a specific step
    Jump(String),
    
    /// Execute tool calls and continue
    ExecuteTools(Vec<ToolCall>),
    
    /// Retrieve memories and continue
    RetrieveMemories(String),
}

/// Step that retrieves relevant memories
pub struct MemoryRetrievalStep;

#[async_trait]
impl WorkflowStep for MemoryRetrievalStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing memory retrieval step");
        
        // Only request memory retrieval once per user input
        let already_retrieved = context
            .metadata
            .get("memories_retrieved")
            .map(|v| v == "true")
            .unwrap_or(false);
        
        if let Some(last_message) = context.messages.last() {
            if matches!(last_message.role, Role::User) && !already_retrieved {
                let content = last_message.content.to_lowercase();
                
                // Check if this is a query about past conversations or memory-related
                let is_memory_query = content.contains("earlier") || 
                                    content.contains("before") ||
                                    content.contains("previous") ||
                                    content.contains("remember") ||
                                    content.contains("talked about") ||
                                    content.contains("discussed") ||
                                    content.contains("said") ||
                                    content.contains("conversation") ||
                                    content.contains("what do i") ||
                                    content.contains("what did i") ||
                                    content.contains("do i like") ||
                                    content.contains("did i tell") ||
                                    content.contains("did i mention") ||
                                    (content.contains("what") && (content.contains("like") || content.contains("prefer"))) ||
                                    (content.starts_with("do i") || content.starts_with("did i"));
                
                if is_memory_query {
                    return Ok(WorkflowDecision::RetrieveMemories(last_message.content.clone()));
                }
            }
        }
        
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "memory_retrieval"
    }
}

/// Step that analyzes available tools and decides if any should be called
pub struct ToolAnalysisStep;

#[async_trait]
impl WorkflowStep for ToolAnalysisStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing tool analysis step");
        
        // Avoid re-calling tools if we already have tool results
        if !context.tool_results.is_empty() {
            return Ok(WorkflowDecision::Continue);
        }
        
        // Simple heuristic: if the user asks for system info, call that tool
        if let Some(last_message) = context.messages.last() {
            if matches!(last_message.role, Role::User) {
                let content = last_message.content.to_lowercase();
                
                if content.contains("system") && content.contains("info") {
                    if context.available_tools.contains(&"system_info".to_string()) {
                        let tool_call = ToolCall {
                            id: Uuid::new_v4().to_string(),
                            name: "system_info".to_string(),
                            arguments: serde_json::json!({}),
                        };
                        
                        return Ok(WorkflowDecision::ExecuteTools(vec![tool_call]));
                    }
                }
            }
        }
        
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        "tool_analysis"
    }
}

/// Step that generates the final response
pub struct ResponseGenerationStep;

#[async_trait]
impl WorkflowStep for ResponseGenerationStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        debug!("Executing response generation step");
        
        // This would normally call the LLM to generate a response
        // For now, we'll create a simple response based on context
        
        let mut response_parts = Vec::new();
        
        // Include tool results if any
        if !context.tool_results.is_empty() {
            response_parts.push("Based on the tools I called:".to_string());
            for (_, result) in &context.tool_results {
                for content in &result.content {
                    match content {
                        crate::mcp::ToolContent::Text { text } => {
                            response_parts.push(text.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Include memory context if any
        if !context.memories.is_empty() {
            response_parts.push(format!("Based on our previous conversations, I found {} relevant memories:", context.memories.len()));
            for (i, memory) in context.memories.iter().enumerate().take(3) { // Show top 3
                response_parts.push(format!("{}. {}", i + 1, memory.entry.content));
            }
        }
        
        // If we have specific content (tool results or memories), provide structured response
        if !response_parts.is_empty() {
            let final_response = response_parts.join("\n\n");
            Ok(WorkflowDecision::Complete(final_response))
        } else {
            // For general queries, we want the LLM to generate the response
            // Return an empty response to signal that LLM generation is needed
            Ok(WorkflowDecision::Complete(String::new()))
        }
    }

    fn name(&self) -> &str {
        "response_generation"
    }
}

/// Workflow engine that orchestrates the execution of steps
pub struct WorkflowEngine {
    steps: Vec<Box<dyn WorkflowStep>>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
        }
    }

    pub fn add_step(mut self, step: Box<dyn WorkflowStep>) -> Self {
        self.steps.push(step);
        self
    }

    pub fn with_default_steps(self) -> Self {
        self.add_step(Box::new(MemoryRetrievalStep))
            .add_step(Box::new(ToolAnalysisStep))
            .add_step(Box::new(ResponseGenerationStep))
    }

    /// Execute the workflow
    pub async fn execute(&self, mut context: WorkflowContext) -> Result<WorkflowResult> {
        info!("Starting workflow execution with {} steps", self.steps.len());

        while context.should_continue() {
            context.increment_step();
            
            // Execute steps in order
            for step in &self.steps {
                debug!("Executing step: {}", step.name());
                
                match step.execute(&mut context).await? {
                    WorkflowDecision::Continue => {
                        // Continue to next step
                        continue;
                    }
                    WorkflowDecision::Complete(response) => {
                        let step_count = context.step_count;
                        info!("Workflow completed after {} steps", step_count);
                        return Ok(WorkflowResult {
                            response,
                            context,
                            completed: true,
                            steps_executed: step_count,
                            pending_tool_calls: None,
                            pending_memory_query: None,
                        });
                    }
                    WorkflowDecision::Jump(step_name) => {
                        // This would require more complex step management
                        debug!("Jump to step requested: {}", step_name);
                        return Err(AgentError::Workflow("Step jumping not implemented".to_string()).into());
                    }
                    WorkflowDecision::ExecuteTools(tool_calls) => {
                        debug!("Tool execution requested: {} tools", tool_calls.len());
                        let step_count = context.step_count;
                        return Ok(WorkflowResult {
                            response: String::new(),
                            context,
                            completed: false,
                            steps_executed: step_count,
                            pending_tool_calls: None,
                            pending_memory_query: None,
                        }.with_tool_calls(tool_calls));
                    }
                    WorkflowDecision::RetrieveMemories(query) => {
                        debug!("Memory retrieval requested for: {}", query);
                        let step_count = context.step_count;
                        return Ok(WorkflowResult {
                            response: String::new(),
                            context,
                            completed: false,
                            steps_executed: step_count,
                            pending_tool_calls: None,
                            pending_memory_query: None,
                        }.with_memory_query(query));
                    }
                }
            }
            
            // If we get here, all steps were executed but no completion decision was made
            break;
        }

        // Fallback: generate a default response
        info!("Workflow reached maximum steps, generating fallback response");
        let step_count = context.step_count;
        Ok(WorkflowResult {
            response: "I've processed your request but reached the maximum number of thinking steps.".to_string(),
            context,
            completed: true,
            steps_executed: step_count,
            pending_tool_calls: None,
            pending_memory_query: None,
        })
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new().with_default_steps()
    }
}

/// Result of workflow execution
#[derive(Debug)]
pub struct WorkflowResult {
    /// Generated response (if completed)
    pub response: String,
    
    /// Final workflow context
    pub context: WorkflowContext,
    
    /// Whether the workflow completed successfully
    pub completed: bool,
    
    /// Number of steps executed
    pub steps_executed: usize,
    
    /// Tool calls to execute (if any)
    pub pending_tool_calls: Option<Vec<ToolCall>>,
    
    /// Memory query to execute (if any)
    pub pending_memory_query: Option<String>,
}

impl WorkflowResult {
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.pending_tool_calls = Some(tool_calls);
        self
    }

    pub fn with_memory_query(mut self, query: String) -> Self {
        self.pending_memory_query = Some(query);
        self
    }

    pub fn has_pending_actions(&self) -> bool {
        self.pending_tool_calls.is_some() || self.pending_memory_query.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{user_message, assistant_message};

    #[tokio::test]
    async fn test_workflow_context() {
        let mut context = WorkflowContext::new(5);
        
        assert_eq!(context.step_count, 0);
        assert!(context.should_continue());
        
        context.increment_step();
        assert_eq!(context.step_count, 1);
        
        context.add_message(user_message("Hello"));
        assert_eq!(context.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_retrieval_step() {
        let step = MemoryRetrievalStep;
        let mut context = WorkflowContext::new(5);
        
        // No user message
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));
        
        // With user message
        context.add_message(user_message("What is Rust?"));
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::RetrieveMemories(_)));
    }

    #[tokio::test]
    async fn test_tool_analysis_step() {
        let step = ToolAnalysisStep;
        let mut context = WorkflowContext::new(5);
        context.available_tools.push("system_info".to_string());
        
        // No relevant message
        context.add_message(user_message("Hello"));
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::Continue));
        
        // System info request
        context.add_message(user_message("Show me system info"));
        let decision = step.execute(&mut context).await.unwrap();
        assert!(matches!(decision, WorkflowDecision::ExecuteTools(_)));
    }

    #[tokio::test]
    async fn test_workflow_engine() {
        let engine = WorkflowEngine::default();
        let mut context = WorkflowContext::new(10);
        context.add_message(user_message("Hello, how are you?"));
        
        let result = engine.execute(context).await.unwrap();
        assert!(result.completed);
        assert!(!result.response.is_empty());
    }
}