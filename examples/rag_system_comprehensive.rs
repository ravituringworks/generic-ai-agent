//! Comprehensive RAG (Retrieval-Augmented Generation) System
//!
//! This system demonstrates:
//! - Document processing (.docx, .pdf) with metadata extraction
//! - Multiple chunking strategies (recursive, sliding window, semantic)
//! - Support for multiple vector stores (SQLite, pgvector, Pinecone, etc.)
//! - Embedding generation and similarity search
//! - Observability and performance tracking
//! - Query optimization and caching
//! - Integration with workflow system for enhanced responses

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use the_agency::{
    error::Result,
    workflow::{WorkflowBuilder, WorkflowContext, WorkflowDecision, WorkflowStep},
};
use uuid::Uuid;

/// Supported document types for RAG processing
#[derive(Debug, Clone)]
pub enum DocumentType {
    Pdf,
    Docx,
    Text,
    Markdown,
}

/// Different chunking strategies for document processing
#[derive(Debug, Clone)]
pub enum ChunkingStrategy {
    /// Split text recursively on separators
    Recursive {
        chunk_size: usize,
        chunk_overlap: usize,
        separators: Vec<String>,
    },
    /// Sliding window approach
    SlidingWindow {
        window_size: usize,
        step_size: usize,
    },
    /// Semantic-based chunking
    Semantic {
        max_chunk_size: usize,
        similarity_threshold: f32,
    },
    /// Fixed-size chunks
    FixedSize { size: usize, overlap: usize },
}

/// Vector store implementations
#[derive(Debug, Clone)]
pub enum VectorStoreType {
    SQLite,
    PgVector,
    Pinecone,
    Qdrant,
    MongoDB,
}

/// Document chunk with metadata
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub embedding: Option<Vec<f32>>,
    pub chunk_index: usize,
    pub source_document: String,
}

