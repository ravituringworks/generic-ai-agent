# LLM Provider Support

The Agency supports multiple LLM providers through a pluggable strategy pattern architecture. You can use one provider or configure multiple providers with automatic fallback.

## Supported Providers

### 1. Ollama (Local - Free)

**Best for**: Privacy, offline use, no API costs, full control

- **Type**: Self-hosted, local
- **Models**: Llama 3.2, Mistral, CodeLlama, Phi, etc.
- **Cost**: Free (hardware costs only)
- **Rate Limits**: None
- **Setup**: Install Ollama locally

```toml
[llm]
provider = "ollama"
ollama_url = "http://localhost:11434"
text_model = "llama3.2"
embedding_model = "nomic-embed-text"
max_tokens = 4096
temperature = 0.7
```

### 2. OpenAI (Cloud - Paid)

**Best for**: Production, highest quality, fastest responses

- **Type**: Cloud API
- **Models**: GPT-4, GPT-4 Turbo, GPT-3.5 Turbo
- **Cost**: Pay-per-token ($0.01-$0.06 per 1K tokens)
- **Rate Limits**: Tier-based (up to 10,000 RPM)
- **Embeddings**: text-embedding-3-small/large

```toml
[llm]
provider = "openai"
api_key = "${OPENAI_API_KEY}"  # Use environment variable
base_url = "https://api.openai.com/v1"  # Optional: for proxies
text_model = "gpt-4-turbo-preview"
embedding_model = "text-embedding-3-small"
max_tokens = 4096
temperature = 0.7
organization_id = "org-xxx"  # Optional
```

### 3. Anthropic Claude (Cloud - Paid)

**Best for**: Long context, safety, analysis tasks

- **Type**: Cloud API
- **Models**: Claude 3 Opus, Sonnet, Haiku
- **Cost**: Pay-per-token ($0.015-$0.075 per 1K tokens)
- **Rate Limits**: 5 requests/min (free tier), higher for paid
- **Context**: Up to 200K tokens

```toml
[llm]
provider = "anthropic"
api_key = "${ANTHROPIC_API_KEY}"
base_url = "https://api.anthropic.com"
text_model = "claude-3-opus-20240229"
embedding_model = "voyage-large-2-instruct"  # Via Voyage AI
max_tokens = 4096
temperature = 0.7
```

### 4. Google Gemini (Cloud - Free Tier + Paid)

**Best for**: Multimodal, free tier, Google ecosystem

- **Type**: Cloud API
- **Models**: Gemini Pro, Gemini Pro Vision
- **Cost**: Free tier available, then pay-per-token
- **Rate Limits**: 60 RPM (free), higher for paid
- **Features**: Native multimodal support

```toml
[llm]
provider = "google"
api_key = "${GOOGLE_API_KEY}"
base_url = "https://generativelanguage.googleapis.com/v1"
text_model = "gemini-pro"
embedding_model = "embedding-001"
max_tokens = 2048
temperature = 0.7
```

### 5. Azure OpenAI (Cloud - Paid)

**Best for**: Enterprise, compliance, Azure integration

- **Type**: Cloud API (Microsoft Azure)
- **Models**: Same as OpenAI (GPT-4, GPT-3.5)
- **Cost**: Pay-per-token, enterprise pricing
- **Rate Limits**: Configurable quotas
- **Features**: VNET, private endpoints, compliance

```toml
[llm]
provider = "azure-openai"
api_key = "${AZURE_OPENAI_API_KEY}"
base_url = "https://YOUR-RESOURCE.openai.azure.com"
deployment_name = "gpt-4-deployment"  # Your deployment name
api_version = "2024-02-15-preview"
text_model = "gpt-4"
embedding_model = "text-embedding-ada-002"
max_tokens = 4096
temperature = 0.7
```

### 6. Groq (Cloud - Free + Paid)

**Best for**: Speed, free tier, cost-effective

- **Type**: Cloud API with LPU acceleration
- **Models**: Llama 3, Mixtral, Gemma
- **Cost**: Free tier with generous limits, then cheap
- **Rate Limits**: 30 RPM (free), 14,400 RPM (paid)
- **Speed**: Extremely fast inference

```toml
[llm]
provider = "groq"
api_key = "${GROQ_API_KEY}"
base_url = "https://api.groq.com/openai/v1"
text_model = "llama3-70b-8192"
max_tokens = 8192
temperature = 0.7
```

### 7. Together AI (Cloud - Paid)

**Best for**: Open models, fine-tuning, custom deployments

- **Type**: Cloud API
- **Models**: 50+ open source models
- **Cost**: Pay-per-token, competitive pricing
- **Rate Limits**: Based on plan
- **Features**: Fine-tuning, custom models

```toml
[llm]
provider = "together"
api_key = "${TOGETHER_API_KEY}"
base_url = "https://api.together.xyz/v1"
text_model = "meta-llama/Llama-3-70b-chat-hf"
embedding_model = "togethercomputer/m2-bert-80M-8k-retrieval"
max_tokens = 4096
temperature = 0.7
```

### 8. Replicate (Cloud - Pay-per-use)

**Best for**: Experimentation, diverse models, pay-per-use

- **Type**: Cloud API
- **Models**: Hundreds of models including Stable Diffusion
- **Cost**: Pay-per-second of compute
- **Rate Limits**: Based on plan
- **Features**: Easy model deployment

```toml
[llm]
provider = "replicate"
api_key = "${REPLICATE_API_TOKEN}"
text_model = "meta/llama-2-70b-chat"
max_tokens = 4096
temperature = 0.7
webhook_url = "https://your-server.com/webhook"  # Optional
```

