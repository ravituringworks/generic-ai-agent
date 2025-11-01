//! Main AI Agent implementation

use crate::a2a::{A2AManager, AgentCapabilities, AgentId, HttpA2AClient};
use crate::config::AgentConfig;
use crate::error::Result;
use crate::llm::{
    assistant_message, system_message, user_message, LlmClient, Message, OllamaClient, Role,
};
use crate::mcp::{McpClient, ToolCall};
use crate::memory::{MemoryStore, SqliteMemoryStore};
use crate::tools::BuiltinTools;
use crate::workflow::{WorkflowContext, WorkflowEngine, WorkflowResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Main AI Agent that coordinates all components
pub struct Agent {
    /// Configuration
    config: AgentConfig,

    /// LLM client for text generation and embeddings
    llm: Box<dyn LlmClient>,

    /// Memory store for persistent knowledge
    memory: Arc<RwLock<Box<dyn MemoryStore>>>,

    /// MCP client for tool calling
    mcp: Arc<RwLock<McpClient>>,

    /// A2A manager for agent-to-agent communication
    a2a: Option<A2AManager>,

    /// Built-in tools
    builtin_tools: BuiltinTools,

    /// Workflow engine
    workflow: WorkflowEngine,

    /// Conversation history
    conversation: Vec<Message>,
}

impl Agent {
    /// Create a new agent with the given configuration
    pub async fn new(config: AgentConfig) -> Result<Self> {
        info!("Initializing AI Agent: {}", config.agent.name);

        // Validate configuration
        config.validate()?;

        // Initialize LLM client
        let llm: Box<dyn LlmClient> = Box::new(OllamaClient::new(config.llm.clone()));

        // Initialize memory store
        let mut memory_store: Box<dyn MemoryStore> =
            Box::new(SqliteMemoryStore::new(config.memory.clone()));
        memory_store.initialize().await?;
        let memory = Arc::new(RwLock::new(memory_store));

        // Initialize MCP client
        let mut mcp_client = McpClient::new(config.mcp.clone());

        // Add configured MCP servers
        for (name, server_config) in &config.mcp.servers {
            if let Err(e) = mcp_client
                .add_server(name.clone(), server_config.clone())
                .await
            {
                warn!("Failed to add MCP server {}: {}", name, e);
            }
        }

        let mcp = Arc::new(RwLock::new(mcp_client));

        // Initialize built-in tools
        let builtin_tools = BuiltinTools::new();

        // Initialize A2A manager if enabled
        let a2a = if config.a2a.discovery.enabled {
            let agent_id = AgentId::new(&config.agent.name, &config.agent.name);
            let a2a_config = config.a2a.clone();

            match HttpA2AClient::new(a2a_config) {
                Ok(client) => {
                    let a2a_manager = A2AManager::new(Arc::new(client), agent_id);
                    Some(a2a_manager)
                }
                Err(e) => {
                    warn!("Failed to initialize A2A client: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize workflow engine with configuration
        let mut workflow = WorkflowEngine::default();

        // Configure SQLite snapshot storage using the same database as memory
        let database_url = config
            .memory
            .database_url
            .clone()
            .unwrap_or_else(|| "sqlite:.agency/agent.db".to_string());

        use crate::workflow::SqliteSnapshotStorage;
        let mut storage = SqliteSnapshotStorage::new(database_url);
        if let Err(e) = storage.initialize().await {
            warn!("Failed to initialize workflow snapshot storage: {}", e);
        } else {
            workflow = workflow.with_snapshot_storage(Box::new(storage));
        }

        // Apply workflow suspend configuration
        if config.workflow.enable_suspend_resume {
            use crate::workflow::WorkflowSuspendConfig;
            let suspend_config = WorkflowSuspendConfig {
                auto_checkpoint: config.workflow.auto_checkpoint,
                checkpoint_interval: config.workflow.checkpoint_interval,
                max_snapshots: config.workflow.max_snapshots,
                snapshot_retention: chrono::Duration::days(config.workflow.snapshot_retention_days),
            };
            workflow = workflow.with_suspend_config(suspend_config);
        }

        // Initialize conversation with system message
        let conversation = vec![system_message(&config.agent.system_prompt)];

        info!("AI Agent initialized successfully");

        Ok(Self {
            config,
            llm,
            memory,
            mcp,
            a2a,
            builtin_tools,
            workflow,
            conversation,
        })
    }

    /// Process a user message and return a response
    pub async fn process(&mut self, user_input: &str) -> Result<String> {
        info!(
            "Processing user input: {}",
            user_input.chars().take(100).collect::<String>()
        );

        // Add user message to conversation
        let user_msg = user_message(user_input);
        self.conversation.push(user_msg.clone());

        // Create workflow context
        let mut context = WorkflowContext::new(self.config.agent.max_thinking_steps);

        // Add conversation history to context
        for message in &self.conversation {
            context.add_message(message.clone());
        }

        // Add available tools to context
        context.available_tools = self.get_available_tools().await;

        // Execute workflow
        let mut result = self.workflow.execute(context).await?;

        // Handle pending actions
        while result.has_pending_actions() {
            if let Some(tool_calls) = result.pending_tool_calls.take() {
                result = self.handle_tool_calls(result, tool_calls).await?;
            }

            if let Some(memory_query) = result.pending_memory_query.take() {
                result = self.handle_memory_retrieval(result, memory_query).await?;
            }
        }

        // Generate final response if workflow didn't complete OR returned empty response
        if !result.completed || result.response.is_empty() {
            result = self.generate_final_response(result).await?;
        }

        // Add assistant response to conversation
        let assistant_msg = assistant_message(&result.response);
        self.conversation.push(assistant_msg);

        // Limit conversation history
        self.limit_conversation_history();

        // Store conversation in memory if enabled
        if self.config.agent.use_memory {
            self.store_conversation_memory(user_input, &result.response)
                .await?;
        }

        debug!(
            "Generated response with {} characters",
            result.response.len()
        );
        Ok(result.response)
    }

    /// Handle tool calls during workflow execution
    async fn handle_tool_calls(
        &self,
        mut result: WorkflowResult,
        tool_calls: Vec<ToolCall>,
    ) -> Result<WorkflowResult> {
        debug!("Handling {} tool calls", tool_calls.len());

        for tool_call in tool_calls {
            // Try built-in tools first
            if let Some(tool_result) = self.builtin_tools.execute(&tool_call.name).await {
                result
                    .context
                    .add_tool_result(tool_call.id.clone(), tool_result);
                continue;
            }

            // Try MCP tools
            let mcp = self.mcp.read().await;
            match mcp.call_tool(tool_call.clone()).await {
                Ok(tool_result) => {
                    result.context.add_tool_result(tool_call.id, tool_result);
                }
                Err(e) => {
                    warn!("Tool call failed: {}", e);
                    // Continue with other tools
                }
            }
        }

        // Continue workflow with tool results
        // Don't reset step count to avoid infinite loops
        self.workflow.execute(result.context).await
    }

    /// Handle memory retrieval during workflow execution
    async fn handle_memory_retrieval(
        &self,
        mut result: WorkflowResult,
        query: String,
    ) -> Result<WorkflowResult> {
        debug!("Handling memory retrieval for query: {}", query);

        if self.config.agent.use_memory {
            // Generate embedding for the query
            let embedding_response = self.llm.embed(&query).await?;

            // Search memory
            let memory = self.memory.read().await;
            let search_results = memory
                .search(
                    embedding_response.embedding,
                    self.config.memory.max_search_results,
                    self.config.memory.similarity_threshold,
                )
                .await?;

            result.context.memories = search_results;
            result
                .context
                .metadata
                .insert("memories_retrieved".to_string(), "true".to_string());
            debug!(
                "Retrieved {} relevant memories",
                result.context.memories.len()
            );
        }

        // Continue workflow with memory results
        // Don't reset step count to avoid infinite loops
        self.workflow.execute(result.context).await
    }

    /// Generate final response using LLM
    async fn generate_final_response(&self, mut result: WorkflowResult) -> Result<WorkflowResult> {
        debug!("Generating final LLM response");

        // Build context for LLM
        let mut messages = result.context.messages.clone();

        // Add tool results as messages
        if !result.context.tool_results.is_empty() {
            let mut tool_summary = String::new();
            tool_summary.push_str("Tool results:\n");

            for tool_result in result.context.tool_results.values() {
                for content in &tool_result.content {
                    if let crate::mcp::ToolContent::Text { text } = content {
                        tool_summary.push_str(&format!("- {}\n", text));
                    }
                }
            }

            messages.push(assistant_message(tool_summary));
        }

        // Add memory context
        if !result.context.memories.is_empty() && self.config.agent.use_memory {
            let mut memory_summary = String::new();
            memory_summary.push_str("Relevant memories:\n");

            for memory in &result.context.memories {
                memory_summary.push_str(&format!("- {}\n", memory.entry.content));
            }

            messages.push(assistant_message(memory_summary));
        }

        // Generate response
        let generation_result = self.llm.generate(&messages).await?;

        result.response = generation_result.text;
        result.completed = true;

        Ok(result)
    }

    /// Get list of all available tools
    pub async fn get_available_tools(&self) -> Vec<String> {
        let mut tools = Vec::new();

        // Add built-in tools
        tools.extend(self.builtin_tools.list_tools());

        // Add MCP tools
        let mcp = self.mcp.read().await;
        let mcp_tools = mcp.list_tools();
        for (_, tool) in mcp_tools {
            tools.push(tool.name.clone());
        }

        tools
    }

    /// Store conversation in memory
    async fn store_conversation_memory(&self, user_input: &str, response: &str) -> Result<()> {
        // Skip if memory is disabled
        if !self.config.agent.use_memory {
            debug!("Memory disabled, skipping conversation storage");
            return Ok(());
        }

        debug!("Storing conversation in memory");

        let conversation_text = format!("User: {}\nAssistant: {}", user_input, response);

        // Generate embedding
        let embedding_response = self.llm.embed(&conversation_text).await?;

        // Store in memory
        let mut memory = self.memory.write().await;
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "conversation".to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        memory
            .store(conversation_text, embedding_response.embedding, metadata)
            .await?;

        debug!("Conversation stored in memory");
        Ok(())
    }

    /// Limit conversation history to configured maximum
    fn limit_conversation_history(&mut self) {
        if self.conversation.len() > self.config.agent.max_history_length {
            let keep_from = self.conversation.len() - self.config.agent.max_history_length;

            // Always keep the system message if it exists
            let mut new_conversation = Vec::new();
            if let Some(first) = self.conversation.first() {
                if matches!(first.role, Role::System) {
                    new_conversation.push(first.clone());
                    new_conversation.extend_from_slice(&self.conversation[keep_from..]);
                } else {
                    new_conversation.extend_from_slice(&self.conversation[keep_from..]);
                }
            }

            self.conversation = new_conversation;
            debug!(
                "Limited conversation history to {} messages",
                self.conversation.len()
            );
        }
    }

    /// Get agent statistics
    pub async fn stats(&self) -> AgentStats {
        let memory = self.memory.read().await;
        let memory_stats = memory.stats().await.unwrap_or(crate::memory::MemoryStats {
            total_memories: 0,
            embedding_dimension: self.config.memory.embedding_dimension,
            store_size_bytes: None,
        });

        let mcp = self.mcp.read().await;
        let mcp_stats = mcp.stats();

        AgentStats {
            conversation_length: self.conversation.len(),
            memory_stats,
            mcp_stats,
            builtin_tools_count: self.builtin_tools.list_tools().len(),
        }
    }

    /// Clear conversation history
    pub fn clear_conversation(&mut self) {
        // Keep system message if it exists
        if let Some(first) = self.conversation.first() {
            if matches!(first.role, Role::System) {
                self.conversation = vec![first.clone()];
            } else {
                self.conversation.clear();
            }
        } else {
            self.conversation.clear();
        }

        info!("Cleared conversation history");
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.conversation.push(message);
        self.limit_conversation_history();
    }

    /// Get current conversation
    pub fn get_conversation(&self) -> &[Message] {
        &self.conversation
    }

    /// Start A2A communication (register agent and begin listening)
    pub async fn start_a2a(&self) -> Result<()> {
        if let Some(a2a) = &self.a2a {
            let _capabilities = AgentCapabilities {
                services: vec![
                    "chat".to_string(),
                    "llm".to_string(),
                    "memory".to_string(),
                    "tools".to_string(),
                ],
                protocols: vec!["http".to_string()],
                message_types: vec!["text".to_string(), "task".to_string(), "query".to_string()],
                metadata: HashMap::from([
                    ("model".to_string(), self.config.llm.text_model.clone()),
                    ("version".to_string(), crate::VERSION.to_string()),
                ]),
            };

            a2a.start().await?;
            info!(
                "A2A communication started for agent: {}",
                self.config.agent.name
            );
        }
        Ok(())
    }

    /// Stop A2A communication
    pub async fn stop_a2a(&self) -> Result<()> {
        if let Some(a2a) = &self.a2a {
            a2a.stop().await?;
            info!(
                "A2A communication stopped for agent: {}",
                self.config.agent.name
            );
        }
        Ok(())
    }

    /// Send a message to another agent via A2A
    pub async fn send_to_agent(&self, target_agent: AgentId, message: &str) -> Result<String> {
        if let Some(a2a) = &self.a2a {
            use crate::a2a::MessagePayload;

            let payload = MessagePayload::Text {
                content: message.to_string(),
            };

            let response = a2a.send_request(target_agent, "chat", payload).await?;

            match response.payload {
                Some(MessagePayload::Text { content }) => Ok(content),
                Some(MessagePayload::Json { data }) => Ok(data.to_string()),
                _ => Ok("No response content".to_string()),
            }
        } else {
            Err(crate::error::AgentError::Config(
                "A2A not enabled".to_string(),
            ))
        }
    }

    /// Discover other agents with specific capabilities
    pub async fn discover_agents(
        &self,
        capability: &str,
    ) -> Result<Vec<crate::a2a::AgentRegistration>> {
        if let Some(a2a) = &self.a2a {
            a2a.discover_service(capability).await
        } else {
            Err(crate::error::AgentError::Config(
                "A2A not enabled".to_string(),
            ))
        }
    }

    /// Process a task requested by another agent
    pub async fn process_agent_task(&mut self, task_description: &str) -> Result<String> {
        info!("Processing task from another agent: {}", task_description);

        // Use the existing process method to handle the task
        let response = self.process(task_description).await?;

        Ok(response)
    }

    /// Check if A2A communication is enabled
    pub fn has_a2a(&self) -> bool {
        self.a2a.is_some()
    }

    /// Get agent configuration (read-only access)
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
}

/// Agent statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentStats {
    pub conversation_length: usize,
    pub memory_stats: crate::memory::MemoryStats,
    pub mcp_stats: crate::mcp::McpStats,
    pub builtin_tools_count: usize,
}

/// Builder pattern for creating an Agent
pub struct AgentBuilder {
    config: AgentConfig,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            config: AgentConfig::default(),
        }
    }

    pub fn with_config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.config.agent.name = name;
        self
    }

    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.config.agent.system_prompt = prompt;
        self
    }

    pub fn with_ollama_url(mut self, url: String) -> Self {
        self.config.llm.ollama_url = url;
        self
    }

    pub async fn build(self) -> Result<Agent> {
        Agent::new(self.config).await
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_agent() -> Agent {
        // Use in-memory SQLite database for tests
        let mut config = AgentConfig::default();
        config.memory.database_url = Some("sqlite::memory:".to_string());

        Agent::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_agent_creation() {
        let agent = create_test_agent().await;
        assert!(!agent.config.agent.name.is_empty());
        assert_eq!(agent.conversation.len(), 1); // System message
    }

    #[tokio::test]
    async fn test_agent_builder() {
        let agent = AgentBuilder::new()
            .with_name("Test Agent".to_string())
            .with_system_prompt("You are a test assistant.".to_string())
            .build()
            .await;

        // This will fail if Ollama is not running, which is expected in CI
        // In practice, you'd use dependency injection or mock the LLM client
        match agent {
            Ok(agent) => {
                assert_eq!(agent.config.agent.name, "Test Agent");
            }
            Err(_) => {
                // Expected if Ollama is not available
            }
        }
    }

    #[tokio::test]
    async fn test_conversation_management() {
        let mut agent = create_test_agent().await;

        // Test message adding
        agent.add_message(user_message("Hello"));
        assert_eq!(agent.conversation.len(), 2); // System + user message

        // Test conversation clearing
        agent.clear_conversation();
        assert_eq!(agent.conversation.len(), 1); // Only system message remains
    }

    #[tokio::test]
    async fn test_available_tools() {
        let agent = create_test_agent().await;
        let tools = agent.get_available_tools().await;

        // Should have at least built-in tools
        assert!(!tools.is_empty());
        assert!(tools.contains(&"system_info".to_string()));
    }
}
