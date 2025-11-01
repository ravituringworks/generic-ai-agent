//! Unit tests for the Generic AI Agent

use std::collections::HashMap;
use tempfile::tempdir;
use the_agency::*;

/// Test configuration validation
#[tokio::test]
async fn test_config_validation() {
    let mut config = AgentConfig::default();

    // Valid config should pass
    assert!(config.validate().is_ok());

    // Invalid Ollama URL should fail
    config.llm.ollama_url = "invalid-url".to_string();
    assert!(config.validate().is_err());

    // Reset and test empty model
    config = AgentConfig::default();
    config.llm.text_model = "".to_string();
    assert!(config.validate().is_err());

    // Reset and test invalid similarity threshold
    config = AgentConfig::default();
    config.memory.similarity_threshold = 2.0; // Invalid: should be between 0.0 and 1.0
    assert!(config.validate().is_err());
}

/// Test agent builder pattern
#[tokio::test]
async fn test_agent_builder() {
    let _builder = AgentBuilder::new()
        .with_name("Test Agent".to_string())
        .with_system_prompt("You are a test assistant.".to_string())
        .with_ollama_url("http://test:11434".to_string());

    // The actual build will fail without Ollama
    // Builder pattern is tested implicitly through successful compilation
}

/// Test memory store functionality
#[tokio::test]
async fn test_memory_store() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let config = MemoryConfig {
        database_url: Some(format!("sqlite://{}", db_path.to_str().unwrap())),
        embedding_dimension: 384,
        max_search_results: 10,
        similarity_threshold: 0.7,
        persistent: true,
        store_type: "sqlite".to_string(),
    };

    let mut store = memory::SqliteMemoryStore::new(config);
    store.initialize().await.unwrap();

    // Test storing a memory
    let embedding = vec![0.1; 384];
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "test".to_string());

    let id = store
        .store(
            "This is a test memory".to_string(),
            embedding.clone(),
            metadata.clone(),
        )
        .await
        .unwrap();

    // Test retrieving the memory
    let retrieved = store.get(id).await.unwrap().unwrap();
    assert_eq!(retrieved.content, "This is a test memory");
    assert_eq!(retrieved.embedding.len(), 384);
    assert_eq!(retrieved.metadata.get("source").unwrap(), "test");

    // Test searching memories
    let results = store.search(embedding, 10, 0.5).await.unwrap();
    assert!(!results.is_empty());
    assert!(results[0].similarity > 0.9); // Should be very similar to itself

    // Test updating the memory
    let mut new_metadata = HashMap::new();
    new_metadata.insert("updated".to_string(), "true".to_string());

    store
        .update(
            id,
            Some("Updated content".to_string()),
            None,
            Some(new_metadata),
        )
        .await
        .unwrap();

    let updated = store.get(id).await.unwrap().unwrap();
    assert_eq!(updated.content, "Updated content");

    // Test statistics
    let stats = store.stats().await.unwrap();
    assert_eq!(stats.total_memories, 1);
    assert_eq!(stats.embedding_dimension, 384);

    // Test deleting the memory
    store.delete(id).await.unwrap();
    let deleted = store.get(id).await.unwrap();
    assert!(deleted.is_none());
}

/// Test vector similarity calculations
#[test]
fn test_cosine_similarity() {
    // Test identical vectors
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let similarity = memory::SqliteMemoryStore::cosine_similarity(&a, &b);
    assert_eq!(similarity, 1.0);

    // Test orthogonal vectors
    let a = vec![1.0, 0.0, 0.0];
    let c = vec![0.0, 1.0, 0.0];
    let similarity = memory::SqliteMemoryStore::cosine_similarity(&a, &c);
    assert_eq!(similarity, 0.0);

    // Test zero vectors
    let zero = vec![0.0, 0.0, 0.0];
    let similarity = memory::SqliteMemoryStore::cosine_similarity(&a, &zero);
    assert_eq!(similarity, 0.0);
}