### 9. Hugging Face Inference API (Cloud - Free + Paid)

**Best for**: Open models, experimentation, research

- **Type**: Cloud API
- **Models**: 350,000+ models
- **Cost**: Free tier, then pay-per-request
- **Rate Limits**: 1,000 requests/day (free)
- **Features**: Model deployment, serverless

```toml
[llm]
provider = "huggingface"
api_key = "${HUGGINGFACE_API_KEY}"
base_url = "https://api-inference.huggingface.co"
text_model = "meta-llama/Meta-Llama-3-70B-Instruct"
embedding_model = "sentence-transformers/all-MiniLM-L6-v2"
max_tokens = 2048
temperature = 0.7
```

### 10. Cohere (Cloud - Free Tier + Paid)

**Best for**: Embeddings, RAG, enterprise search

- **Type**: Cloud API
- **Models**: Command, Embed (best embeddings)
- **Cost**: Free tier, then pay-per-token
- **Rate Limits**: 100 calls/min (trial)
- **Features**: Best-in-class embeddings

```toml
[llm]
provider = "cohere"
api_key = "${COHERE_API_KEY}"
base_url = "https://api.cohere.ai/v1"
text_model = "command-r-plus"
embedding_model = "embed-english-v3.0"
max_tokens = 4096
temperature = 0.7
```

## Multi-Provider Configuration

Configure multiple providers with automatic fallback:

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
ollama_url = "http://localhost:11434"
text_model = "llama3.2"
priority = 3

# Fallback strategy
[llm.fallback]
enabled = true
max_retries = 3
retry_delay_ms = 1000
```

## Provider Selection Strategies

### 1. Primary with Fallback

```toml
[llm]
strategy = "fallback"
# Tries providers in priority order
```

### 2. Round Robin

```toml
[llm]
strategy = "round-robin"
# Distributes load across providers
```

### 3. Cost-Based

```toml
[llm]
strategy = "cost-optimized"
# Uses cheapest provider first
```

### 4. Task-Based Routing

```toml
[llm.routing]
# Route different tasks to different providers
code_generation = "openai:gpt-4"
embeddings = "cohere:embed-english-v3.0"
chat = "anthropic:claude-3-sonnet"
summarization = "groq:llama3-70b"
```

## Cost Comparison

| Provider | Input (per 1M tokens) | Output (per 1M tokens) | Embeddings (per 1M) |
|----------|----------------------|------------------------|---------------------|
| Ollama | $0 (hardware) | $0 (hardware) | $0 (hardware) |
| OpenAI GPT-4 Turbo | $10 | $30 | $0.13 |
| OpenAI GPT-3.5 | $0.50 | $1.50 | $0.13 |
| Anthropic Claude 3 Opus | $15 | $75 | N/A |
| Anthropic Claude 3 Sonnet | $3 | $15 | N/A |
| Anthropic Claude 3 Haiku | $0.25 | $1.25 | N/A |
| Google Gemini Pro | $0.50 | $1.50 | Free |
| Groq | Free tier | Free tier | N/A |
| Together AI | $0.60 | $0.60 | $0.02 |
| Cohere | $0.50 | $1.50 | $0.10 |

## Environment Variables

Store API keys securely in environment variables:

```bash
# .env file
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GOOGLE_API_KEY="AIza..."
export AZURE_OPENAI_API_KEY="..."
export GROQ_API_KEY="gsk_..."
export TOGETHER_API_KEY="..."
export REPLICATE_API_TOKEN="r8_..."
export HUGGINGFACE_API_KEY="hf_..."
export COHERE_API_KEY="..."
```

Load in your shell:

```bash
source .env
```

## Provider Capabilities Matrix

| Feature | Ollama | OpenAI | Anthropic | Google | Azure | Groq | Together | Others |
|---------|--------|--------|-----------|--------|-------|------|----------|--------|
| Chat Completion | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Streaming | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Embeddings | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ |
| Function Calling | ⚠️ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ |
| Vision | ⚠️ | ✅ | ✅ | ✅ | ✅ | ❌ | ⚠️ | ⚠️ |
| JSON Mode | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Fine-tuning | ⚠️ | ✅ | ❌ | ⚠️ | ✅ | ❌ | ✅ | ⚠️ |
| Offline | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

✅ = Fully Supported, ⚠️ = Partially/Some models, ❌ = Not Supported

## Best Practices

### 1. Start with Ollama for Development

- Free, fast iteration
- No API costs during development
- Easy to test and debug

### 2. Use Multiple Providers in Production

- Configure fallbacks for reliability
- Route tasks to best provider
- Monitor costs and performance

### 3. Secure API Keys

- Never commit keys to git
- Use environment variables
- Rotate keys regularly
- Use secrets management in production

### 4. Monitor Usage and Costs

- Track token usage per provider
- Set up billing alerts
- Review costs monthly
- Optimize model selection

### 5. Test Provider Switching

- Ensure your code works with multiple providers
- Test fallback scenarios
- Validate output quality across providers

## Getting Started

1. **Choose your primary provider** based on your needs
2. **Configure in `config.toml`** with appropriate settings
3. **Set environment variables** for API keys
4. **Test the connection** with simple requests
5. **Add fallback providers** for reliability
6. **Monitor and optimize** usage over time

## Further Reading

- [Configuration Guide](../README.md#configuration)
- [API Documentation](API_DOCUMENTATION.md)
- [Provider Implementation Details](../src/llm/providers/)
