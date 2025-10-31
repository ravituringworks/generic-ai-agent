# LLM Multi-Provider Implementation Guide

## Status

**Phase 1**: ✅ Architecture & Documentation Complete  
**Phase 2**: ⏳ In Progress (OpenAI provider started)  
**Phase 3**: 📋 Planned

## What's Been Completed

1. ✅ **Documentation**: `docs/LLM_PROVIDERS.md` - Complete guide for 10 providers
2. ✅ **Provider Trait**: `src/llm/provider.rs` - Core strategy pattern interface
3. ✅ **Architecture Design**: Strategy pattern with fallback support

## Implementation Roadmap

### Phase 1: Core Architecture ✅

**Files Created:**
- `src/llm/provider.rs` - Provider trait and types
- `docs/LLM_PROVIDERS.md` - Complete provider documentation

**What It Provides:**
- `LlmProvider` trait - Common interface for all providers
- `ProviderType` enum - Supported provider types
- `ProviderConfig` - Configuration structure
- `FallbackConfig` - Multi-provider fallback configuration
- `ProviderStats` - Usage tracking

### Phase 2: Provider Implementations ⏳

#### 2.1 Refactor Ollama (Adapt to new trait)

**File**: `src/llm/providers/ollama.rs`

```rust
use super::provider::{LlmProvider, ProviderType, ProviderConfig};

pub struct OllamaProvider {
    config: ProviderConfig,
    client: reqwest::Client,
    // ... existing fields
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Ollama
    }
    
    // Implement other trait methods...
}
```

#### 2.2 Implement OpenAI Provider

**File**: `src/llm/providers/openai.rs`

**Dependencies to add to Cargo.toml:**
```toml
# OpenAI-compatible providers
async-openai = "0.20"  # Official OpenAI Rust client
```

**Implementation:**
```rust
use async_openai::{Client as OpenAIClient, types::*};
use crate::llm::provider::{LlmProvider, ProviderType};

pub struct OpenAIProvider {
    client: OpenAIClient<OpenAIConfig>,
    config: ProviderConfig,
    stats: Arc<RwLock<ProviderStats>>,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let api_key = config.api_key
            .or_else(|| env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| LlmError::Config("OpenAI API key required".to_string()))?;
        
        let mut openai_config = OpenAIConfig::new().with_api_key(api_key);
        
        if let Some(base_url) = &config.base_url {
            openai_config = openai_config.with_api_base(base_url);
        }
        
        Ok(Self {
            client: OpenAIClient::with_config(openai_config),
            config,
            stats: Arc::new(RwLock::new(ProviderStats::default())),
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenAI
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse> {
        // Convert our Message format to OpenAI format
        let openai_messages: Vec<ChatCompletionRequestMessage> = messages
            .iter()
            .map(|m| match m.role {
                Role::System => ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(m.content.clone()),
                    ..Default::default()
                }.into(),
                Role::User => ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(m.content.clone()),
                    ..Default::default()
                }.into(),
                Role::Assistant => ChatCompletionRequestAssistantMessage {
                    content: Some(ChatCompletionRequestAssistantMessageContent::Text(m.content.clone())),
                    ..Default::default()
                }.into(),
            })
            .collect();
        
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.text_model)
            .messages(openai_messages)
            .max_tokens(self.config.max_tokens)
            .temperature(self.config.temperature)
            .build()?;
        
        let response = self.client
            .chat()
            .create(request)
            .await
            .map_err(|e| LlmError::GenerationFailed(e.to_string()))?;
        
        let choice = response.choices.first()
            .ok_or_else(|| LlmError::InvalidResponse("No choices in response".to_string()))?;
        
        let text = choice.message.content.clone()
            .unwrap_or_default();
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.successful_requests += 1;
        if let Some(usage) = response.usage {
            stats.total_tokens_input += usage.prompt_tokens as u64;
            stats.total_tokens_output += usage.completion_tokens as u64;
            stats.estimated_cost_usd += calculate_openai_cost(&self.config.text_model, &usage);
        }
        
        Ok(GenerationResponse {
            text,
            tokens_used: response.usage.map(|u| u.total_tokens),
            model: response.model,
            finish_reason: choice.finish_reason.as_ref().map(|r| format!("{:?}", r)),
        })
    }
    
    async fn embed(&self, text: &str) -> Result<EmbeddingResponse> {
        let model = self.config.embedding_model.as_ref()
            .ok_or_else(|| LlmError::Config("No embedding model configured".to_string()))?;
        
        let request = CreateEmbeddingRequestArgs::default()
            .model(model)
            .input(text)
            .build()?;
        
        let response = self.client
            .embeddings()
            .create(request)
            .await
            .map_err(|e| LlmError::EmbeddingFailed(e.to_string()))?;
        
        let embedding = response.data.first()
            .ok_or_else(|| LlmError::InvalidResponse("No embeddings in response".to_string()))?
            .embedding.clone();
        
        Ok(EmbeddingResponse {
            embedding,
            model: response.model,
        })
    }
    
    async fn list_models(&self) -> Result<Vec<String>> {
        let response = self.client
            .models()
            .list()
            .await
            .map_err(|e| LlmError::ConnectionFailed(e.to_string()))?;
        
        Ok(response.data.into_iter().map(|m| m.id).collect())
    }
    
    async fn is_model_available(&self, model: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m == model))
    }
    
    fn get_stats(&self) -> ProviderStats {
        self.stats.blocking_read().clone()
    }
}

fn calculate_openai_cost(model: &str, usage: &CompletionUsage) -> f64 {
    // Pricing per 1M tokens (as of 2024)
    let (input_cost, output_cost) = match model {
        m if m.contains("gpt-4-turbo") => (10.0, 30.0),
        m if m.contains("gpt-4") => (30.0, 60.0),
        m if m.contains("gpt-3.5-turbo") => (0.50, 1.50),
        _ => (0.0, 0.0),
    };
    
    let input_usd = (usage.prompt_tokens as f64 / 1_000_000.0) * input_cost;
    let output_usd = (usage.completion_tokens as f64 / 1_000_000.0) * output_cost;
    
    input_usd + output_usd
}
```