/// Test embedding serialization/deserialization
#[test]
fn test_embedding_serialization() {
    let embedding = vec![1.5, -2.3, 0.0, 42.1];
    let serialized = memory::SqliteMemoryStore::serialize_embedding(&embedding);
    let deserialized = memory::SqliteMemoryStore::deserialize_embedding(&serialized);

    assert_eq!(embedding.len(), deserialized.len());
    for (orig, deser) in embedding.iter().zip(deserialized.iter()) {
        assert!((orig - deser).abs() < f32::EPSILON);
    }
}

/// Test LLM message creation helpers
#[test]
fn test_llm_messages() {
    let system_msg = llm::system_message("You are a helpful assistant");
    assert_eq!(system_msg.role, llm::Role::System);
    assert_eq!(system_msg.content, "You are a helpful assistant");

    let user_msg = llm::user_message("Hello");
    assert_eq!(user_msg.role, llm::Role::User);
    assert_eq!(user_msg.content, "Hello");

    let assistant_msg = llm::assistant_message("Hi there!");
    assert_eq!(assistant_msg.role, llm::Role::Assistant);
    assert_eq!(assistant_msg.content, "Hi there!");
}

/// Test MCP tool serialization
#[test]
fn test_mcp_tool_serialization() {
    let tool_call = mcp::ToolCall {
        id: "test-id".to_string(),
        name: "test-tool".to_string(),
        arguments: serde_json::json!({"param": "value"}),
    };

    assert_eq!(tool_call.name, "test-tool");
    assert_eq!(tool_call.arguments["param"], "value");

    // Test tool content
    let text_content = mcp::ToolContent::Text {
        text: "Hello, world!".to_string(),
    };

    let json = serde_json::to_value(&text_content).unwrap();
    assert_eq!(json["type"], "text");
    assert_eq!(json["text"], "Hello, world!");

    let deserialized: mcp::ToolContent = serde_json::from_value(json).unwrap();
    if let mcp::ToolContent::Text { text } = deserialized {
        assert_eq!(text, "Hello, world!");
    } else {
        panic!("Expected text content");
    }
}

/// Test workflow context management
#[tokio::test]
async fn test_workflow_context() {
    let mut context = workflow::WorkflowContext::new(5);

    assert_eq!(context.step_count, 0);
    assert!(context.should_continue());

    context.increment_step();
    assert_eq!(context.step_count, 1);

    context.add_message(llm::user_message("Hello"));
    assert_eq!(context.messages.len(), 1);

    // Test tool result addition
    let tool_result = mcp::ToolResult {
        id: "test-id".to_string(),
        content: vec![mcp::ToolContent::Text {
            text: "Result".to_string(),
        }],
        is_error: false,
    };

    context.add_tool_result("call-1".to_string(), tool_result);
    assert!(context.tool_results.contains_key("call-1"));
}

/// Test workflow steps
#[tokio::test]
async fn test_workflow_steps() {
    // Test memory retrieval step
    let step = workflow::MemoryRetrievalStep;
    let mut context = workflow::WorkflowContext::new(5);

    // No user message
    let decision = step.execute(&mut context).await.unwrap();
    assert!(matches!(decision, workflow::WorkflowDecision::Continue));

    // With user message
    context.add_message(llm::user_message("What is Rust?"));
    let decision = step.execute(&mut context).await.unwrap();
    assert!(matches!(
        decision,
        workflow::WorkflowDecision::RetrieveMemories(_)
    ));

    // Test tool analysis step
    let step = workflow::ToolAnalysisStep;
    let mut context = workflow::WorkflowContext::new(5);
    context.available_tools.push("system_info".to_string());

    // No relevant message
    context.add_message(llm::user_message("Hello"));
    let decision = step.execute(&mut context).await.unwrap();
    assert!(matches!(decision, workflow::WorkflowDecision::Continue));

    // System info request
    context.add_message(llm::user_message("Show me system info"));
    let decision = step.execute(&mut context).await.unwrap();
    assert!(matches!(
        decision,
        workflow::WorkflowDecision::ExecuteTools(_)
    ));
}

/// Test workflow engine execution
#[tokio::test]
async fn test_workflow_engine() {
    let engine = workflow::WorkflowEngine::default();
    let mut context = workflow::WorkflowContext::new(10);
    context.add_message(llm::user_message("Hello, how are you?"));

    let result = engine.execute(context).await.unwrap();
    assert!(result.completed);
    assert!(!result.response.is_empty());
}