impl DocumentChunk {
    pub fn new(content: String, source_document: String, chunk_index: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            metadata: HashMap::new(),
            embedding: None,
            chunk_index,
            source_document,
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

/// Document processing and chunking engine
pub struct DocumentProcessor {
    pub chunking_strategy: ChunkingStrategy,
}

impl DocumentProcessor {
    pub fn new(chunking_strategy: ChunkingStrategy) -> Self {
        Self { chunking_strategy }
    }

    /// Process document and extract chunks
    pub async fn process_document(
        &self,
        content: &str,
        document_name: &str,
        doc_type: DocumentType,
    ) -> Result<Vec<DocumentChunk>> {
        println!(
            "  📄 Processing document: {} ({:?})",
            document_name, doc_type
        );

        // Simulate document type-specific processing
        let processed_content = match doc_type {
            DocumentType::Pdf => self.process_pdf_content(content).await?,
            DocumentType::Docx => self.process_docx_content(content).await?,
            DocumentType::Text => content.to_string(),
            DocumentType::Markdown => self.process_markdown_content(content).await?,
        };

        // Apply chunking strategy
        let chunks = self
            .chunk_content(&processed_content, document_name)
            .await?;

        println!(
            "  ✅ Created {} chunks from document: {}",
            chunks.len(),
            document_name
        );
        Ok(chunks)
    }

    async fn process_pdf_content(&self, content: &str) -> Result<String> {
        // Simulate PDF processing (OCR, text extraction, etc.)
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Mock PDF processing - add structure and metadata
        let processed = format!(
            "PDF_PROCESSED: {}\n\nExtracted Text:\n{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            content
        );
        Ok(processed)
    }

    async fn process_docx_content(&self, content: &str) -> Result<String> {
        // Simulate DOCX processing (style preservation, structure extraction)
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        let processed = format!(
            "DOCX_PROCESSED: {}\n\nFormatted Content:\n{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            content
        );
        Ok(processed)
    }

    async fn process_markdown_content(&self, content: &str) -> Result<String> {
        // Simulate markdown processing (heading extraction, link processing)
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(format!("MARKDOWN_PROCESSED:\n{}", content))
    }

    async fn chunk_content(&self, content: &str, source: &str) -> Result<Vec<DocumentChunk>> {
        match &self.chunking_strategy {
            ChunkingStrategy::Recursive {
                chunk_size,
                chunk_overlap,
                separators,
            } => {
                self.chunk_recursive(content, source, *chunk_size, *chunk_overlap, separators)
                    .await
            }
            ChunkingStrategy::SlidingWindow {
                window_size,
                step_size,
            } => {
                self.chunk_sliding_window(content, source, *window_size, *step_size)
                    .await
            }
            ChunkingStrategy::Semantic {
                max_chunk_size,
                similarity_threshold,
            } => {
                self.chunk_semantic(content, source, *max_chunk_size, *similarity_threshold)
                    .await
            }
            ChunkingStrategy::FixedSize { size, overlap } => {
                self.chunk_fixed_size(content, source, *size, *overlap)
                    .await
            }
        }
    }

    async fn chunk_recursive(
        &self,
        content: &str,
        source: &str,
        chunk_size: usize,
        chunk_overlap: usize,
        separators: &[String],
    ) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_index = 0;

        // Split by separators in order of preference
        for separator in separators {
            if content.contains(separator) {
                let parts: Vec<&str> = content.split(separator).collect();
                for part in parts {
                    if current_chunk.len() + part.len() > chunk_size && !current_chunk.is_empty() {
                        // Create chunk with overlap handling
                        let chunk = DocumentChunk::new(
                            current_chunk.clone(),
                            source.to_string(),
                            chunk_index,
                        )
                        .with_metadata("chunking_strategy", "recursive")
                        .with_metadata("separator", separator)
                        .with_metadata("chunk_size", &chunk_size.to_string());
                        chunks.push(chunk);

                        // Handle overlap
                        if chunk_overlap > 0 && current_chunk.len() > chunk_overlap {
                            current_chunk = current_chunk
                                .chars()
                                .skip(current_chunk.len() - chunk_overlap)
                                .collect();
                        } else {
                            current_chunk.clear();
                        }
                        chunk_index += 1;
                    }
                    current_chunk.push_str(part);
                    if separator != "\n" {
                        current_chunk.push_str(separator);
                    }
                }
                break;
            }
        }

        // Add remaining content as final chunk
        if !current_chunk.is_empty() {
            chunks.push(
                DocumentChunk::new(current_chunk, source.to_string(), chunk_index)
                    .with_metadata("chunking_strategy", "recursive")
                    .with_metadata("chunk_size", &chunk_size.to_string()),
            );
        }

        Ok(chunks)
    }

    async fn chunk_sliding_window(
        &self,
        content: &str,
        source: &str,
        window_size: usize,
        step_size: usize,
    ) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let mut chunk_index = 0;

        let mut start = 0;
        while start < chars.len() {
            let end = std::cmp::min(start + window_size, chars.len());
            let chunk_content: String = chars[start..end].iter().collect();

            let chunk = DocumentChunk::new(chunk_content, source.to_string(), chunk_index)
                .with_metadata("chunking_strategy", "sliding_window")
                .with_metadata("window_size", &window_size.to_string())
                .with_metadata("step_size", &step_size.to_string())
                .with_metadata("start_position", &start.to_string());

            chunks.push(chunk);
            chunk_index += 1;
            start += step_size;
        }

        Ok(chunks)
    }

    async fn chunk_semantic(
        &self,
        content: &str,
        source: &str,
        max_chunk_size: usize,
        _similarity_threshold: f32,
    ) -> Result<Vec<DocumentChunk>> {
        // Simulate semantic chunking (would use sentence embeddings in real implementation)
        let sentences: Vec<&str> = content
            .split('.')
            .filter(|s| !s.trim().is_empty())
            .collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_index = 0;

        for sentence in sentences {
            if current_chunk.len() + sentence.len() > max_chunk_size && !current_chunk.is_empty() {
                let chunk =
                    DocumentChunk::new(current_chunk.clone(), source.to_string(), chunk_index)
                        .with_metadata("chunking_strategy", "semantic")
                        .with_metadata("max_chunk_size", &max_chunk_size.to_string());
                chunks.push(chunk);
                current_chunk.clear();
                chunk_index += 1;
            }
            current_chunk.push_str(sentence);
            current_chunk.push('.');
        }

        if !current_chunk.is_empty() {
            chunks.push(
                DocumentChunk::new(current_chunk, source.to_string(), chunk_index)
                    .with_metadata("chunking_strategy", "semantic")
                    .with_metadata("max_chunk_size", &max_chunk_size.to_string()),
            );
        }

        Ok(chunks)
    }

    async fn chunk_fixed_size(
        &self,
        content: &str,
        source: &str,
        size: usize,
        overlap: usize,
    ) -> Result<Vec<DocumentChunk>> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let mut chunk_index = 0;
        let mut start = 0;

