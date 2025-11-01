//! Tests for LLM module

use std::collections::HashMap;
use the_agency::{
    config::LlmConfig,
    llm::{LlmClient, Message, OllamaClient, Role},
};

/// Create a test LLM config
fn test_llm_config() -> LlmConfig {
    LlmConfig {
        ollama_url: "http://localhost:11434".to_string(),
        text_model: "llama3.2".to_string(),
        embedding_model: "nomic-embed-text".to_string(),
        max_tokens: 1000,
        temperature: 0.7,
        timeout: 30,
        stream: false,
        task_models: HashMap::new(),
        cache: the_agency::cache::LlmCacheConfig::default(),
    }
}

#[test]
fn test_ollama_client_creation() {
    let config = test_llm_config();
    let _client = OllamaClient::new(config);
    // Client creation should not panic
}

#[test]
fn test_message_serialization() {
    let message = Message {
        role: Role::User,
        content: "Hello, world!".to_string(),
    };

    let json = serde_json::to_string(&message).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();

    assert_eq!(message.role, deserialized.role);
    assert_eq!(message.content, deserialized.content);
}

#[test]
fn test_role_serialization() {
    let roles = vec![Role::System, Role::User, Role::Assistant];

    for role in roles {
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(role, deserialized);
    }
}

#[test]
fn test_role_lowercase_serialization() {
    // Test that roles serialize to lowercase as expected by Ollama API
    assert_eq!(serde_json::to_string(&Role::System).unwrap(), "\"system\"");
    assert_eq!(serde_json::to_string(&Role::User).unwrap(), "\"user\"");
    assert_eq!(
        serde_json::to_string(&Role::Assistant).unwrap(),
        "\"assistant\""
    );
}

#[tokio::test]
async fn test_ollama_client_list_models_integration() {
    // This test requires Ollama to be running
    let config = test_llm_config();
    let client = OllamaClient::new(config);

    match client.list_models().await {
        Ok(models) => {
            println!("Available models: {:?}", models);
            // If Ollama is running, we should get a list (possibly empty)
        }
        Err(e) => {
            // If Ollama is not running, that's okay for CI
            println!("Ollama not available: {}", e);
        }
    }
}

#[tokio::test]
async fn test_ollama_client_model_availability() {
    let config = test_llm_config();
    let client = OllamaClient::new(config);

    match client.is_model_available("llama3.2").await {
        Ok(available) => {
            println!("Model llama3.2 available: {}", available);
        }
        Err(e) => {
            println!("Could not check model availability: {}", e);
        }
    }
}

#[tokio::test]
async fn test_ollama_client_generation_mock() {
    // This is a basic test that just checks client creation and message formatting
    let config = test_llm_config();
    let _client = OllamaClient::new(config);

    let messages = vec![
        Message {
            role: Role::System,
            content: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: Role::User,
            content: "Hello!".to_string(),
        },
    ];

    // Verify messages can be serialized (this is what would be sent to Ollama)
    let serialized = serde_json::to_string(&messages).unwrap();
    assert!(serialized.contains("system"));
    assert!(serialized.contains("user"));
    assert!(serialized.contains("Hello!"));
}

#[test]
fn test_generation_response_structure() {
    use the_agency::llm::GenerationResponse;

    let response = GenerationResponse {
        text: "Test response".to_string(),
        tokens_used: Some(42),
        model: "llama3.2".to_string(),
        finish_reason: Some("stop".to_string()),
    };

    assert_eq!(response.text, "Test response");
    assert_eq!(response.tokens_used, Some(42));
    assert_eq!(response.model, "llama3.2");
    assert_eq!(response.finish_reason, Some("stop".to_string()));
}

#[test]
fn test_embedding_response_structure() {
    use the_agency::llm::EmbeddingResponse;

    let embedding = vec![0.1, 0.2, 0.3];
    let response = EmbeddingResponse {
        embedding: embedding.clone(),
        model: "nomic-embed-text".to_string(),
    };

    assert_eq!(response.embedding, embedding);
    assert_eq!(response.model, "nomic-embed-text");
}

// Helper function tests
mod helper_functions {
    use super::*;

    #[test]
    fn test_system_message_creation() {
        use the_agency::llm::system_message;

        let msg = system_message("You are helpful");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "You are helpful");
    }

    #[test]
    fn test_user_message_creation() {
        use the_agency::llm::user_message;

        let msg = user_message("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_assistant_message_creation() {
        use the_agency::llm::assistant_message;

        let msg = assistant_message("Hi there");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "Hi there");
    }
}
