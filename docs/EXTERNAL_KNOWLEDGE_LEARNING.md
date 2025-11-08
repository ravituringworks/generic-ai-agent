# External Knowledge Learning System

## Overview

Enable agents to learn from web content, books, PDFs, documentation, and other external sources by ingesting, processing, consolidating, and storing knowledge in a retrievable format.

---

## Current Capabilities

### ✅ Already Available

1. **MCP Tool Integration** (`src/mcp.rs`)
   - Can integrate external tools via MCP servers
   - HTTP, WebSocket, and stdio transports supported

2. **Vector Store & Embeddings** (`src/memory.rs`)
   - SQLite-based vector storage
   - Semantic similarity search
   - Metadata tagging support

3. **PDF RAG Example** (`examples/pdf_rag_with_tables.rs`)
   - PDF text extraction
   - Table parsing
   - Embedding generation
   - Question answering

### ❌ What's Missing

1. **Web scraping/browsing tools**
2. **Document format parsers** (EPUB, DOCX, etc.)
3. **Source attribution system**
4. **Knowledge consolidation** (deduplication, synthesis)
5. **Automatic knowledge quality assessment**
6. **Configurable limits** (not just hardcoded 100)

---

## Architecture: External Knowledge Ingestion

### 1. Knowledge Source Connectors

```rust
pub enum KnowledgeSource {
    Web {
        url: String,
        crawl_depth: u32,
        allowed_domains: Vec<String>,
    },
    Document {
        path: String,
        format: DocumentFormat,
    },
    Api {
        endpoint: String,
        auth: Option<String>,
    },
    Git {
        repo_url: String,
        branch: String,
        file_patterns: Vec<String>,
    },
    RSS {
        feed_url: String,
        categories: Vec<String>,
    },
}

pub enum DocumentFormat {
    PDF,
    EPUB,
    Markdown,
    HTML,
    DOCX,
    TXT,
    Code { language: String },
}
```

### 2. Content Processing Pipeline

```rust
pub struct KnowledgeIngestionPipeline {
    // Stage 1: Fetch
    fetcher: Box<dyn ContentFetcher>,
    
    // Stage 2: Parse & Extract
    parser: Box<dyn ContentParser>,
    
    // Stage 3: Chunk
    chunker: ContentChunker,
    
    // Stage 4: Embed
    embedder: Box<dyn LlmClient>,
    
    // Stage 5: Quality Assessment
    quality_filter: QualityFilter,
    
    // Stage 6: Store
    store: Arc<RwLock<Box<dyn MemoryStore>>>,
}

impl KnowledgeIngestionPipeline {
    pub async fn ingest(
        &mut self,
        source: KnowledgeSource,
        config: IngestionConfig,
    ) -> Result<IngestionResult> {
        // 1. Fetch content
        let raw_content = self.fetcher.fetch(&source).await?;
        
        // 2. Parse and extract structured content
        let parsed = self.parser.parse(raw_content).await?;
        
        // 3. Chunk into manageable pieces
        let chunks = self.chunker.chunk(parsed, config.chunk_size)?;
        
        // 4. Filter low-quality content
        let filtered_chunks = self.quality_filter.filter(chunks).await?;
        
        // 5. Generate embeddings
        let embedded_chunks = self.embed_chunks(filtered_chunks).await?;
        
        // 6. Store with metadata
        self.store_chunks(embedded_chunks, &source).await?;
        
        Ok(IngestionResult {
            source: source.to_string(),
            chunks_stored: embedded_chunks.len(),
            timestamp: chrono::Utc::now(),
        })
    }
}
```

### 3. Web Browsing Integration (via MCP)

**Using Browser MCP Server**:

```toml
# Add to config.toml
[mcp.servers.browser]
transport = "stdio"
command = ["npx", "-y", "@modelcontextprotocol/server-puppeteer"]
enabled = true

[mcp.servers.brave_search]
transport = "http"
url = "https://api.brave.com/mcp"
auth_token = "${BRAVE_API_KEY}"
enabled = true
```

**Agent Tool Usage**:

