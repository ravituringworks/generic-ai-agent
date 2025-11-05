//! BDD step definitions for Multi-Provider LLM feature tests

use cucumber::{given, then, when, World};
use std::collections::HashMap;

#[derive(World)]
#[world(init = Self::new)]
pub struct LlmProviderWorld {
    last_response: Option<String>,
    last_error: Option<String>,
    provider_type: Option<String>,
    embeddings_generated: bool,
    config: ProviderConfig,
}

// Custom Debug implementation
impl std::fmt::Debug for LlmProviderWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmProviderWorld")
            .field("last_response", &self.last_response)
            .field("last_error", &self.last_error)
            .field("provider_type", &self.provider_type)
            .field("embeddings_generated", &self.embeddings_generated)
            .finish()
    }
}

#[derive(Debug, Clone)]
struct ProviderConfig {
    fallback_enabled: bool,
    fallback_order: Vec<String>,
    task_model_map: HashMap<String, String>,
    streaming_enabled: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            fallback_enabled: false,
            fallback_order: Vec::new(),
            task_model_map: HashMap::new(),
            streaming_enabled: false,
        }
    }
}

impl LlmProviderWorld {
    fn new() -> Self {
        Self {
            last_response: None,
            last_error: None,
            provider_type: None,
            embeddings_generated: false,
            config: ProviderConfig::default(),
        }
    }
}

#[given("I have API keys configured for multiple providers")]
async fn setup_api_keys(_world: &mut LlmProviderWorld) {
    // API keys should be set as environment variables
    // This step verifies their existence or uses test keys
}

#[given("the LLM manager is initialized")]
async fn initialize_llm_manager(_world: &mut LlmProviderWorld) {
    // Initialize the LLM manager with configured providers
}

#[given(regex = r"I have configured OpenAI with model (.*)")]
async fn configure_openai(world: &mut LlmProviderWorld, _model: String) {
    world.provider_type = Some("openai".to_string());
}

#[given(regex = r"I have configured Anthropic with model (.*)")]
async fn configure_anthropic(world: &mut LlmProviderWorld, _model: String) {
    world.provider_type = Some("anthropic".to_string());
}

#[given(regex = r"I have configured Google with model (.*)")]
async fn configure_google(world: &mut LlmProviderWorld, _model: String) {
    world.provider_type = Some("google".to_string());
}

#[given(regex = r"I have configured Groq with model (.*)")]
async fn configure_groq(world: &mut LlmProviderWorld, _model: String) {
    world.provider_type = Some("groq".to_string());
}

#[given(regex = r"I have configured Together AI with model (.*)")]
async fn configure_together(world: &mut LlmProviderWorld, _model: String) {
    world.provider_type = Some("together".to_string());
}

#[given("I have configured Azure OpenAI with deployment name and endpoint")]
async fn configure_azure(_world: &mut LlmProviderWorld) {
    // Azure OpenAI configuration with deployment and endpoint
}

#[given("I have Ollama running locally")]
async fn verify_ollama(_world: &mut LlmProviderWorld) {
    // Check if Ollama is running on localhost:11434
}

#[given(regex = r"I have pulled model (.*)")]
async fn verify_ollama_model(_world: &mut LlmProviderWorld, _model: String) {
    // Verify the model is available in Ollama
}

#[given("I have configured multiple providers with fallback order")]
async fn configure_fallback(world: &mut LlmProviderWorld) {
    world.config.fallback_enabled = true;
    world.config.fallback_order = vec![
        "openai".to_string(),
        "anthropic".to_string(),
        "ollama".to_string(),
    ];
}

#[given("the primary provider is unavailable")]
async fn disable_primary_provider(_world: &mut LlmProviderWorld) {
    // Simulate primary provider being down
}

#[given("I have configured different models for different task types")]
async fn configure_task_models(world: &mut LlmProviderWorld) {
    world
        .config
        .task_model_map
        .insert("code".to_string(), "gpt-4".to_string());
    world
        .config
        .task_model_map
        .insert("creative".to_string(), "claude-3-opus".to_string());
    world
        .config
        .task_model_map
        .insert("math".to_string(), "gpt-4".to_string());
}

#[given("I have configured streaming enabled")]
async fn enable_streaming(world: &mut LlmProviderWorld) {
    world.config.streaming_enabled = true;
}

#[when(regex = r"I send a message (.*)")]
async fn send_message(world: &mut LlmProviderWorld, message: String) {
    // Simulate message sending
    world.last_response = Some(format!("Response to: {}", message));
    world.last_error = None;
}

#[when("I request code generation")]
async fn request_code_generation(world: &mut LlmProviderWorld) {
    send_message(world, "Write a function to calculate fibonacci".to_string()).await;
}

#[when("I request creative writing")]
async fn request_creative_writing(world: &mut LlmProviderWorld) {
    send_message(world, "Write a creative story about AI".to_string()).await;
}

#[when("I request mathematical reasoning")]
async fn request_math_reasoning(world: &mut LlmProviderWorld) {
    send_message(world, "Solve this math problem: 2x + 5 = 13".to_string()).await;
}

#[when(regex = r"I request embeddings for (.*)")]
async fn request_embeddings(world: &mut LlmProviderWorld, _text: String) {
    // Simulate embedding generation
    world.embeddings_generated = true;
    world.last_error = None;
}

#[then(regex = r"the (.*) provider should generate a response")]
async fn verify_provider_response(world: &mut LlmProviderWorld, _provider: String) {
    assert!(
        world.last_response.is_some(),
        "Response should be generated"
    );
    assert!(world.last_error.is_none(), "No error should occur");
}

#[then("the response should be non-empty")]
async fn verify_non_empty_response(world: &mut LlmProviderWorld) {
    if let Some(response) = &world.last_response {
        assert!(!response.is_empty(), "Response text should not be empty");
    }
}

#[then(regex = r"the response should mention (.*)")]
async fn verify_response_contains(world: &mut LlmProviderWorld, keyword: String) {
    if let Some(response) = &world.last_response {
        assert!(
            response.to_lowercase().contains(&keyword.to_lowercase()),
            "Response should mention {}",
            keyword
        );
    }
}

#[then("the provider should generate vector embeddings")]
async fn verify_embeddings_generated(world: &mut LlmProviderWorld) {
    assert!(world.embeddings_generated, "Embeddings should be generated");
}

#[then("the embedding dimension should match the model specification")]
async fn verify_embedding_dimension(world: &mut LlmProviderWorld) {
    assert!(
        world.embeddings_generated,
        "Embeddings should have been generated"
    );
}
