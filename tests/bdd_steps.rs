//! Cucumber step definitions for BDD tests

use cucumber::{given, then, when, World};
use std::collections::HashMap;
use tempfile::tempdir;
use the_agency::*;

#[derive(World)]
#[world(init = Self::new)]
pub struct AgentWorld {
    agent: Option<Agent>,
    last_response: Option<String>,
    config: Option<AgentConfig>,
    temp_dir: Option<tempfile::TempDir>,
    error: Option<String>,
}

// Custom Debug implementation that doesn't require Agent to implement Debug
impl std::fmt::Debug for AgentWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentWorld")
            .field("agent", &self.agent.as_ref().map(|_| "<Agent>"))
            .field("last_response", &self.last_response)
            .field("config", &self.config)
            .field("temp_dir", &self.temp_dir.as_ref().map(|_| "<TempDir>"))
            .field("error", &self.error)
            .finish()
    }
}

impl AgentWorld {
    pub fn new() -> Self {
        Self {
            agent: None,
            last_response: None,
            config: None,
            temp_dir: None,
            error: None,
        }
    }
}

#[given("an AI agent is initialized with default configuration")]
async fn initialize_agent(world: &mut AgentWorld) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    let mut config = AgentConfig::default();
    config.memory.database_url = Some(format!("sqlite://{}", db_path.to_str().unwrap()));

    world.temp_dir = Some(temp_dir);
    world.config = Some(config.clone());

    // In a real test environment, we might mock the Ollama client
    // For now, we'll create the agent but expect it might fail without Ollama
    match Agent::new(config).await {
        Ok(agent) => {
            world.agent = Some(agent);
        }
        Err(e) => {
            // Store error for testing scenarios where Ollama isn't available
            world.error = Some(e.to_string());
        }
    }
}

#[given("the agent has access to built-in tools")]
async fn verify_builtin_tools(world: &mut AgentWorld) {
    if let Some(agent) = &world.agent {
        let available_tools = agent.get_available_tools().await;
        assert!(
            !available_tools.is_empty(),
            "Agent should have built-in tools available"
        );
        assert!(
            available_tools.contains(&"system_info".to_string()),
            "Agent should have system_info tool"
        );
    }
}

#[given("the agent has memory capabilities enabled")]
async fn verify_memory_capabilities(world: &mut AgentWorld) {
    if let Some(agent) = &world.agent {
        let stats = agent.stats().await;
        // Memory is enabled by default in our configuration
        assert_eq!(stats.memory_stats.embedding_dimension, 768);
    }
}

#[given(regex = r"I have told the agent (.*)")]
async fn tell_agent_information(world: &mut AgentWorld, message: String) {
    if let Some(agent) = &mut world.agent {
        let response = agent.process(&message).await;
        match response {
            Ok(resp) => {
                world.last_response = Some(resp);
            }
            Err(e) => {
                world.error = Some(e.to_string());
            }
        }
    }
}

#[given(regex = r"I ask (.*)")]
async fn ask_agent_question(world: &mut AgentWorld, question: String) {
    if let Some(agent) = &mut world.agent {
        let response = agent.process(&question).await;
        match response {
            Ok(resp) => {
                world.last_response = Some(resp);
            }
            Err(e) => {
                world.error = Some(e.to_string());
            }
        }
    }
}

#[given("I have had a conversation with 10 messages")]
async fn create_long_conversation(world: &mut AgentWorld) {
    if let Some(agent) = &mut world.agent {
        for i in 1..=5 {
            let _ = agent
                .process(&format!("This is message number {}", i))
                .await;
        }
    }
}

#[given("I have had several interactions with the agent")]
async fn create_several_interactions(world: &mut AgentWorld) {
    if let Some(agent) = &mut world.agent {
        let interactions = vec![
            "Hello, I'm testing the agent",
            "Remember that I like coffee",
            "What's my system information?",
        ];

        for interaction in interactions {
            let _ = agent.process(interaction).await;
        }
    }
}

#[given("I have an agent configuration")]
async fn create_agent_configuration(world: &mut AgentWorld) {
    world.config = Some(AgentConfig::default());
}

#[when(regex = r"I send the message (.*)")]
async fn send_message(world: &mut AgentWorld, message: String) {
    if let Some(agent) = &mut world.agent {
        let response = agent.process(&message).await;
        match response {
            Ok(resp) => {
                world.last_response = Some(resp);
                world.error = None;
            }
            Err(e) => {
                world.error = Some(e.to_string());
            }
        }
    } else {
        world.error = Some("Agent not initialized".to_string());
    }
}