```rust
// Agent can now use these tools:
// - browser_navigate(url)
// - browser_screenshot()
// - browser_extract_content()
// - brave_search(query)

let search_results = mcp.call_tool(ToolCall {
    name: "brave_search".to_string(),
    arguments: json!({
        "query": "best practices for Rust async programming",
        "count": 10
    }),
}).await?;

// Extract URLs and content
for result in search_results {
    let content = mcp.call_tool(ToolCall {
        name: "browser_extract_content".to_string(),
        arguments: json!({ "url": result.url }),
    }).await?;
    
    // Ingest content into knowledge base
    ingest_external_knowledge(content, result.url).await?;
}
```

### 4. Document Parsers

```rust
pub trait ContentParser: Send + Sync {
    async fn parse(&self, content: RawContent) -> Result<ParsedContent>;
}

pub struct MultiFormatParser {
    parsers: HashMap<DocumentFormat, Box<dyn ContentParser>>,
}

impl MultiFormatParser {
    pub fn with_defaults() -> Self {
        let mut parsers = HashMap::new();
        
        // PDF Parser (using pdf-extract or lopdf)
        parsers.insert(DocumentFormat::PDF, Box::new(PdfParser::new()));
        
        // EPUB Parser
        parsers.insert(DocumentFormat::EPUB, Box::new(EpubParser::new()));
        
        // Markdown Parser
        parsers.insert(DocumentFormat::Markdown, Box::new(MarkdownParser::new()));
        
        // HTML Parser (using scraper or html5ever)
        parsers.insert(DocumentFormat::HTML, Box::new(HtmlParser::new()));
        
        // DOCX Parser (using docx-rs)
        parsers.insert(DocumentFormat::DOCX, Box::new(DocxParser::new()));
        
        // Code Parser (using tree-sitter)
        parsers.insert(
            DocumentFormat::Code { language: "rust".to_string() },
            Box::new(CodeParser::new("rust"))
        );
        
        Self { parsers }
    }
}
```

### 5. Knowledge Consolidation System

**Problem**: Ingesting multiple sources can lead to:

- Duplicate information
- Contradictory advice
- Information overload

**Solution**: Consolidation Pipeline

```rust
pub struct KnowledgeConsolidator {
    deduplicator: SemanticDeduplicator,
    synthesizer: KnowledgeSynthesizer,
    quality_ranker: QualityRanker,
}

impl KnowledgeConsolidator {
    /// Consolidate multiple knowledge chunks on the same topic
    pub async fn consolidate(
        &self,
        topic: &str,
        chunks: Vec<KnowledgeChunk>,
    ) -> Result<ConsolidatedKnowledge> {
        // 1. Find semantic duplicates (similarity > 0.9)
        let deduplicated = self.deduplicator.deduplicate(chunks).await?;
        
        // 2. Rank by quality (source credibility, recency, detail)
        let ranked = self.quality_ranker.rank(deduplicated).await?;
        
        // 3. Synthesize into coherent knowledge
        let synthesized = self.synthesizer.synthesize(topic, ranked).await?;
        
        Ok(synthesized)
    }
}

pub struct ConsolidatedKnowledge {
    topic: String,
    summary: String,
    key_points: Vec<String>,
    best_practices: Vec<BestPractice>,
    sources: Vec<SourceAttribution>,
    confidence_score: f32,
    last_updated: DateTime<Utc>,
}
```

**Synthesis Using LLM**:

```rust
async fn synthesize_knowledge(
    &self,
    topic: &str,
    chunks: Vec<KnowledgeChunk>,
) -> Result<String> {
    let prompt = format!(
        "Synthesize knowledge on: {}\n\n\
        From {} sources:\n{}\n\n\
        Create a consolidated summary that:\n\
        1. Identifies consensus points\n\
        2. Notes contradictions with reasoning\n\
        3. Ranks recommendations by evidence strength\n\
        4. Provides actionable takeaways",
        topic,
        chunks.len(),
        chunks.iter()
            .map(|c| format!("- {}: {}", c.source, c.content.chars().take(200).collect::<String>()))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    self.llm.generate(&[user_message(&prompt)]).await
        .map(|r| r.text)
}
```

---

## Adaptive Best Practices Limits

### Current Limitation

```rust
// Hardcoded in config
max_best_practices_per_role = 100
```

### Enhanced Approach: Dynamic Limits with Pruning