#### 2.3 Implement Anthropic Provider

**File**: `src/llm/providers/anthropic.rs`

**Dependencies:**
```toml
anthropic-sdk = "0.2"  # Or use reqwest directly
```

**Key Differences from OpenAI:**
- Uses `messages` API with system parameter separate
- Different token counting
- Streaming format is different
- Max context: 200K tokens

#### 2.4 Implement Google/Gemini Provider

**File**: `src/llm/providers/google.rs`

**Dependencies:**
```toml
google-generative-ai-rs = "0.1"  # Or reqwest
```

**Key Features:**
- Free tier available
- Multimodal support
- Different API structure

#### 2.5 Implement Groq Provider

**File**: `src/llm/providers/groq.rs`

**Note**: Groq uses OpenAI-compatible API, so can reuse most OpenAI code:

```rust
pub struct GroqProvider {
    inner: OpenAIProvider,  // Reuse OpenAI implementation
}

impl GroqProvider {
    pub fn new(mut config: ProviderConfig) -> Result<Self> {
        // Override base URL for Groq
        config.base_url = Some("https://api.groq.com/openai/v1".to_string());
        
        Ok(Self {
            inner: OpenAIProvider::new(config)?,
        })
    }
}

#[async_trait]
impl LlmProvider for GroqProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Groq
    }
    
    // Delegate to inner provider
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse> {
        self.inner.generate(messages).await
    }
    
    // ... other delegated methods
}
```

### Phase 3: Provider Factory & Manager ⏳

#### 3.1 Provider Factory

**File**: `src/llm/factory.rs`

