//! Test helpers and utilities for The Agency test suite

use std::path::PathBuf;
use tempfile::TempDir;
use the_agency::AgentConfig;

/// Creates a temporary directory for test artifacts
pub fn create_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Creates a test agent configuration with in-memory database
pub fn create_test_config() -> AgentConfig {
    let mut config = AgentConfig::default();
    config.memory.database_url = Some(":memory:".to_string());
    config.memory.persistent = false;
    config.agent.use_memory = true;
    config.agent.use_tools = true;
    config
}

/// Creates a test agent configuration with file-based database
pub fn create_test_config_with_file(temp_dir: &TempDir) -> AgentConfig {
    let db_path = temp_dir.path().join("test.db");
    let mut config = AgentConfig::default();
    config.memory.database_url = Some(format!("sqlite://{}", db_path.to_str().unwrap()));
    config.memory.persistent = true;
    config.agent.use_memory = true;
    config.agent.use_tools = true;
    config
}

/// Checks if Ollama is running and accessible
pub async fn is_ollama_available() -> bool {
    reqwest::Client::new()
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .await
        .is_ok()
}

/// Checks if a specific Ollama model is available
pub async fn is_ollama_model_available(model: &str) -> bool {
    if !is_ollama_available().await {
        return false;
    }

    let response = reqwest::Client::new()
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .await;

    if let Ok(resp) = response {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                return models.iter().any(|m| {
                    m.get("name")
                        .and_then(|n| n.as_str())
                        .map(|name| name.contains(model))
                        .unwrap_or(false)
                });
            }
        }
    }

    false
}

/// Checks if API key is set for a provider
pub fn has_api_key(provider: &str) -> bool {
    match provider {
        "openai" => std::env::var("OPENAI_API_KEY").is_ok(),
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").is_ok(),
        "google" => std::env::var("GOOGLE_API_KEY").is_ok(),
        "groq" => std::env::var("GROQ_API_KEY").is_ok(),
        "together" => std::env::var("TOGETHER_API_KEY").is_ok(),
        "azure" => {
            std::env::var("AZURE_OPENAI_API_KEY").is_ok()
                && std::env::var("AZURE_OPENAI_ENDPOINT").is_ok()
        }
        _ => false,
    }
}

/// Waits for a condition to be true with timeout
pub async fn wait_for_condition<F>(mut condition: F, timeout_secs: u64) -> bool
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    false
}

/// Creates a test message for agent processing
pub fn create_test_message(content: &str) -> String {
    content.to_string()
}

/// Asserts that a response contains expected keywords
pub fn assert_response_contains(response: &str, keywords: &[&str]) {
    for keyword in keywords {
        assert!(
            response.to_lowercase().contains(&keyword.to_lowercase()),
            "Response should contain '{}'\nResponse: {}",
            keyword,
            response
        );
    }
}

/// Asserts that a response does not contain forbidden keywords
pub fn assert_response_not_contains(response: &str, keywords: &[&str]) {
    for keyword in keywords {
        assert!(
            !response.to_lowercase().contains(&keyword.to_lowercase()),
            "Response should not contain '{}'\nResponse: {}",
            keyword,
            response
        );
    }
}

/// Creates a test workflow context
pub fn create_test_workflow_context() -> the_agency::workflow::WorkflowContext {
    the_agency::workflow::WorkflowContext::new(10)
}

/// Creates test resource ID for storage tests
pub fn create_test_resource_id(namespace: &str, id: &str) -> the_agency::unified_storage::ResourceId {
    the_agency::unified_storage::ResourceId::new(namespace, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_config() {
        let config = create_test_config();
        assert!(config.agent.use_memory);
        assert!(config.agent.use_tools);
    }

    #[test]
    fn test_create_test_dir() {
        let dir = create_test_dir();
        assert!(dir.path().exists());
    }

    #[tokio::test]
    async fn test_wait_for_condition() {
        let mut counter = 0;
        let result = wait_for_condition(
            || {
                counter += 1;
                counter >= 3
            },
            1,
        )
        .await;
        assert!(result);
        assert!(counter >= 3);
    }

    #[test]
    fn test_assert_response_contains() {
        let response = "This is a test response with AI and machine learning";
        assert_response_contains(response, &["test", "AI", "machine learning"]);
    }

    #[test]
    #[should_panic]
    fn test_assert_response_contains_missing() {
        let response = "This is a test response";
        assert_response_contains(response, &["missing_keyword"]);
    }
}