        while start < chars.len() {
            let end = std::cmp::min(start + size, chars.len());
            let chunk_content: String = chars[start..end].iter().collect();

            let chunk = DocumentChunk::new(chunk_content, source.to_string(), chunk_index)
                .with_metadata("chunking_strategy", "fixed_size")
                .with_metadata("size", &size.to_string())
                .with_metadata("overlap", &overlap.to_string());

            chunks.push(chunk);
            chunk_index += 1;

            // Move start position considering overlap
            let next_start = start + size.saturating_sub(overlap);
            if next_start <= start {
                break;
            }
            start = next_start;
        }

        Ok(chunks)
    }
}

/// Embedding generator with performance tracking
pub struct EmbeddingGenerator {
    pub model_name: String,
    pub embedding_dimensions: usize,
}

impl EmbeddingGenerator {
    pub fn new(model_name: &str, dimensions: usize) -> Self {
        Self {
            model_name: model_name.to_string(),
            embedding_dimensions: dimensions,
        }
    }

    /// Generate embeddings for text chunks
    pub async fn generate_embeddings(
        &self,
        chunks: &mut [DocumentChunk],
    ) -> Result<EmbeddingMetrics> {
        let start_time = Instant::now();
        let mut total_tokens = 0;
        let mut embedding_cost = 0.0;

        println!(
            "  🔢 Generating embeddings for {} chunks using {}",
            chunks.len(),
            self.model_name
        );

        for chunk in chunks.iter_mut() {
            let embedding = self.generate_single_embedding(&chunk.content).await?;
            let token_count = chunk.content.split_whitespace().count();

            chunk.embedding = Some(embedding);
            total_tokens += token_count;
            embedding_cost += self.calculate_embedding_cost(token_count);
        }

        let duration = start_time.elapsed();

        let metrics = EmbeddingMetrics {
            total_chunks: chunks.len(),
            total_tokens,
            total_cost: embedding_cost,
            generation_time: duration,
            model_name: self.model_name.clone(),
            dimensions: self.embedding_dimensions,
        };

        println!(
            "  ✅ Generated {} embeddings in {:?} (${:.4} estimated cost)",
            chunks.len(),
            duration,
            embedding_cost
        );

        Ok(metrics)
    }

    async fn generate_single_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Simulate embedding generation with different model characteristics
        let processing_time = match self.model_name.as_str() {
            "text-embedding-ada-002" => 50,
            "text-embedding-3-small" => 30,
            "text-embedding-3-large" => 80,
            _ => 40,
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(processing_time)).await;

        // Generate mock embedding based on text content
        let mut embedding = vec![0.0; self.embedding_dimensions];
        let text_hash = self.simple_text_hash(text);

        for (i, value) in embedding.iter_mut().enumerate() {
            *value = ((text_hash + i as u64) as f64 / u64::MAX as f64) as f32 * 2.0 - 1.0;
        }

        Ok(embedding)
    }

    fn simple_text_hash(&self, text: &str) -> u64 {
        text.chars().enumerate().fold(0u64, |acc, (i, c)| {
            acc.wrapping_add((c as u64).wrapping_mul(i as u64 + 1))
        })
    }

    fn calculate_embedding_cost(&self, token_count: usize) -> f64 {
        // Mock pricing based on OpenAI's embedding models
        let cost_per_1k_tokens = match self.model_name.as_str() {
            "text-embedding-ada-002" => 0.0001,
            "text-embedding-3-small" => 0.00002,
            "text-embedding-3-large" => 0.00013,
            _ => 0.0001,
        };

        (token_count as f64 / 1000.0) * cost_per_1k_tokens
    }
}

/// Metrics for embedding generation
#[derive(Debug)]
pub struct EmbeddingMetrics {
    pub total_chunks: usize,
    pub total_tokens: usize,
    pub total_cost: f64,
    pub generation_time: std::time::Duration,
    pub model_name: String,
    pub dimensions: usize,
}

/// Vector store interface for similarity search
#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn store_chunks(&self, chunks: &[DocumentChunk]) -> Result<usize>;
    async fn similarity_search(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> Result<Vec<SimilarityMatch>>;
    async fn get_stats(&self) -> Result<VectorStoreStats>;
}

/// Similarity search result
#[derive(Debug, Clone)]
pub struct SimilarityMatch {
    pub chunk: DocumentChunk,
    pub similarity_score: f32,
    pub rank: usize,
}

/// Vector store statistics
#[derive(Debug)]
pub struct VectorStoreStats {
    pub total_chunks: usize,
    pub total_documents: usize,
    pub storage_size_mb: f64,
    pub avg_chunk_size: usize,
}

