# LLM Response Caching

The Agency includes a sophisticated LLM response caching system to improve performance and reduce latency for repeated or similar queries.

## Overview

The caching system stores LLM responses in a SQLite database and automatically retrieves cached responses for identical queries, avoiding redundant API calls to Ollama.

### Key Features

- **SQLite Backend**: Persistent storage with efficient querying
- **Cache Key Generation**: Deterministic hashing based on query parameters
- **TTL (Time-To-Live)**: Automatic expiration of old entries
- **LRU Eviction**: Least Recently Used entries are removed when cache is full
- **Temperature Threshold**: Only caches deterministic queries (low temperature)
- **Hit/Miss Tracking**: Statistics on cache performance
- **Automatic Cleanup**: Expired entries are automatically removed

## Configuration

### Basic Configuration

Add cache configuration to your `config.toml`:

```toml
[llm.cache]
# Enable or disable caching
enabled = true

# Maximum number of cache entries
max_entries = 1000

# Time-to-live for cache entries in seconds (1 hour)
ttl_seconds = 3600

# SQLite database path for cache
db_path = "cache.db"

# Minimum temperature threshold for caching
# Only cache deterministic queries (low temperature)
min_temperature_threshold = 0.3
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | true | Enable/disable caching |
| `max_entries` | usize | 1000 | Maximum number of cached responses |
| `ttl_seconds` | i64 | 3600 | Time-to-live in seconds (1 hour) |
| `db_path` | string | "cache.db" | SQLite database file path |
| `min_temperature_threshold` | f32 | 0.3 | Only cache queries with temperature ≤ this value |

## Usage

### Programmatic Usage

```rust
use the_agency::{AgentConfig, OllamaClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration with cache settings
    let config = AgentConfig::from_file("config.toml")?;
    
    // Create LLM client with cache enabled
    let llm_client = OllamaClient::new_with_cache(config.llm).await?;
    
    // First call - cache miss, calls Ollama
    let messages = vec![
        system_message("You are a helpful assistant"),
        user_message("What is Rust?")
    ];
    let response1 = llm_client.generate(&messages).await?;
    println!("Response: {}", response1.text);
    
    // Second call - cache hit, returns instantly
    let response2 = llm_client.generate(&messages).await?;
    println!("Cached response: {}", response2.text);
    
    Ok(())
}
```

### Cache Management

```rust
use the_agency::{LlmCache, LlmCacheConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = LlmCacheConfig::default();
    let cache = LlmCache::new(config).await?;
    
    // Get cache statistics
    let stats = cache.stats().await?;
    println!("Total entries: {}", stats.total_entries);
    println!("Total hits: {}", stats.total_hits);
    println!("Average age: {}s", stats.avg_age_seconds);
    
    // Clean up expired entries
    let removed = cache.cleanup_expired().await?;
    println!("Removed {} expired entries", removed);
    
    // Clear all cache entries
    cache.clear().await?;
    
    // Invalidate specific entry
    let cache_key = "some_key";
    cache.invalidate(cache_key).await?;
    
    Ok(())
}
```

## How It Works

### Cache Key Generation

The cache key is computed from:
- Query messages (serialized)
- Model name
- Temperature
- Max tokens
- System prompt (if present)

This ensures that identical queries with the same parameters return the same cache key.

### Cache Behavior

**Cached Queries:**
- ✅ Queries with temperature ≤ 0.3 (deterministic)
- ✅ Identical messages and parameters
- ✅ Within TTL period

**Not Cached:**
- ❌ High temperature queries (> 0.3) - too creative/random
- ❌ Expired entries (older than TTL)
- ❌ When cache is disabled

### Eviction Strategy

When the cache reaches `max_entries`:
1. Identifies least recently accessed entries
2. Removes oldest entries to make room
3. Logs eviction count

## Performance Impact

### Benefits

- **Reduced Latency**: Instant response for cached queries (< 1ms vs 100-1000ms)
- **Lower Load**: Reduces Ollama API calls
- **Improved UX**: Faster responses for repeated questions
- **Cost Savings**: Fewer compute resources used

### Trade-offs

- **Storage**: Uses disk space for cache database
- **Memory**: Small memory overhead for cache management
- **Consistency**: Cached responses may become stale

## Best Practices

### When to Use Caching

✅ **Good Use Cases:**
- FAQ/documentation queries
- Repeated system prompts
- Deterministic tasks (code analysis, fact checking)
- Development/testing with repeated queries

❌ **Avoid Caching For:**
- Creative writing (high temperature)
- Time-sensitive information
- User-specific personalized responses
- One-time queries

### Recommended Settings

**Development Environment:**
```toml
[llm.cache]
enabled = true
max_entries = 500
ttl_seconds = 1800  # 30 minutes
min_temperature_threshold = 0.3
```

**Production Environment:**
```toml
[llm.cache]
enabled = true
max_entries = 5000
ttl_seconds = 7200  # 2 hours
min_temperature_threshold = 0.2  # More conservative
```

**Testing Environment:**
```toml
[llm.cache]
enabled = true
max_entries = 100
ttl_seconds = 300  # 5 minutes
min_temperature_threshold = 0.5
```

## Monitoring

### Cache Statistics

Monitor cache performance:

```rust
let stats = cache.stats().await?;
let hit_rate = stats.total_hits as f64 / (stats.total_entries as f64 + 1.0);
println!("Cache hit rate: {:.2}%", hit_rate * 100.0);
```

### Logging

Enable debug logging to see cache activity:

```bash
RUST_LOG=the_agency::cache=debug cargo run
```

You'll see logs like:
```
DEBUG Cache hit! (model: llama3.2, temp: 0.20, hits: 5)
DEBUG Cache miss for key: abc123def456
DEBUG Cached response (model: llama3.2, temp: 0.20)
INFO Evicted 10 old cache entries (LRU)
```

## Troubleshooting

### Cache Not Working

1. **Check if enabled**: Verify `enabled = true` in config
2. **Check temperature**: Queries with high temperature won't be cached
3. **Check logs**: Enable debug logging to see cache activity
4. **Verify database**: Ensure cache.db file exists and is writable

### Cache Too Large

Reduce `max_entries` or decrease `ttl_seconds`:

```toml
max_entries = 500
ttl_seconds = 1800
```

### Stale Responses

Reduce TTL or manually clear cache:

```rust
cache.clear().await?;
```

### Database Errors

Ensure proper permissions and disk space:

```bash
ls -lah cache.db
rm cache.db  # Delete and recreate if corrupted
```

## Advanced Usage

### Custom Cache Implementation

You can create a custom cache with specific settings:

```rust
use the_agency::{LlmCache, LlmCacheConfig};

let config = LlmCacheConfig {
    enabled: true,
    max_entries: 2000,
    ttl_seconds: 14400,  // 4 hours
    db_path: "custom_cache.db".to_string(),
    min_temperature_threshold: 0.1,  // Very conservative
};

let cache = LlmCache::new(config).await?;
```

### Periodic Cleanup

Set up automatic cleanup:

```rust
use tokio::time::{interval, Duration};

tokio::spawn(async move {
    let mut cleanup_timer = interval(Duration::from_secs(3600));
    loop {
        cleanup_timer.tick().await;
        if let Ok(removed) = cache.cleanup_expired().await {
            info!("Cleaned up {} expired entries", removed);
        }
    }
});
```

## See Also

- [Task-Based LLM Configuration](TASK_BASED_LLM.md)
- [Configuration Guide](../README.md#configuration)
- [API Documentation](https://docs.rs/the_agency)