```rust
pub struct AdaptiveKnowledgeManager {
    config: KnowledgeManagementConfig,
}

pub struct KnowledgeManagementConfig {
    // Soft limits (triggers consolidation)
    soft_limit_best_practices: usize,
    
    // Hard limits (triggers aggressive pruning)
    hard_limit_best_practices: usize,
    
    // Retention policies
    min_reuse_count: u32,          // Keep if used >= N times
    min_quality_score: f32,        // Keep if quality >= threshold
    max_age_days: i64,             // Prune if older than N days AND unused
    
    // Consolidation strategy
    enable_auto_consolidation: bool,
    consolidation_similarity_threshold: f32,
}

impl Default for KnowledgeManagementConfig {
    fn default() -> Self {
        Self {
            soft_limit_best_practices: 100,
            hard_limit_best_practices: 500,
            min_reuse_count: 2,
            min_quality_score: 0.7,
            max_age_days: 90,
            enable_auto_consolidation: true,
            consolidation_similarity_threshold: 0.85,
        }
    }
}
```

**Automatic Pruning Strategy**:

```rust
impl AdaptiveKnowledgeManager {
    pub async fn manage_best_practices(
        &self,
        role: &AgentRole,
        store: &mut Box<dyn MemoryStore>,
    ) -> Result<ManagementResult> {
        let practices = self.get_best_practices(role, store).await?;
        let count = practices.len();
        
        if count < self.config.soft_limit_best_practices {
            return Ok(ManagementResult::NoActionNeeded);
        }
        
        if count >= self.config.hard_limit_best_practices {
            // Aggressive pruning
            self.prune_aggressively(role, store, practices).await?;
        } else if count >= self.config.soft_limit_best_practices {
            // Consolidation
            if self.config.enable_auto_consolidation {
                self.consolidate_practices(role, store, practices).await?;
            }
        }
        
        Ok(ManagementResult::Managed)
    }
    
    async fn prune_aggressively(
        &self,
        role: &AgentRole,
        store: &mut Box<dyn MemoryStore>,
        practices: Vec<BestPractice>,
    ) -> Result<()> {
        // Sort by composite score
        let mut scored_practices: Vec<_> = practices
            .into_iter()
            .map(|p| {
                let score = self.calculate_retention_score(&p);
                (score, p)
            })
            .collect();
        
        scored_practices.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Keep top N based on soft limit
        let to_keep = &scored_practices[..self.config.soft_limit_best_practices];
        let to_prune = &scored_practices[self.config.soft_limit_best_practices..];
        
        // Delete low-scoring practices
        for (_, practice) in to_prune {
            store.delete(practice.id).await?;
        }
        
        Ok(())
    }
    
    fn calculate_retention_score(&self, practice: &BestPractice) -> f32 {
        let recency_score = self.calculate_recency_score(practice.last_used);
        let usage_score = (practice.reuse_count as f32 / 10.0).min(1.0);
        let quality_score = practice.avg_quality_score;
        
        // Weighted average
        (recency_score * 0.2) + (usage_score * 0.4) + (quality_score * 0.4)
    }
}
```

---

## Configuration Updates

```toml
[learning]
# Adaptive limits
soft_limit_best_practices = 100
hard_limit_best_practices = 500
enable_auto_consolidation = true
consolidation_similarity_threshold = 0.85

# Retention policies
min_reuse_count_to_keep = 2
min_quality_score_to_keep = 0.7
max_age_days_if_unused = 90

# External knowledge ingestion
[learning.external_sources]
enable_web_learning = true
enable_document_ingestion = true
enable_code_learning = true

# Web learning
max_crawl_depth = 2
max_pages_per_domain = 50
allowed_domains = [
    "docs.rs",
    "rust-lang.org",
    "stackoverflow.com",
    "github.com"
]

# Document ingestion
supported_formats = ["pdf", "epub", "md", "html", "docx", "txt"]
max_document_size_mb = 50
chunk_size = 1000
chunk_overlap = 200

# Quality filtering
min_content_quality_score = 0.6
enable_source_credibility_check = true
trusted_sources = [
    "*.rust-lang.org",
    "docs.rs",
    "*.arxiv.org"
]
```

---

## Usage Examples

### Example 1: Learn from Web Search