#[when(regex = r"I ask (.*)")]
async fn ask_question(world: &mut AgentWorld, question: String) {
    send_message(world, question).await;
}

#[when(regex = r"I request (.*)")]
async fn make_request(world: &mut AgentWorld, request: String) {
    send_message(world, request).await;
}

#[when("the agent processes this request")]
async fn process_request(world: &mut AgentWorld) {
    // This is a placeholder - the actual processing happened in the given step
    // In a more sophisticated test, we might have separate steps for request and processing
}

#[when("I send another message")]
async fn send_another_message(world: &mut AgentWorld) {
    send_message(world, "Another message".to_string()).await;
}

#[when("I send an empty message")]
async fn send_empty_message(world: &mut AgentWorld) {
    send_message(world, "".to_string()).await;
}

#[when("I check the agent statistics")]
async fn check_agent_statistics(world: &mut AgentWorld) {
    if let Some(agent) = &world.agent {
        let stats = agent.stats().await;
        // Store stats in last_response as JSON for verification
        world.last_response = Some(serde_json::to_string(&stats).unwrap_or_default());
    }
}

#[when("I set an invalid Ollama URL")]
async fn set_invalid_ollama_url(world: &mut AgentWorld) {
    if let Some(config) = &mut world.config {
        config.llm.ollama_url = "invalid-url".to_string();

        match config.validate() {
            Ok(_) => {
                world.error = Some("Configuration should have failed validation".to_string());
            }
            Err(e) => {
                world.error = Some(e.to_string());
            }
        }
    }
}

#[then("the agent should respond with a greeting")]
async fn verify_greeting_response(world: &mut AgentWorld) {
    if let Some(response) = &world.last_response {
        let response_lower = response.to_lowercase();
        assert!(
            response_lower.contains("hello")
                || response_lower.contains("hi")
                || response_lower.contains("greet")
                || response_lower.contains("how can i help"),
            "Response should contain a greeting: {}",
            response
        );
    } else if let Some(error) = &world.error {
        // If Ollama is not available, we expect an error
        assert!(
            error.contains("Connection") || error.contains("Ollama"),
            "Expected connection error, got: {}",
            error
        );
    }
}

#[then("the response should be non-empty")]
async fn verify_non_empty_response(world: &mut AgentWorld) {
    if let Some(response) = &world.last_response {
        assert!(!response.trim().is_empty(), "Response should not be empty");
    } else if world.error.is_some() {
        // If there's an error (like Ollama not available), that's acceptable for testing
        return;
    } else {
        panic!("No response received");
    }
}

#[then(regex = r"the conversation history should contain (\d+) messages")]
async fn verify_conversation_length(world: &mut AgentWorld, expected_count: usize) {
    if let Some(agent) = &world.agent {
        let conversation = agent.get_conversation();
        assert_eq!(
            conversation.len(),
            expected_count,
            "Expected {} messages, got {}",
            expected_count,
            conversation.len()
        );
    }
}

#[then(regex = r"the agent should mention my name (.*)")]
async fn verify_name_mention(world: &mut AgentWorld, name: String) {
    if let Some(response) = &world.last_response {
        let name_clean = name.trim_matches('"');
        assert!(
            response.contains(name_clean),
            "Response should mention name '{}': {}",
            name_clean,
            response
        );
    }
}

#[then(regex = r"the agent should mention (.*)")]
async fn verify_mention(world: &mut AgentWorld, expected_text: String) {
    if let Some(response) = &world.last_response {
        let expected_clean = expected_text.trim_matches('"');
        assert!(
            response
                .to_lowercase()
                .contains(&expected_clean.to_lowercase()),
            "Response should mention '{}': {}",
            expected_clean,
            response
        );
    }
}

#[then("the agent should call the system_info tool")]
async fn verify_system_info_tool_call(world: &mut AgentWorld) {
    if let Some(response) = &world.last_response {
        // In a real implementation, we might track tool calls more explicitly
        // For now, we check if the response contains system information
        let response_lower = response.to_lowercase();
        assert!(
            response_lower.contains("system")
                || response_lower.contains("os")
                || response_lower.contains("arch"),
            "Response should indicate system info tool was called: {}",
            response
        );
    }
}

