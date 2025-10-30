# External Knowledge Learning - Usage Guide

This guide demonstrates how to use the external knowledge learning system to enable agents to fetch, process, and learn from web sources, documents, and code repositories.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Architecture](#architecture)
4. [Configuration](#configuration)
5. [Fetching Web Content](#fetching-web-content)
6. [Content Processing](#content-processing)
7. [Knowledge Management](#knowledge-management)
8. [Integration with Agents](#integration-with-agents)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

---

## Overview

The external knowledge learning system consists of four main components:

1. **WebFetcher**: Fetches content from URLs using MCP browser/HTTP tools
2. **ContentChunker**: Splits large content into manageable, contextual chunks
3. **KnowledgeConsolidator**: Deduplicates and synthesizes knowledge
4. **AdaptiveKnowledgeManager**: Manages knowledge lifecycle with adaptive limits

### Key Features

- ✅ Fetch web content via MCP (browser automation or HTTP)
- ✅ Smart chunking with overlap for context preservation
- ✅ Deduplication and consolidation of similar knowledge
- ✅ Adaptive retention with soft (100) and hard (500) limits
- ✅ Quality-based pruning using usage and recency metrics
- ✅ Support for text, markdown, and code content types

---

## Quick Start

### 1. Configure MCP Browser Tools

Add an MCP server with browser/fetch tools to your `config.toml`:

```toml
[mcp]
enabled = true
default_timeout = 30
max_concurrent_calls = 5

[mcp.servers.browser]
enabled = true
transport = "http"
url = "http://localhost:3000/mcp"
# Or use stdio for local tools
# transport = "stdio"
# command = ["npx", "-y", "@modelcontextprotocol/server-puppeteer"]
```

### 2. Initialize Components

```rust
use the_agency::{
    knowledge::{
        ContentChunker, FetcherConfig, WebFetcher,
        AdaptiveKnowledgeManager, KnowledgeConsolidator,
    },
    Agent, AgentBuilder, AgentConfig,
};

// Create MCP client for web fetching
let mut mcp_client = McpClient::new(config.mcp.clone());
mcp_client.add_server("browser".to_string(), browser_config).await?;

// Create fetcher
let fetcher = WebFetcher::new(FetcherConfig::default());

// Create chunker
let chunker = ContentChunker::default();

// Create consolidator
let consolidator = KnowledgeConsolidator::default();

// Create knowledge manager
let manager = AdaptiveKnowledgeManager::new(config.learning);
```

### 3. Fetch and Process Content

```rust
// Fetch content from URL
let content = fetcher
    .fetch_url(&mcp_client, "https://example.com/article")
    .await?;

// Chunk content
let chunks = chunker.chunk_markdown(
    &content.content,
    content.url.clone(),
    "markdown".to_string(),
);

println!("Fetched and chunked: {} chunks", chunks.len());
```

### 4. Store in Agent Memory

```rust
// Convert chunks to memory entries and store
for chunk in chunks {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("role".to_string(), agent.role().to_string());
    metadata.insert("source".to_string(), chunk.source.clone());
    metadata.insert("source_type".to_string(), chunk.source_type.clone());
    metadata.insert("quality_score".to_string(), "0.8".to_string());
    metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
    
    agent.store_knowledge(chunk.content, metadata).await?;
}
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      External Sources                            │
│  Web Pages │ Documents │ Code Repos │ RSS Feeds │ APIs          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                        WebFetcher                                │
│  • Browser automation (Puppeteer, Playwright)                   │
│  • HTTP fetch tools                                              │
│  • Content extraction & truncation                               │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      ContentChunker                              │
│  • Text chunking with overlap                                    │
│  • Markdown-aware splitting                                      │
│  • Code function boundary preservation                           │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   KnowledgeConsolidator                          │
│  • Similarity-based deduplication                                │
│  • Synthesis of related knowledge                                │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                 AdaptiveKnowledgeManager                         │
│  • Retention scoring (quality, usage, recency)                   │
│  • Adaptive pruning (soft/hard limits)                           │
│  • Per-role knowledge stats                                      │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      Vector Memory Store                         │
│  • Embedded knowledge chunks                                     │
│  • Semantic retrieval for agent queries                          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Configuration

### Learning Configuration

In `config.toml`:

```toml
[learning]
enabled = true

# Adaptive knowledge limits
soft_limit_best_practices = 100   # Trigger consolidation
hard_limit_best_practices = 500   # Force pruning

# Retention policies
min_reuse_count = 2               # Keep if reused at least twice
min_quality_score = 0.5           # Keep if quality >= 0.5
max_age_days = 90                 # Prune if older than 90 days

# Ingestion settings
[learning.ingestion]
chunk_size = 1024                 # Characters per chunk
chunk_overlap = 100               # Overlap for context
max_chunks = 50                   # Max chunks per document

# External sources
[[learning.external_sources]]
type = "web"
url = "https://docs.example.com"
enabled = true
fetch_interval = 86400            # Daily (seconds)

[[learning.external_sources]]
type = "rss"
url = "https://blog.example.com/rss"
enabled = true
fetch_interval = 3600             # Hourly
```

### Fetcher Configuration

```rust
use the_agency::knowledge::FetcherConfig;

let fetcher_config = FetcherConfig {
    max_content_size: 1024 * 1024,  // 1MB
    user_agent: "the-agency/0.2.0".to_string(),
    timeout: 30,
    extract_main_content: true,
    follow_redirects: true,
};
```

---

## Fetching Web Content

### Using Browser Tools

Browser tools provide JavaScript execution and can handle dynamic content:

```rust
// Browser tools automatically selected if available
let content = fetcher
    .fetch_url(&mcp_client, "https://example.com/dynamic-page")
    .await?;

println!("Fetched: {} bytes", content.content.len());
println!("Title: {:?}", content.title);
```

The fetcher tries these browser tools in order:
1. `browser_navigate` (generic)
2. `puppeteer_fetch` (Puppeteer-specific)
3. `playwright_fetch` (Playwright-specific)

### Using HTTP Tools

For static content, HTTP tools are faster:

```rust
// HTTP tools used as fallback or for static content
let content = fetcher
    .fetch_url(&mcp_client, "https://api.example.com/docs")
    .await?;
```

The fetcher tries these HTTP tools:
1. `fetch` (generic HTTP)
2. `http_get` (GET request)
3. `curl` (cURL wrapper)

### Batch Fetching

Fetch multiple URLs concurrently:

```rust
let urls = vec![
    "https://example.com/page1".to_string(),
    "https://example.com/page2".to_string(),
    "https://example.com/page3".to_string(),
];

let results = fetcher.fetch_urls(&mcp_client, urls).await;

for result in results {
    match result {
        Ok(content) => println!("Fetched: {}", content.url),
        Err(e) => eprintln!("Failed: {}", e),
    }
}
```

---

## Content Processing

### Text Chunking

Split plain text with sentence-aware boundaries:

```rust
let chunker = ContentChunker::default();
let chunks = chunker.chunk_text(
    &long_text,
    "https://source.com/doc".to_string(),
    "text".to_string(),
);
```

### Markdown Chunking

Preserve markdown structure (headers, lists):

```rust
let chunks = chunker.chunk_markdown(
    &markdown_content,
    "https://docs.example.com".to_string(),
    "markdown".to_string(),
);

// Chunks respect header boundaries
for chunk in chunks {
    println!("Chunk: {} chars", chunk.content.len());
}
```

### Code Chunking

Preserve function/class boundaries:

```rust
let chunks = chunker.chunk_code(
    &rust_code,
    "rust",
    "https://github.com/user/repo/file.rs".to_string(),
);
```

### Deduplication

Remove similar or duplicate knowledge:

```rust
use the_agency::knowledge::deduplicate_knowledge;

let unique_chunks = deduplicate_knowledge(chunks, 0.85); // 85% similarity threshold
println!("Deduplicated: {} → {} chunks", chunks.len(), unique_chunks.len());
```

### Synthesis

Merge related knowledge chunks:

```rust
use the_agency::knowledge::synthesize_knowledge;

let synthesized = synthesize_knowledge(chunks);
println!("Synthesized: {} → {} entries", chunks.len(), synthesized.len());
```

---

## Knowledge Management

### Adaptive Limits

The system automatically manages knowledge growth:

```rust
let manager = AdaptiveKnowledgeManager::new(learning_config);

// Check if consolidation needed
let stats = manager.get_stats("engineer", &memory_store).await?;

if stats.is_over_limit() {
    println!("Soft limit reached: consolidation recommended");
}

if stats.needs_pruning() {
    println!("Hard limit reached: pruning required");
}
```

### Pruning Strategy

Prune low-value knowledge when limits are reached:

```rust
let result = manager
    .prune_if_needed("engineer", &memory_store)
    .await?;

match result {
    ManagementResult::Pruned { removed_count, remaining_count } => {
        println!("Pruned {} entries, {} remaining", removed_count, remaining_count);
    }
    ManagementResult::NoActionNeeded { current_count } => {
        println!("No pruning needed, {} entries", current_count);
    }
    _ => {}
}
```

### Retention Scoring

Knowledge is scored based on:

1. **Usage** (40%): How often it's retrieved
2. **Quality** (30%): Initial quality score
3. **Recency** (30%): How recently created/updated

```rust
// Metadata for high-retention knowledge
let mut metadata = HashMap::new();
metadata.insert("quality_score".to_string(), "0.9".to_string());
metadata.insert("reuse_count".to_string(), "10".to_string());
metadata.insert("timestamp".to_string(), Utc::now().to_rfc3339());
```

---

## Integration with Agents

### Learning Loop

Implement a continuous learning loop:

```rust
async fn learning_loop(
    agent: &mut Agent,
    sources: Vec<ExternalSource>,
    mcp_client: &McpClient,
) -> Result<()> {
    let fetcher = WebFetcher::new(FetcherConfig::default());
    let chunker = ContentChunker::default();
    let manager = AdaptiveKnowledgeManager::new(agent.config().learning.clone());
    
    for source in sources {
        // Fetch content
        let content = fetcher.fetch_url(mcp_client, &source.url).await?;
        
        // Chunk and store
        let chunks = chunker.chunk_markdown(
            &content.content,
            content.url,
            "markdown".to_string(),
        );
        
        for chunk in chunks {
            agent.store_knowledge(chunk.content, create_metadata(&chunk)).await?;
        }
        
        // Manage knowledge size
        manager.prune_if_needed(agent.role(), agent.memory_store()).await?;
    }
    
    Ok(())
}
```

### Query-Time Learning

Learn just-in-time for specific queries:

```rust
async fn learn_and_answer(
    agent: &mut Agent,
    query: &str,
    search_urls: Vec<String>,
) -> Result<String> {
    // Search external sources
    let fetcher = WebFetcher::new(FetcherConfig::default());
    let results = fetcher.fetch_urls(&mcp_client, search_urls).await;
    
    // Store relevant knowledge
    for result in results {
        if let Ok(content) = result {
            // Filter relevance before storing
            if is_relevant(&content, query) {
                agent.learn_from_content(&content).await?;
            }
        }
    }
    
    // Now answer with new knowledge
    agent.process(query).await
}
```

---

## Best Practices

### 1. Source Quality

✅ **DO**: Curate high-quality, authoritative sources
❌ **DON'T**: Ingest random web pages without validation

### 2. Chunking Strategy

✅ **DO**: Use appropriate chunk size for your embedding model
✅ **DO**: Preserve context with overlap (10-20% of chunk size)
❌ **DON'T**: Create chunks smaller than 100 chars

### 3. Deduplication

✅ **DO**: Deduplicate before embedding (saves cost)
✅ **DO**: Use 0.85-0.90 similarity threshold for text
❌ **DON'T**: Skip deduplication with large ingestion volumes

### 4. Knowledge Limits

✅ **DO**: Set soft limit at 100 for focused domains
✅ **DO**: Allow hard limit at 500 for multi-domain agents
✅ **DO**: Monitor utilization regularly

### 5. Retention Policy

✅ **DO**: Keep frequently reused knowledge (reuse_count >= 2)
✅ **DO**: Prune low-quality, unused, old knowledge
❌ **DON'T**: Delete knowledge that's < 7 days old

### 6. MCP Server Setup

✅ **DO**: Use local MCP servers for development
✅ **DO**: Use managed MCP services for production
❌ **DON'T**: Expose unauthenticated MCP endpoints

---

## Troubleshooting

### Issue: No MCP Tools Available

**Error**: `No suitable fetch tool available`

**Solution**:
1. Verify MCP server is running
2. Check `config.toml` has correct MCP server URL
3. List available tools:
   ```rust
   let tools = mcp_client.list_tools();
   println!("Available tools: {:?}", tools);
   ```

### Issue: Content Too Large

**Error**: `Content truncated from X to Y bytes`

**Solution**:
1. Increase `max_content_size` in FetcherConfig
2. Or filter large documents before fetching

### Issue: Too Many Chunks

**Warning**: `Content chunked: 1000+ chunks`

**Solution**:
1. Reduce chunk overlap
2. Increase chunk size
3. Set `max_chunks` in IngestionConfig

### Issue: Memory Growing Too Fast

**Symptom**: Agent memory exceeds hard limit

**Solution**:
1. Lower soft/hard limits in config
2. Increase pruning frequency
3. Improve quality filtering before ingestion

### Issue: Embeddings Failing

**Error**: `Embedding failed: dimension mismatch`

**Solution**:
1. Ensure all chunks use same embedding model
2. Check chunker produces consistent chunk sizes
3. Verify embedding model is loaded correctly

---

## Example: Complete Learning Pipeline

See `examples/collaborative_robotics_workspace.rs` for a full example:

```bash
cargo run --example collaborative_robotics_workspace
```

This example demonstrates:
- Simulated web content fetching
- Content chunking and storage
- Knowledge integration in collaborative agents
- Workspace-based knowledge management

---

## Next Steps

1. **Set up MCP server** with browser tools
2. **Configure external sources** in `config.toml`
3. **Implement learning loop** in your agent
4. **Monitor knowledge stats** regularly
5. **Tune retention policies** based on usage

For more details, see:
- [EXTERNAL_KNOWLEDGE_LEARNING.md](./EXTERNAL_KNOWLEDGE_LEARNING.md) - Design document
- [API Documentation](../src/knowledge/) - Module code documentation
- [MCP Documentation](https://modelcontextprotocol.io/) - MCP protocol details

---

**Questions or issues?** Open an issue on GitHub or check existing documentation.
