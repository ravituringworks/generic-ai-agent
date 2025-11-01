//! Example: Multi-Provider LLM Usage
//!
//! Demonstrates how to use multiple LLM providers with fallback support

use the_agency::error::Result;
use the_agency::llm::provider::{LlmProvider, ProviderConfig, ProviderType};
use the_agency::llm::providers::{
    AzureOpenAIProvider, GroqProvider, OpenAIProvider, TogetherProvider,
};
use the_agency::llm::user_message;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Multi-Provider LLM Example ===\n");

    // Example 1: Using OpenAI directly
    println!("1. OpenAI Provider");
    if let Ok(openai) = OpenAIProvider::from_env(
        "gpt-4".to_string(),
        Some("text-embedding-ada-002".to_string()),
    ) {
        demonstrate_provider(openai, "OpenAI").await?;
    } else {
        println!("   Skipping: OPENAI_API_KEY not set\n");
    }

    // Example 2: Using Groq for fast inference
    println!("2. Groq Provider (Fast Inference)");
    if let Ok(groq) = GroqProvider::from_env("llama3-70b-8192".to_string()) {
        demonstrate_provider(groq, "Groq").await?;
    } else {
        println!("   Skipping: GROQ_API_KEY not set\n");
    }

    // Example 3: Using Together AI
    println!("3. Together AI Provider");
    if let Ok(together) = TogetherProvider::from_env(
        "meta-llama/Llama-3-70b-chat-hf".to_string(),
        Some("togethercomputer/m2-bert-80M-8k-retrieval".to_string()),
    ) {
        demonstrate_provider(together, "Together AI").await?;
    } else {
        println!("   Skipping: TOGETHER_API_KEY not set\n");
    }

    // Example 4: Using Azure OpenAI
    println!("4. Azure OpenAI Provider");
    if let Ok(endpoint) = std::env::var("AZURE_OPENAI_ENDPOINT") {
        if let Ok(azure) = AzureOpenAIProvider::from_env(
            endpoint,
            "gpt-4-deployment".to_string(),
            "gpt-4".to_string(),
        ) {
            demonstrate_provider(azure, "Azure OpenAI").await?;
        }
    } else {
        println!("   Skipping: AZURE_OPENAI_ENDPOINT not set\n");
    }

    // Example 5: Creating custom provider config
    println!("5. Custom Provider Configuration");
    let custom_config = ProviderConfig {
        provider: ProviderType::OpenAI,
        name: "custom-openai".to_string(),
        priority: 1,
        api_key: std::env::var("OPENAI_API_KEY").ok(),
        base_url: Some("https://api.openai.com/v1".to_string()),
        text_model: "gpt-3.5-turbo".to_string(),
        embedding_model: Some("text-embedding-ada-002".to_string()),
        max_tokens: 2048,
        temperature: 0.8,
        timeout: 60,
        options: serde_json::Value::Null,
    };

    let provider = OpenAIProvider::create(custom_config);
    println!("   Created custom provider: {}", provider.name());
    println!("   Provider type: {}", provider.provider_type());
    println!();

    println!("=== Example Complete ===");
    println!("\nKey Benefits:");
    println!("  • Unified interface across all providers");
    println!("  • Easy provider switching without code changes");
    println!("  • Automatic fallback support");
    println!("  • ~90% code reuse for OpenAI-compatible providers");

    Ok(())
}

async fn demonstrate_provider(provider: std::sync::Arc<dyn LlmProvider>, name: &str) -> Result<()> {
    println!("   Provider: {}", name);
    println!("   Type: {}", provider.provider_type());

    let messages = vec![user_message("What is 2+2? Answer briefly.")];

    match provider.generate(&messages).await {
        Ok(response) => {
            println!("   Response: {}", response.text.trim());
            if let Some(tokens) = response.tokens_used {
                println!("   Tokens used: {}", tokens);
            }
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }

    println!();
    Ok(())
}