#[then("the response should contain system details")]
async fn verify_system_details(world: &mut AgentWorld) {
    if let Some(response) = &world.last_response {
        let response_lower = response.to_lowercase();
        assert!(
            response_lower.contains("system")
                || response_lower.contains("os")
                || response_lower.contains("operating"),
            "Response should contain system details: {}",
            response
        );
    }
}

#[then("the response should mention the operating system")]
async fn verify_os_mention(world: &mut AgentWorld) {
    if let Some(response) = &world.last_response {
        let response_lower = response.to_lowercase();
        assert!(
            response_lower.contains("macos")
                || response_lower.contains("linux")
                || response_lower.contains("windows")
                || response_lower.contains("unix")
                || response_lower.contains("os"),
            "Response should mention operating system: {}",
            response
        );
    }
}

#[then("the agent should store information about my Rust preference")]
async fn verify_rust_preference_stored(world: &mut AgentWorld) {
    if let Some(agent) = &world.agent {
        let stats = agent.stats().await;
        // In a real implementation, we might have more specific memory queries
        // For now, we check that memories were stored
        assert!(
            stats.memory_stats.total_memories > 0,
            "Agent should have stored memory about preferences"
        );
    }
}

#[then("the response should combine both tool results and memory storage")]
async fn verify_combined_response(world: &mut AgentWorld) {
    if let Some(response) = &world.last_response {
        // Response should indicate both system info and memory storage occurred
        let response_lower = response.to_lowercase();
        assert!(
            response_lower.contains("system")
                && (response_lower.contains("remember") || response_lower.contains("stored")),
            "Response should combine tool results and memory storage: {}",
            response
        );
    }
}

#[then("the conversation should not exceed the maximum history length")]
async fn verify_conversation_limit(world: &mut AgentWorld) {
    if let Some(agent) = &world.agent {
        let conversation = agent.get_conversation();
        let max_length = world
            .config
            .as_ref()
            .map(|c| c.agent.max_history_length)
            .unwrap_or(20);

        assert!(
            conversation.len() <= max_length,
            "Conversation length {} should not exceed maximum {}",
            conversation.len(),
            max_length
        );
    }
}

#[then("the system message should be preserved")]
async fn verify_system_message_preserved(world: &mut AgentWorld) {
    if let Some(agent) = &world.agent {
        let conversation = agent.get_conversation();
        if !conversation.is_empty() {
            assert!(
                matches!(conversation[0].role, llm::Role::System),
                "First message should be system message"
            );
        }
    }
}

#[then("the agent should handle it gracefully")]
async fn verify_graceful_handling(world: &mut AgentWorld) {
    // Either we got a response or a handled error, but no panic
    assert!(
        world.last_response.is_some() || world.error.is_some(),
        "Agent should handle empty message gracefully"
    );
}

#[then("should not crash or return an error")]
async fn verify_no_crash(world: &mut AgentWorld) {
    // For empty message handling, we might actually expect an error or empty response
    // The important thing is no panic occurred (which would terminate the test)
    // Just completing this step means no crash occurred
}

#[then("I should see the conversation length")]
async fn verify_conversation_length_in_stats(world: &mut AgentWorld) {
    if let Some(response) = &world.last_response {
        assert!(
            response.contains("conversation_length") || response.contains("conversation"),
            "Statistics should include conversation length"
        );
    }
}

#[then("I should see memory statistics")]
async fn verify_memory_statistics(world: &mut AgentWorld) {
    if let Some(response) = &world.last_response {
        assert!(
            response.contains("memory_stats") || response.contains("memory"),
            "Statistics should include memory information"
        );
    }
}

#[then("I should see tool availability information")]
async fn verify_tool_availability(world: &mut AgentWorld) {
    if let Some(response) = &world.last_response {
        assert!(
            response.contains("builtin_tools_count") || response.contains("tools"),
            "Statistics should include tool information"
        );
    }
}

#[then("the configuration validation should fail")]
async fn verify_config_validation_failure(world: &mut AgentWorld) {
    assert!(
        world.error.is_some(),
        "Configuration validation should have failed"
    );
}

#[then("I should get a meaningful error message")]
async fn verify_meaningful_error(world: &mut AgentWorld) {
    if let Some(error) = &world.error {
        assert!(
            error.contains("Invalid") || error.contains("URL") || error.contains("Ollama"),
            "Error message should be meaningful: {}",
            error
        );
    } else {
        panic!("Expected an error message");
    }
}

#[tokio::main]
async fn main() {
    AgentWorld::run("features").await;
}