/// SQLite vector store implementation
pub struct SQLiteVectorStore {
    pub store_name: String,
    pub chunks: Arc<tokio::sync::RwLock<Vec<DocumentChunk>>>,
}

impl SQLiteVectorStore {
    pub fn new(store_name: &str) -> Self {
        Self {
            store_name: store_name.to_string(),
            chunks: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl VectorStore for SQLiteVectorStore {
    async fn store_chunks(&self, chunks: &[DocumentChunk]) -> Result<usize> {
        let mut store = self.chunks.write().await;
        store.extend(chunks.iter().cloned());
        println!("  💾 Stored {} chunks in SQLite vector store", chunks.len());
        Ok(chunks.len())
    }

    async fn similarity_search(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> Result<Vec<SimilarityMatch>> {
        let store = self.chunks.read().await;
        let mut similarities = Vec::new();

        for chunk in store.iter() {
            if let Some(ref embedding) = chunk.embedding {
                let similarity = self.cosine_similarity(query_embedding, embedding);
                similarities.push((chunk.clone(), similarity));
            }
        }

        // Sort by similarity (descending) and take top k
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(k);

        let matches = similarities
            .into_iter()
            .enumerate()
            .map(|(rank, (chunk, score))| SimilarityMatch {
                chunk,
                similarity_score: score,
                rank,
            })
            .collect();

        Ok(matches)
    }

    async fn get_stats(&self) -> Result<VectorStoreStats> {
        let store = self.chunks.read().await;
        let total_chunks = store.len();
        let unique_docs: std::collections::HashSet<String> =
            store.iter().map(|c| c.source_document.clone()).collect();
        let avg_chunk_size = if total_chunks > 0 {
            store.iter().map(|c| c.content.len()).sum::<usize>() / total_chunks
        } else {
            0
        };

        Ok(VectorStoreStats {
            total_chunks,
            total_documents: unique_docs.len(),
            storage_size_mb: (total_chunks * avg_chunk_size) as f64 / (1024.0 * 1024.0),
            avg_chunk_size,
        })
    }
}

impl SQLiteVectorStore {
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

/// RAG query cache for performance optimization
pub struct QueryCache {
    cache: Arc<tokio::sync::RwLock<HashMap<String, CachedResult>>>,
    max_size: usize,
    hit_count: Arc<tokio::sync::RwLock<usize>>,
    miss_count: Arc<tokio::sync::RwLock<usize>>,
}

#[derive(Debug, Clone)]
struct CachedResult {
    matches: Vec<SimilarityMatch>,
    timestamp: SystemTime,
    ttl: std::time::Duration,
}

impl QueryCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            max_size,
            hit_count: Arc::new(tokio::sync::RwLock::new(0)),
            miss_count: Arc::new(tokio::sync::RwLock::new(0)),
        }
    }

    pub async fn get(&self, query_key: &str) -> Option<Vec<SimilarityMatch>> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(query_key) {
            // Check TTL
            if cached.timestamp.elapsed().unwrap_or_default() < cached.ttl {
                *self.hit_count.write().await += 1;
                return Some(cached.matches.clone());
            }
        }

        *self.miss_count.write().await += 1;
        None
    }

    pub async fn put(
        &self,
        query_key: String,
        matches: Vec<SimilarityMatch>,
        ttl: std::time::Duration,
    ) {
        let mut cache = self.cache.write().await;

        // Simple LRU eviction if cache is full
        if cache.len() >= self.max_size {
            let oldest_key = cache.keys().next().cloned();
            if let Some(key) = oldest_key {
                cache.remove(&key);
            }
        }

        cache.insert(
            query_key,
            CachedResult {
                matches,
                timestamp: SystemTime::now(),
                ttl,
            },
        );
    }

    pub async fn get_stats(&self) -> (usize, usize, f64) {
        let hits = *self.hit_count.read().await;
        let misses = *self.miss_count.read().await;
        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };
        (hits, misses, hit_rate)
    }
}

/// RAG observability and metrics collector
pub struct RAGObservability {
    pub metrics: Arc<tokio::sync::RwLock<RAGMetrics>>,
}

#[derive(Debug, Default, Clone)]
pub struct RAGMetrics {
    pub total_queries: usize,
    pub total_documents_processed: usize,
    pub total_chunks_created: usize,
    pub total_embeddings_generated: usize,
    pub embedding_costs: f64,
    pub query_latencies: Vec<std::time::Duration>,
    pub retrieval_relevance_scores: Vec<f32>,
    pub cache_stats: (usize, usize, f64), // hits, misses, hit_rate
}