```rust
use super::provider::{LlmProvider, ProviderType, ProviderConfig};
use super::providers::*;

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(config: ProviderConfig) -> Result<Box<dyn LlmProvider>> {
        match config.provider {
            ProviderType::Ollama => {
                Ok(Box::new(OllamaProvider::new(config)?))
            },
            ProviderType::OpenAI => {
                Ok(Box::new(OpenAIProvider::new(config)?))
            },
            ProviderType::Anthropic => {
                Ok(Box::new(AnthropicProvider::new(config)?))
            },
            ProviderType::Google => {
                Ok(Box::new(GoogleProvider::new(config)?))
            },
            ProviderType::AzureOpenAI => {
                Ok(Box::new(AzureOpenAIProvider::new(config)?))
            },
            ProviderType::Groq => {
                Ok(Box::new(GroqProvider::new(config)?))
            },
            ProviderType::Together => {
                Ok(Box::new(TogetherProvider::new(config)?))
            },
            ProviderType::Replicate => {
                Ok(Box::new(ReplicateProvider::new(config)?))
            },
            ProviderType::HuggingFace => {
                Ok(Box::new(HuggingFaceProvider::new(config)?))
            },
            ProviderType::Cohere => {
                Ok(Box::new(CohereProvider::new(config)?))
            },
        }
    }
}
```

#### 3.2 Multi-Provider Manager

**File**: `src/llm/manager.rs`

```rust
use super::provider::{LlmProvider, FallbackConfig, FallbackStrategy};

pub struct ProviderManager {
    providers: Vec<Box<dyn LlmProvider>>,
    fallback_config: FallbackConfig,
    current_index: AtomicUsize,  // For round-robin
}

impl ProviderManager {
    pub fn new(
        providers: Vec<Box<dyn LlmProvider>>,
        fallback_config: FallbackConfig,
    ) -> Self {
        // Sort by priority
        let mut providers = providers;
        providers.sort_by_key(|p| {
            // Get priority from provider config somehow
            0u8  // TODO: Need to store priority in provider
        });
        
        Self {
            providers,
            fallback_config,
            current_index: AtomicUsize::new(0),
        }
    }
    
    async fn generate_with_fallback(
        &self,
        messages: &[Message],
    ) -> Result<GenerationResponse> {
        let mut last_error = None;
        
        for provider in &self.providers {
            for attempt in 0..self.fallback_config.max_retries {
                match provider.generate(messages).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        warn!(
                            "Provider {} failed (attempt {}): {}",
                            provider.name(),
                            attempt + 1,
                            e
                        );
                        last_error = Some(e);
                        
                        if attempt < self.fallback_config.max_retries - 1 {
                            tokio::time::sleep(
                                Duration::from_millis(self.fallback_config.retry_delay_ms)
                            ).await;
                        }
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            LlmError::GenerationFailed("All providers failed".to_string()).into()
        }))
    }
    
    fn select_provider_round_robin(&self) -> &Box<dyn LlmProvider> {
        let index = self.current_index.fetch_add(1, Ordering::SeqCst) % self.providers.len();
        &self.providers[index]
    }
}

#[async_trait]
impl LlmProvider for ProviderManager {
    fn provider_type(&self) -> ProviderType {
        // Return type of current primary provider
        self.providers.first()
            .map(|p| p.provider_type())
            .unwrap_or(ProviderType::Ollama)
    }
    
    fn name(&self) -> &str {
        "multi-provider-manager"
    }
    
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResponse> {
        match self.fallback_config.strategy {
            FallbackStrategy::Priority => {
                self.generate_with_fallback(messages).await
            },
            FallbackStrategy::RoundRobin => {
                let provider = self.select_provider_round_robin();
                provider.generate(messages).await
            },
            // Implement other strategies...
            _ => self.generate_with_fallback(messages).await,
        }
    }
    
    // ... other trait methods
}
```

#### 3.3 Update Configuration

**File**: `src/config.rs`

Add to `LlmConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    // Keep existing fields for backward compatibility
    #[serde(default)]
    pub ollama_url: String,
    
    // NEW: Multi-provider support
    /// List of provider configurations
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    
    /// Fallback configuration
    #[serde(default)]
    pub fallback: FallbackConfig,
    
    /// Single provider (legacy, for backward compatibility)
    #[serde(default)]
    pub provider: Option<ProviderType>,
    
    // ... rest of fields
}
```

#### 3.4 Update LlmClient Creation

**File**: Update `src/agent.rs` or wherever LlmClient is created:

```rust
use crate::llm::{ProviderFactory, ProviderManager};

// In agent initialization:
let llm_client: Box<dyn LlmClient> = if config.llm.providers.is_empty() {
    // Legacy single provider (Ollama)
    Box::new(OllamaClient::new(config.llm.clone()))
} else if config.llm.providers.len() == 1 {
    // Single provider from new config
    ProviderFactory::create(config.llm.providers[0].clone())?
} else {
    // Multiple providers with fallback
    let providers: Result<Vec<_>> = config.llm.providers
        .iter()
        .map(|cfg| ProviderFactory::create(cfg.clone()))
        .collect();
    
    Box::new(ProviderManager::new(providers?, config.llm.fallback.clone()))
};
```

## Configuration Examples

### Single Provider (OpenAI)

```toml
[[llm.providers]]
name = "openai-primary"
provider = "openai"
api_key = "${OPENAI_API_KEY}"
text_model = "gpt-4-turbo-preview"
embedding_model = "text-embedding-3-small"
max_tokens = 4096
temperature = 0.7
priority = 1
```

### Multi-Provider with Fallback

```toml
[[llm.providers]]
name = "primary"
provider = "openai"
api_key = "${OPENAI_API_KEY}"
text_model = "gpt-4-turbo-preview"
priority = 1

[[llm.providers]]
name = "backup"
provider = "anthropic"
api_key = "${ANTHROPIC_API_KEY}"
text_model = "claude-3-sonnet-20240229"
priority = 2

[[llm.providers]]
name = "local"
provider = "ollama"
base_url = "http://localhost:11434"
text_model = "llama3.2"
priority = 3

[llm.fallback]
enabled = true
max_retries = 3
retry_delay_ms = 1000
strategy = "priority"
```

## Dependencies to Add

```toml
[dependencies]
# OpenAI
async-openai = "0.20"

# Anthropic
anthropic-sdk = "0.2"  # Or implement with reqwest

# Google
google-generative-ai-rs = "0.1"  # Or implement with reqwest

# For providers without SDKs, use reqwest directly
```

## Testing Strategy

1. **Unit Tests**: Test each provider individually with mocked responses
2. **Integration Tests**: Test with real API keys (optional, use env vars)
3. **Fallback Tests**: Test multi-provider fallback scenarios
4. **Cost Tracking Tests**: Verify cost calculations

## Migration Path

1. **Backward Compatible**: Existing Ollama configs continue to work
2. **Gradual Adoption**: Add providers one at a time
3. **No Breaking Changes**: Old code paths still work

## Next Steps

1. ✅ Complete Phase 1 (Architecture)
2. ⏳ **Start Phase 2.2**: Implement OpenAI provider fully
3. ⏳ **Phase 2.3-2.5**: Implement other high-priority providers
4. ⏳ **Phase 3**: Implement factory and manager
5. ⏳ **Update docs**: Add examples to README
6. ⏳ **Testing**: Add comprehensive tests

## File Structure

```
src/llm/
├── mod.rs                 # Main module (existing)
├── provider.rs           # NEW: Provider trait and types
├── factory.rs            # NEW: Provider factory
├── manager.rs            # NEW: Multi-provider manager
└── providers/
    ├── mod.rs            # Provider module exports
    ├── ollama.rs         # REFACTOR: Adapt existing
    ├── openai.rs         # NEW: OpenAI implementation
    ├── anthropic.rs      # NEW: Anthropic implementation
    ├── google.rs         # NEW: Google/Gemini implementation
    ├── groq.rs           # NEW: Groq implementation
    ├── together.rs       # NEW: Together AI implementation
    ├── replicate.rs      # NEW: Replicate implementation
    ├── huggingface.rs    # NEW: HuggingFace implementation
    └── cohere.rs         # NEW: Cohere implementation
```

## Estimated Effort

- **Phase 1**: ✅ Complete (2 hours)
- **Phase 2**: ~8-12 hours (1-2 hours per provider)
- **Phase 3**: ~4-6 hours (factory + manager + config updates)
- **Testing & Docs**: ~4-6 hours
- **Total**: ~20-30 hours of development time

## Priority Order

1. **High Priority**: OpenAI, Anthropic, Groq (most requested)
2. **Medium Priority**: Google, Azure OpenAI (enterprise)
3. **Low Priority**: Others (nice to have)