/// Test built-in tools
#[tokio::test]
async fn test_builtin_tools() {
    let tools = tools::BuiltinTools::new();

    let tool_list = tools.list_tools();
    assert!(!tool_list.is_empty());
    assert!(tool_list.contains(&"system_info".to_string()));

    let result = tools.execute("system_info").await;
    assert!(result.is_some());

    let result = result.unwrap();
    assert!(!result.is_error);
    assert!(!result.content.is_empty());
}

/// Test error types and categorization
#[test]
fn test_error_types() {
    let timeout_error = error::AgentError::Llm(error::LlmError::Timeout);
    assert!(timeout_error.is_retryable());
    assert_eq!(timeout_error.category(), "llm");

    let config_error = error::AgentError::Config("invalid config".to_string());
    assert!(!config_error.is_retryable());
    assert_eq!(config_error.category(), "config");

    let memory_error = error::AgentError::Memory(error::MemoryError::NotInitialized);
    assert!(!memory_error.is_retryable());
    assert_eq!(memory_error.category(), "memory");
}

/// Test configuration file operations (when possible)
#[test]
fn test_config_file_operations() {
    let config = AgentConfig::default();

    // Test JSON serialization/deserialization
    let json_str = serde_json::to_string_pretty(&config).unwrap();
    let parsed_config: AgentConfig = serde_json::from_str(&json_str).unwrap();
    assert_eq!(config.agent.name, parsed_config.agent.name);

    // Test MCP server management
    let mut config = AgentConfig::default();
    let server_config = config::McpServerConfig {
        transport: "http".to_string(),
        url: Some("http://localhost:8000".to_string()),
        command: None,
        env: None,
        timeout: Some(30),
        auth_token: None,
        enabled: true,
    };

    config.add_mcp_server("test-server".to_string(), server_config.clone());
    assert!(config.mcp.servers.contains_key("test-server"));

    let removed = config.remove_mcp_server("test-server");
    assert!(removed.is_some());
    assert!(!config.mcp.servers.contains_key("test-server"));
}

/// Test concurrent operations
#[tokio::test]
async fn test_concurrent_operations() {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("concurrent_test.db");

    let config = MemoryConfig {
        database_url: Some(format!("sqlite://{}", db_path.to_str().unwrap())),
        embedding_dimension: 384,
        max_search_results: 10,
        similarity_threshold: 0.7,
        persistent: true,
        store_type: "sqlite".to_string(),
    };

    let mut store = memory::SqliteMemoryStore::new(config);
    store.initialize().await.unwrap();
    let store = Arc::new(RwLock::new(Box::new(store) as Box<dyn memory::MemoryStore>));

    // Test concurrent reads and writes
    let mut handles = vec![];

    for i in 0..5 {
        let store_clone = Arc::clone(&store);
        let handle = tokio::spawn(async move {
            let embedding = vec![0.1 * i as f32; 384];
            let mut metadata = HashMap::new();
            metadata.insert("thread".to_string(), i.to_string());

            // Write operation
            let mut store_lock = store_clone.write().await;
            let id = store_lock
                .store(
                    format!("Memory from thread {}", i),
                    embedding.clone(),
                    metadata,
                )
                .await
                .unwrap();

            drop(store_lock); // Release write lock

            // Read operation
            let store_lock = store_clone.read().await;
            let retrieved = store_lock.get(id).await.unwrap();
            assert!(retrieved.is_some());

            id
        });
        handles.push(handle);
    }

    let ids: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    assert_eq!(ids.len(), 5);

    // Verify all memories were stored
    let store_lock = store.read().await;
    let stats = store_lock.stats().await.unwrap();
    assert_eq!(stats.total_memories, 5);
}

/// Integration test with mocked components
#[tokio::test]
async fn test_integration_with_mocks() {
    // This test would require more sophisticated mocking setup
    // For now, we test component integration at a basic level

    let config = AgentConfig::default();
    assert!(config.validate().is_ok());

    let workflow_engine = workflow::WorkflowEngine::default();
    let context = workflow::WorkflowContext::new(5);

    // This should not panic and should complete successfully
    let result = workflow_engine.execute(context).await;
    assert!(result.is_ok());
}