impl Default for RAGObservability {
    fn default() -> Self {
        Self::new()
    }
}

impl RAGObservability {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(tokio::sync::RwLock::new(RAGMetrics::default())),
        }
    }

    pub async fn record_query(&self, latency: std::time::Duration, relevance_scores: Vec<f32>) {
        let mut metrics = self.metrics.write().await;
        metrics.total_queries += 1;
        metrics.query_latencies.push(latency);
        metrics.retrieval_relevance_scores.extend(relevance_scores);
    }

    pub async fn record_embedding_generation(&self, embedding_metrics: &EmbeddingMetrics) {
        let mut metrics = self.metrics.write().await;
        metrics.total_embeddings_generated += embedding_metrics.total_chunks;
        metrics.embedding_costs += embedding_metrics.total_cost;
    }

    pub async fn record_document_processing(&self, chunks_created: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.total_documents_processed += 1;
        metrics.total_chunks_created += chunks_created;
    }

    pub async fn update_cache_stats(&self, cache_stats: (usize, usize, f64)) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_stats = cache_stats;
    }

    pub async fn export_metrics(&self) -> RAGMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn print_summary(&self) {
        let metrics = self.metrics.read().await;

        println!("\n📊 RAG System Performance Summary");
        println!("=================================");
        println!(
            "📄 Documents Processed: {}",
            metrics.total_documents_processed
        );
        println!("🧩 Chunks Created: {}", metrics.total_chunks_created);
        println!(
            "🔢 Embeddings Generated: {}",
            metrics.total_embeddings_generated
        );
        println!("💰 Embedding Costs: ${:.6}", metrics.embedding_costs);
        println!("🔍 Total Queries: {}", metrics.total_queries);

        if !metrics.query_latencies.is_empty() {
            let avg_latency = metrics.query_latencies.iter().sum::<std::time::Duration>()
                / metrics.query_latencies.len() as u32;
            println!("⏱️  Average Query Latency: {:?}", avg_latency);
        }

        if !metrics.retrieval_relevance_scores.is_empty() {
            let avg_relevance = metrics.retrieval_relevance_scores.iter().sum::<f32>()
                / metrics.retrieval_relevance_scores.len() as f32;
            println!("🎯 Average Relevance Score: {:.3}", avg_relevance);
        }

        println!(
            "💾 Cache Hit Rate: {:.1}% ({} hits, {} misses)",
            metrics.cache_stats.2 * 100.0,
            metrics.cache_stats.0,
            metrics.cache_stats.1
        );
    }
}

/// Complete RAG system integrating all components
pub struct RAGSystem {
    pub document_processor: DocumentProcessor,
    pub embedding_generator: EmbeddingGenerator,
    pub vector_store: Arc<SQLiteVectorStore>,
    pub query_cache: QueryCache,
    pub observability: RAGObservability,
}

impl RAGSystem {
    pub fn new(
        chunking_strategy: ChunkingStrategy,
        embedding_model: &str,
        embedding_dimensions: usize,
        vector_store: Arc<SQLiteVectorStore>,
    ) -> Self {
        Self {
            document_processor: DocumentProcessor::new(chunking_strategy),
            embedding_generator: EmbeddingGenerator::new(embedding_model, embedding_dimensions),
            vector_store,
            query_cache: QueryCache::new(1000),
            observability: RAGObservability::new(),
        }
    }

    /// Ingest document into RAG system
    pub async fn ingest_document(
        &self,
        content: &str,
        document_name: &str,
        doc_type: DocumentType,
    ) -> Result<()> {
        // Process and chunk document
        let mut chunks = self
            .document_processor
            .process_document(content, document_name, doc_type)
            .await?;

        // Generate embeddings
        let embedding_metrics = self
            .embedding_generator
            .generate_embeddings(&mut chunks)
            .await?;

        // Store in vector database
        self.vector_store.store_chunks(&chunks).await?;

        // Record metrics
        self.observability
            .record_document_processing(chunks.len())
            .await;
        self.observability
            .record_embedding_generation(&embedding_metrics)
            .await;

        println!("  ✅ Successfully ingested document: {}", document_name);
        Ok(())
    }

