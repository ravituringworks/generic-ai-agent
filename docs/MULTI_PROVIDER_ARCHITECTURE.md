# Multi-Provider LLM Architecture

## Overview

The Agency now supports multiple LLM providers with minimal code duplication through a trait-based abstraction layer.

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                     LlmProvider Trait                       │
│  (generate, embed, list_models, is_model_available, etc.)   │
└──────────────────────────┬──────────────────────────────────┘
                           │
          ┌────────────────┴────────────────┐
          │                                 │
┌─────────▼──────────┐          ┌──────────▼────────────────┐
│  OllamaClient      │          │ OpenAICompatibleProvider  │
│  (Direct impl)     │          │    (Generic base)         │
└────────────────────┘          └───────────┬───────────────┘
                                            │
                    ┌───────────────────────┼──────────────────┐
                    │                       │                  │
          ┌─────────▼─────────┐   ┌─────── ─▼───────┐  ┌───────▼────────┐
          │  OpenAIAdapter    │   │   GroqAdapter   │  │ TogetherAdapter│
          │  (api.openai.com) │   │ (api.groq.com)  │  │(api.together)  │
          └───────────────────┘   └─────────────────┘  └────────────────┘
                    │
          ┌─────────▼──────────┐
          │ AzureOpenAIAdapter │
          │  (Custom routing)  │
          └────────────────────┘
```

## Key Components

### 1. Base Layer (`providers/base.rs`)

**HttpProviderClient**: Common HTTP client with:

- Timeout handling
- Error mapping (401, 429, 5xx)
- JSON serialization/deserialization
- Reusable across all cloud providers

**OpenAICompatible Trait**: Interface for OpenAI-compatible providers:

- `base_url()` - API endpoint
- `api_key()` - Authentication
- `auth_headers()` - Custom headers
- `transform_endpoint()` - Provider-specific routing
- `build_url()` - Full URL construction

### 2. OpenAI-Compatible Layer (`providers/openai_compatible.rs`)

**OpenAICompatibleProvider< T >**: Generic implementation for any OpenAI-compatible API:

- Handles chat completions
- Handles embeddings
- Handles model listing
- Converts between Message types
- ~300 lines of shared code

### 3. Provider Implementations

Each provider is just an adapter (~50 lines):

**OpenAI** (`providers/openai.rs`):

```rust
struct OpenAIAdapter {
    base_url: String,
    api_key: Option<String>,
}

impl OpenAICompatible for OpenAIAdapter {
    fn base_url(&self) -> &str { &self.base_url }
    fn api_key(&self) -> Option<&str> { self.api_key.as_deref() }
}
```

**Groq** (`providers/openai_variants.rs`):

```rust
struct GroqAdapter { api_key: Option<String> }

impl OpenAICompatible for GroqAdapter {
    fn base_url(&self) -> &str { "https://api.groq.com/openai/v1" }
    fn api_key(&self) -> Option<&str> { self.api_key.as_deref() }
}
```

**Azure OpenAI**: Custom endpoint transformation for deployment-based routing

## Code Reuse Metrics

| Provider | Lines of Code | Shared Code | Code Reuse |
|----------|--------------|-------------|------------|
| OpenAI | ~90 | ~300 | 77% |
| Groq | ~65 | ~300 | 82% |
| Together | ~70 | ~300 | 81% |
| Azure | ~95 | ~300 | 76% |

**Total**: ~320 lines for 4 providers vs ~1200 lines without abstraction
**Savings**: ~73% reduction in code

## Implemented Providers

### Currently Implemented

- ✅ **Ollama** - Local inference (existing)
- ✅ **OpenAI** - GPT-4, GPT-3.5
- ✅ **Groq** - Fast inference with LPU
- ✅ **Together AI** - 50+ open source models
- ✅ **Azure OpenAI** - Enterprise deployment

### Planned (Same Pattern)

- ⏳ **Anthropic** - Claude 3 (different message format)
- ⏳ **Google Gemini** - Gemini Pro (different API)
- ⏳ **Replicate** - Various models
- ⏳ **HuggingFace** - 350k+ models
- ⏳ **Cohere** - Embeddings & RAG

## Usage Examples

### Simple Provider Creation

```rust
// From environment variable
let openai = OpenAIProvider::from_env(
    "gpt-4".to_string(),
    Some("text-embedding-ada-002".to_string())
)?;

// Custom configuration
let config = ProviderConfig {
    provider: ProviderType::Groq,
    name: "groq-fast".to_string(),
    api_key: Some(api_key),
    text_model: "llama3-70b-8192".to_string(),
    max_tokens: 8192,
    temperature: 0.7,
    // ...
};
let groq = GroqProvider::create(config);
```

### Unified Interface

```rust
// All providers use the same interface
let messages = vec![user_message("Hello!")];

let response1 = openai.generate(&messages).await?;
let response2 = groq.generate(&messages).await?;
let response3 = together.generate(&messages).await?;
// Same API, different backends
```

### Provider Manager with Fallback

```rust
let manager = ProviderManager::new_ollama(config)
    .with_fallback(Arc::new(groq_provider))
    .with_fallback(Arc::new(openai_provider))
    .with_config(ManagerConfig {
        enable_fallback: true,
        max_retries: 2,
        retry_delay_ms: 1000,
    });

// Automatically tries fallbacks if primary fails
let response = manager.generate(&messages).await?;
```

## Adding a New Provider

### For OpenAI-Compatible APIs (5 minutes)

1. Create adapter struct:

```rust
struct NewProviderAdapter {
    api_key: Option<String>,
}
```

2. Implement trait:

```rust
impl OpenAICompatible for NewProviderAdapter {
    fn base_url(&self) -> &str { "https://api.newprovider.com/v1" }
    fn api_key(&self) -> Option<&str> { self.api_key.as_deref() }
}
```

3. Create type alias:

```rust
pub type NewProvider = OpenAICompatibleProvider<NewProviderAdapter>;
```

4. Add convenience methods:

```rust
impl NewProvider {
    pub fn create(config: ProviderConfig) -> Arc<dyn LlmProvider> {
        let adapter = NewProviderAdapter::new(config.api_key.clone());
        Arc::new(OpenAICompatibleProvider::new(adapter, config))
    }
}
```

Done! ~30-50 lines total.

### For Custom APIs (30-60 minutes)

Implement the `LlmProvider` trait directly with custom request/response handling.
Can still use `HttpProviderClient` for HTTP operations.

## Benefits

1. **Minimal Duplication**: 73% code reduction for similar providers
2. **Type Safety**: Compile-time guarantees for API contracts
3. **Easy Testing**: Mock any provider through the trait
4. **Flexibility**: Mix and match providers at runtime
5. **Maintainability**: Fix bugs once, all providers benefit
6. **Extensibility**: New providers in minutes, not hours

## Configuration

See `config.example.toml` for multi-provider configuration with fallback strategies.

## Testing

```bash
# Run all provider tests
cargo test providers::

# Run specific provider test
cargo test providers::openai::tests

# Run with output
cargo test providers:: -- --nocapture
```

## Next Steps

1. Implement Anthropic provider (different message format)
2. Implement Google Gemini provider
3. Add remaining providers (Replicate, HuggingFace, Cohere)
4. Integrate with ProviderManager for config-driven instantiation
5. Add comprehensive integration tests
6. Add cost tracking and rate limiting per provider
