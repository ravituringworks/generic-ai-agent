use the_agency::config::LlmConfig;
use the_agency::llm::{system_message, user_message, LlmClient, OllamaClient};

#[tokio::main]
async fn main() {
    // Test with localhost
    println!("Testing connection to Ollama at http://localhost:11434");

    let mut config = LlmConfig {
        ollama_url: "http://localhost:11434".to_string(),
        text_model: "llama3.2".to_string(),
        ..Default::default()
    };

    let client = OllamaClient::new(config.clone());

    let messages = vec![
        system_message("You are a helpful assistant"),
        user_message("Say hello in 5 words or less"),
    ];

    match client.generate(&messages).await {
        Ok(response) => {
            println!("✅ SUCCESS!");
            println!("Response: {}", response.text);
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
            println!("\nTrying with 127.0.0.1 instead...");

            // Try with 127.0.0.1
            config.ollama_url = "http://127.0.0.1:11434".to_string();
            let client2 = OllamaClient::new(config);

            match client2.generate(&messages).await {
                Ok(response) => {
                    println!("✅ SUCCESS with 127.0.0.1!");
                    println!("Response: {}", response.text);
                }
                Err(e2) => {
                    println!("❌ ALSO FAILED with 127.0.0.1: {}", e2);
                }
            }
        }
    }
}