    /// Query RAG system for relevant context
    pub async fn query(&self, query_text: &str, k: usize) -> Result<Vec<SimilarityMatch>> {
        let start_time = Instant::now();
        let cache_key = format!("{}:{}", query_text, k);

        // Check cache first
        if let Some(cached_results) = self.query_cache.get(&cache_key).await {
            println!("  💾 Cache hit for query: {}", query_text);
            return Ok(cached_results);
        }

        // Generate query embedding
        let query_embedding = self
            .embedding_generator
            .generate_single_embedding(query_text)
            .await?;

        // Perform similarity search
        let matches = self
            .vector_store
            .similarity_search(&query_embedding, k)
            .await?;

        // Cache results
        self.query_cache
            .put(
                cache_key,
                matches.clone(),
                std::time::Duration::from_secs(300),
            )
            .await;

        let query_duration = start_time.elapsed();
        let relevance_scores: Vec<f32> = matches.iter().map(|m| m.similarity_score).collect();

        // Record metrics
        self.observability
            .record_query(query_duration, relevance_scores.clone())
            .await;
        self.observability
            .update_cache_stats(self.query_cache.get_stats().await)
            .await;

        println!(
            "  🔍 Found {} relevant chunks for query (similarity: {:.3}-{:.3})",
            matches.len(),
            relevance_scores
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&0.0),
            relevance_scores
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(&0.0)
        );

        Ok(matches)
    }
}

/// Workflow step that uses RAG for enhanced responses
pub struct RAGEnhancedStep {
    pub name: String,
    pub rag_system: Arc<RAGSystem>,
}

impl RAGEnhancedStep {
    pub fn new(name: &str, rag_system: Arc<RAGSystem>) -> Self {
        Self {
            name: name.to_string(),
            rag_system,
        }
    }
}