```rust
// Agent discovers need to learn about a topic
let agent_needs_knowledge_on = "asynchronous programming patterns in Rust";

// 1. Search the web
let search_tool = ToolCall {
    name: "brave_search".to_string(),
    arguments: json!({
        "query": agent_needs_knowledge_on,
        "count": 10,
        "filter": "technical"
    }),
};

let search_results = agent.mcp.call_tool(search_tool).await?;

// 2. Ingest top results
for result in search_results.take(5) {
    pipeline.ingest(
        KnowledgeSource::Web {
            url: result.url,
            crawl_depth: 1,
            allowed_domains: config.allowed_domains.clone(),
        },
        IngestionConfig::default(),
    ).await?;
}

// 3. Consolidate learnings
let consolidated = consolidator.consolidate(
    agent_needs_knowledge_on,
    retrieved_chunks
).await?;

// 4. Store as best practice
store_as_best_practice(agent.role, consolidated).await?;
```

### Example 2: Learn from Book (PDF/EPUB)

```rust
// Ingest a technical book
pipeline.ingest(
    KnowledgeSource::Document {
        path: "/path/to/rust_async_book.pdf".to_string(),
        format: DocumentFormat::PDF,
    },
    IngestionConfig {
        chunk_size: 1000,
        chunk_overlap: 200,
        quality_threshold: 0.7,
    },
).await?;

// Book content now searchable by agents
```

### Example 3: Learn from Code Repositories

```rust
// Learn from open-source projects
pipeline.ingest(
    KnowledgeSource::Git {
        repo_url: "https://github.com/tokio-rs/tokio".to_string(),
        branch: "master".to_string(),
        file_patterns: vec!["*.rs".to_string()],
    },
    IngestionConfig {
        extract_code_patterns: true,
        extract_documentation: true,
        extract_best_practices: true,
    },
).await?;
```

---

## Answering Your Questions

### Is 100 Best Practices Adequate?

**Short answer**: It depends on complexity, but likely not for long-term learning.

**Analysis**:

- **100 is reasonable** for a single, focused role in a specific domain
- **Not adequate** for:
  - Long-running agents
  - Agents with broad responsibilities
  - Accumulating knowledge from external sources
  - Multi-domain expertise

**Recommended Approach**:

1. **Start with 100** as soft limit (triggers consolidation)
2. **Set 500 as hard limit** (triggers pruning)
3. **Enable auto-consolidation** to merge similar practices
4. **Use quality-based retention** - keep high-value practices
5. **Implement hierarchical knowledge**:
   - General principles (always keep)
   - Domain-specific practices (prune by usage)
   - Tactical tips (aggressively prune)

### Scaling Knowledge Storage

**Storage estimates**:

```text
100 best practices × 500 bytes avg = 50 KB
500 best practices × 500 bytes avg = 250 KB
10,000 knowledge chunks × 1 KB = 10 MB
100,000 knowledge chunks × 1 KB = 100 MB
```

Even with 100,000 chunks, storage is manageable. The real challenge is:

- **Retrieval speed** (solved with vector indexes)
- **Knowledge quality** (solved with consolidation)
- **Relevance** (solved with semantic search)

---

## Implementation Priorities

### Phase 1: Foundation (Week 1-2)

- [ ] Add MCP browser/search servers
- [ ] Implement basic web content fetching
- [ ] Add HTML/Markdown parsers
- [ ] Create chunking system

### Phase 2: Multi-Format Support (Week 3-4)

- [ ] PDF parser integration
- [ ] EPUB support
- [ ] DOCX support
- [ ] Code parser (tree-sitter)

### Phase 3: Quality & Consolidation (Week 5-6)

- [ ] Quality filtering system
- [ ] Deduplication logic
- [ ] Knowledge synthesis via LLM
- [ ] Source attribution

### Phase 4: Adaptive Management (Week 7-8)

- [ ] Dynamic limit system
- [ ] Pruning strategies
- [ ] Auto-consolidation
- [ ] Retention scoring

### Phase 5: Agent Integration (Week 9-10)

- [ ] Autonomous web learning
- [ ] Document discovery
- [ ] Knowledge gap detection
- [ ] Learning goal setting

---

## Conclusion

**Yes, agents can learn from external sources** - the framework already supports it through MCP and memory systems. You just need to add:

1. Content fetchers (web, docs)
2. Format parsers
3. Quality filters
4. Consolidation logic

**No, 100 best practices is not a hard limit** - it should be:

- **Soft limit** triggering consolidation
- **Dynamic** based on usage and quality
- **Managed automatically** with pruning policies
- **Scalable** to thousands of knowledge chunks

The key insight: **Store raw knowledge chunks indefinitely, but consolidate into actionable best practices on demand.**
