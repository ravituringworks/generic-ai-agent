//! Multi-Provider LLM Example
//!
//! Demonstrates using the ProviderManager to automatically fallback between
//! multiple LLM providers (e.g., Ollama -> OpenAI -> Claude)

use std::collections::HashMap;
use the_agency::{
    config::LlmConfig,
    llm::{
        manager::{ManagerConfig, ProviderManager},
        user_message, LlmClient,
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Multi-Provider LLM Example");
    println!("=============================\n");

    // Configuration for primary provider (Ollama - local)
    let ollama_config = LlmConfig {
        ollama_url: "http://localhost:11434".to_string(),
        text_model: "qwen3-coder:480b-cloud".to_string(),
        embedding_model: "nomic-embed-text".to_string(),
        max_tokens: 1000,
        temperature: 0.7,
        timeout: 30,
        stream: false,
        task_models: HashMap::new(),
        cache: the_agency::cache::LlmCacheConfig::default(),
    };

    // Create provider manager with automatic fallback
    let manager_config = ManagerConfig {
        enable_fallback: true,
        max_retries: 2,
        retry_delay_ms: 1000,
    };

    println!("📋 Configuring Provider Manager");
    println!("  • Primary: Ollama (local) - qwen3-coder:480b-cloud");
    println!("  • Fallback: Enabled");
    println!("  • Max Retries: {}", manager_config.max_retries);
    println!("  • Retry Delay: {}ms\n", manager_config.retry_delay_ms);

    let provider = ProviderManager::new_ollama(ollama_config).with_config(manager_config);

    // Example 1: Simple generation
    println!("📝 Example 1: Simple Text Generation");
    println!("{}", "-".repeat(50));

    let messages = vec![user_message("What is Rust programming language?")];

    match provider.generate(&messages).await {
        Ok(response) => {
            println!("✅ Success!");
            println!("Model: {}", response.model);
            println!("Response: {}\n", response.text);
            if let Some(tokens) = response.tokens_used {
                println!("Tokens used: {}", tokens);
            }
        }
        Err(e) => {
            println!("❌ Generation failed: {}", e);
            println!("Note: Make sure Ollama is running (ollama serve)");
            println!("Or configure additional fallback providers\n");
        }
    }

    // Example 2: Embedding generation
    println!("\n🔢 Example 2: Embedding Generation");
    println!("{}", "-".repeat(50));

    match provider
        .embed("Rust is a systems programming language")
        .await
    {
        Ok(response) => {
            println!("✅ Success!");
            println!("Model: {}", response.model);
            println!("Embedding dimensions: {}", response.embedding.len());
            println!(
                "First 5 values: {:?}",
                &response.embedding[..5.min(response.embedding.len())]
            );
        }
        Err(e) => {
            println!("❌ Embedding failed: {}", e);
        }
    }

    // Example 3: List available models
    println!("\n📚 Example 3: Available Models");
    println!("{}", "-".repeat(50));

    match provider.list_models().await {
        Ok(models) => {
            println!("✅ Available models:");
            for model in models {
                println!("  • {}", model);
            }
        }
        Err(e) => {
            println!("❌ Could not list models: {}", e);
        }
    }

    // Example 4: Check model availability
    println!("\n🔍 Example 4: Check Model Availability");
    println!("{}", "-".repeat(50));

    let test_models = vec!["qwen3-coder:480b-cloud", "gpt-4", "claude-3"];

    for model in test_models {
        match provider.is_model_available(model).await {
            Ok(available) => {
                let status = if available { "✅" } else { "❌" };
                println!(
                    "{} {}: {}",
                    status,
                    model,
                    if available {
                        "available"
                    } else {
                        "not available"
                    }
                );
            }
            Err(e) => {
                println!("❌ Error checking {}: {}", model, e);
            }
        }
    }

    // Example 5: Provider Manager Benefits
    println!("\n💡 Provider Manager Benefits");
    println!("{}", "=".repeat(50));
    println!(
        "
1. Automatic Fallback
   - Primary provider fails → Automatically tries fallbacks
   - No code changes needed when adding providers
   
2. Configurable Retry Logic
   - Retry failed requests with exponential backoff
   - Prevents transient failures from breaking your app
   
3. Transparent Integration
   - Implements LlmClient trait
   - Drop-in replacement for any LLM client
   
4. Detailed Logging
   - Track which provider served each request
   - Debug provider failures easily
   
5. Cost Optimization
   - Use cheap local models (Ollama) first
   - Fallback to cloud providers only when needed
"
    );

    println!("\n🔧 How to Add More Providers");
    println!("{}", "=".repeat(50));
    println!(
        "
// Example: Add OpenAI as fallback
let openai_client = Arc::new(OpenAIClient::new(openai_config));

let manager = ProviderManager::new_ollama(ollama_config)
    .with_fallback(openai_client)
    .with_config(manager_config);

// Now requests will try:
// 1. Ollama (local)
// 2. OpenAI (cloud) if Ollama fails
"
    );

    println!("\n✅ Example Complete!");
    println!("{}", "=".repeat(50));

    Ok(())
}