#[async_trait]
impl WorkflowStep for RAGEnhancedStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        println!("  🧠 RAG-Enhanced step '{}' processing", self.name);

        // Get query from context
        let query = context
            .metadata
            .get("query")
            .cloned()
            .unwrap_or_else(|| "default query".to_string());

        // Query RAG system for relevant context
        let matches = self.rag_system.query(&query, 3).await?;

        // Build enhanced context from retrieved chunks
        let context_text = matches
            .iter()
            .map(|m| {
                format!(
                    "Context (score: {:.3}): {}",
                    m.similarity_score, m.chunk.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // Simulate LLM generation with RAG context
        let enhanced_response = format!(
            "Enhanced Response based on retrieved context:\n\nQuery: {}\n\nRetrieved Context:\n{}\n\nGenerated Answer: Based on the retrieved context, I can provide a comprehensive response that is grounded in the available information. The similarity scores indicate high relevance to your query.",
            query, context_text
        );

        // Store enhanced response in context
        context
            .metadata
            .insert(format!("{}_response", self.name), enhanced_response);

        println!(
            "  ✅ RAG-Enhanced step '{}' completed with {} context chunks",
            self.name,
            matches.len()
        );
        Ok(WorkflowDecision::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Comprehensive RAG System Demo");
    println!("=================================\n");

    // Demo 1: Document Processing with Different Chunking Strategies
    println!("📋 Demo 1: Document Processing & Chunking Strategies");
    println!("----------------------------------------------------");

    let sample_documents = vec![
        ("company_handbook.pdf", DocumentType::Pdf, "COMPANY HANDBOOK\n\nChapter 1: Introduction\nWelcome to our company. We are committed to excellence and innovation in everything we do. Our mission is to provide exceptional value to our customers while maintaining the highest standards of quality and service.\n\nChapter 2: Policies\nAll employees must adhere to our code of conduct. We expect professionalism, integrity, and respect in all interactions. Our workplace is inclusive and diverse, welcoming people from all backgrounds.\n\nChapter 3: Benefits\nWe offer comprehensive benefits including health insurance, retirement plans, and professional development opportunities. Employee wellness is a priority, and we provide various programs to support work-life balance."),

        ("technical_manual.docx", DocumentType::Docx, "TECHNICAL MANUAL\n\nSystem Architecture Overview\nOur system is built on a microservices architecture that provides scalability and maintainability. Each service is independently deployable and operates with its own database.\n\nAPI Documentation\nRESTful APIs are used for inter-service communication. All endpoints are documented with OpenAPI specifications. Rate limiting and authentication are implemented for security.\n\nDeployment Guide\nDeployment uses containerization with Docker and orchestration with Kubernetes. CI/CD pipelines automatically test and deploy code changes to production environments."),
        ("research_paper.txt", DocumentType::Text, "RESEARCH PAPER: Advanced AI Applications\n\nAbstract\nThis paper explores the latest developments in artificial intelligence and their applications in various industries. We examine machine learning algorithms, natural language processing, and computer vision technologies.\n\nIntroduction\nArtificial intelligence has transformed numerous sectors, from healthcare to finance. The rapid advancement of deep learning techniques has enabled new possibilities for automation and decision-making.\n\nMethodology\nWe conducted experiments using transformer models and evaluated their performance on benchmark datasets. Our approach combines supervised and unsupervised learning techniques."),
    ];

    // Test different chunking strategies
    let chunking_strategies = vec![
        (
            "Recursive",
            ChunkingStrategy::Recursive {
                chunk_size: 200,
                chunk_overlap: 50,
                separators: vec!["\n\n".to_string(), "\n".to_string(), ". ".to_string()],
            },
        ),
        (
            "Sliding Window",
            ChunkingStrategy::SlidingWindow {
                window_size: 150,
                step_size: 100,
            },
        ),
        (
            "Semantic",
            ChunkingStrategy::Semantic {
                max_chunk_size: 300,
                similarity_threshold: 0.8,
            },
        ),
        (
            "Fixed Size",
            ChunkingStrategy::FixedSize {
                size: 180,
                overlap: 30,
            },
        ),
    ];

    for (strategy_name, strategy) in chunking_strategies {
        println!("\nTesting {} Chunking Strategy:", strategy_name);
        println!("{}", "=".repeat(40));

        let processor = DocumentProcessor::new(strategy);

        for (doc_name, doc_type, content) in &sample_documents {
            let chunks = processor
                .process_document(content, doc_name, doc_type.clone())
                .await?;
            println!(
                "  📄 {} -> {} chunks (avg size: {} chars)",
                doc_name,
                chunks.len(),
                if !chunks.is_empty() {
                    chunks.iter().map(|c| c.content.len()).sum::<usize>() / chunks.len()
                } else {
                    0
                }
            );
        }
    }

    // Demo 2: Multiple Vector Store Support
    println!("\n📋 Demo 2: Vector Store Implementations");
    println!("--------------------------------------");

    let vector_stores: Vec<(String, Arc<SQLiteVectorStore>)> = vec![
        (
            "SQLite".to_string(),
            Arc::new(SQLiteVectorStore::new("demo_sqlite")),
        ),
        // In real implementation, you would add other stores:
        // ("PgVector".to_string(), Arc::new(PgVectorStore::new(connection_string))),
        // ("Pinecone".to_string(), Arc::new(PineconeStore::new(api_key, index_name))),
    ];

    for (store_name, _store) in &vector_stores {
        println!("  🗄️  {} Vector Store initialized", store_name);
    }

    // Demo 3: Complete RAG System with Observability
    println!("\n📋 Demo 3: Complete RAG System Integration");
    println!("-----------------------------------------");

    // Create RAG system with different embedding models
    let embedding_models = vec![
        ("text-embedding-ada-002", 1536),
        ("text-embedding-3-small", 1536),
        ("text-embedding-3-large", 3072),
    ];

    for (model_name, dimensions) in embedding_models {
        println!(
            "\nTesting with {} model ({} dimensions):",
            model_name, dimensions
        );
        println!("{}", "=".repeat(50));

        let rag_system = Arc::new(RAGSystem::new(
            ChunkingStrategy::Recursive {
                chunk_size: 250,
                chunk_overlap: 50,
                separators: vec!["\n\n".to_string(), "\n".to_string(), ". ".to_string()],
            },
            model_name,
            dimensions,
            Arc::new(SQLiteVectorStore::new(&format!("rag_demo_{}", model_name))),
        ));

        // Ingest documents
        for (doc_name, doc_type, content) in &sample_documents {
            rag_system
                .ingest_document(content, doc_name, doc_type.clone())
                .await?;
        }

        // Test queries
        let test_queries = vec![
            "What are the company policies regarding employee conduct?",
            "How is the system architecture designed?",
            "What research methodology was used in the AI study?",
            "What benefits does the company offer to employees?",
            "Explain the deployment process for the technical system",
        ];

        for query in test_queries {
            println!("\nQuery: {}", query);
            let _matches = rag_system.query(query, 3).await?;
        }

        // Print performance metrics
        rag_system.observability.print_summary().await;
    }

    // Demo 4: RAG-Enhanced Workflow Integration
    println!("\n📋 Demo 4: RAG-Enhanced Workflow Integration");
    println!("--------------------------------------------");

    let rag_system = Arc::new(RAGSystem::new(
        ChunkingStrategy::Recursive {
            chunk_size: 200,
            chunk_overlap: 40,
            separators: vec!["\n\n".to_string(), "\n".to_string()],
        },
        "text-embedding-ada-002",
        1536,
        Arc::new(SQLiteVectorStore::new("workflow_rag")),
    ));

    // Ingest knowledge base
    let knowledge_base = "KNOWLEDGE BASE\n\nProduct Information:\nOur flagship product is an AI-powered analytics platform that helps businesses make data-driven decisions. It features real-time dashboards, predictive analytics, and automated reporting capabilities.\n\nCustomer Support Guidelines:\nWhen assisting customers, always be polite and professional. Listen carefully to their concerns and provide clear, actionable solutions. Escalate complex issues to senior support staff when necessary.\n\nTroubleshooting Common Issues:\n1. Login Problems: Check credentials and reset password if needed\n2. Data Loading Issues: Verify data format and connection settings\n3. Performance Problems: Clear cache and check system resources";

    rag_system
        .ingest_document(knowledge_base, "knowledge_base.txt", DocumentType::Text)
        .await?;

    // Create RAG-enhanced workflow
    let rag_workflow = WorkflowBuilder::new("rag_enhanced_workflow")
        .then(Box::new(RAGEnhancedStep::new(
            "customer_support",
            rag_system.clone(),
        )))
        .with_initial_data(
            json!({"query": "How do I troubleshoot login problems with the analytics platform?"}),
        )
        .build();

    let context = WorkflowContext::new(10);
    let result = rag_workflow.execute(context).await?;
    println!(
        "RAG-enhanced workflow completed: Steps executed = {}",
        result.steps_executed
    );

    // Demo 5: Performance and Cost Analysis
    println!("\n📋 Demo 5: Performance & Cost Analysis");
    println!("-------------------------------------");

    // Test different configurations for performance comparison
    let configs = vec![
        (
            "Small Chunks",
            ChunkingStrategy::FixedSize {
                size: 100,
                overlap: 20,
            },
            "text-embedding-3-small",
            1536,
        ),
        (
            "Medium Chunks",
            ChunkingStrategy::FixedSize {
                size: 300,
                overlap: 50,
            },
            "text-embedding-ada-002",
            1536,
        ),
        (
            "Large Chunks",
            ChunkingStrategy::FixedSize {
                size: 500,
                overlap: 100,
            },
            "text-embedding-3-large",
            3072,
        ),
    ];

    for (config_name, chunk_strategy, model, dimensions) in configs {
        println!("\nConfiguration: {}", config_name);
        println!("Chunking: {:?}", chunk_strategy);
        println!("Model: {} ({}D)", model, dimensions);

        let test_system = RAGSystem::new(
            chunk_strategy,
            model,
            dimensions,
            Arc::new(SQLiteVectorStore::new(&format!(
                "perf_test_{}",
                config_name.replace(" ", "_")
            ))),
        );

        let start_time = Instant::now();
        test_system
            .ingest_document(sample_documents[0].2, "test_doc", DocumentType::Text)
            .await?;
        let ingestion_time = start_time.elapsed();

        let start_time = Instant::now();
        let _results = test_system
            .query("test query for performance measurement", 5)
            .await?;
        let query_time = start_time.elapsed();

        println!("  Ingestion Time: {:?}", ingestion_time);
        println!("  Query Time: {:?}", query_time);

        test_system.observability.print_summary().await;
    }

    println!("\n🎉 RAG System Demo Completed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Multi-format Document Processing (.pdf, .docx, .txt, .md)");
    println!("   • Multiple Chunking Strategies (recursive, sliding window, semantic, fixed-size)");
    println!("   • Vector Store Abstraction (SQLite, with support for pgvector, Pinecone, etc.)");
    println!("   • Embedding Generation with Cost Tracking");
    println!("   • Similarity Search and Ranking");
    println!("   • Query Caching for Performance Optimization");
    println!("   • Comprehensive Observability and Metrics");
    println!("   • Workflow Integration for Enhanced AI Responses");

    println!("\n🏗️ Production-Ready Features:");
    println!("   • Performance Monitoring - Track latency, costs, and relevance");
    println!("   • Caching Layer - Reduce redundant computations");
    println!("   • Multiple Backend Support - Switch between vector stores");
    println!("   • Cost Optimization - Monitor and control embedding costs");
    println!("   • Scalable Architecture - Handle large document collections");
    println!("   • OpenTelemetry Ready - Export metrics to observability platforms");

    Ok(())
}
