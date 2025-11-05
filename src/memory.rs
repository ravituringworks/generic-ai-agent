//! Memory and vector store functionality

use crate::config::MemoryConfig;
use crate::error::{MemoryError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// A memory entry with embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Search result from vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entry: MemoryEntry,
    pub similarity: f32,
}

/// Memory store trait for different implementations
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Initialize the memory store
    async fn initialize(&mut self) -> Result<()>;

    /// Store a memory entry
    async fn store(
        &mut self,
        content: String,
        embedding: Vec<f32>,
        metadata: HashMap<String, String>,
    ) -> Result<Uuid>;

    /// Search for similar memories
    async fn search(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>>;

    /// Get a specific memory by ID
    async fn get(&self, id: Uuid) -> Result<Option<MemoryEntry>>;

    /// Update a memory entry
    async fn update(
        &mut self,
        id: Uuid,
        content: Option<String>,
        embedding: Option<Vec<f32>>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()>;

    /// Delete a memory entry
    async fn delete(&mut self, id: Uuid) -> Result<()>;

    /// Get all memories (for small datasets)
    async fn list(&self, limit: Option<usize>) -> Result<Vec<MemoryEntry>>;

    /// Clear all memories
    async fn clear(&mut self) -> Result<()>;

    /// Get store statistics
    async fn stats(&self) -> Result<MemoryStats>;
}

/// Vector store trait for similarity search
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Add vectors to the store
    async fn add_vectors(&mut self, vectors: Vec<(Uuid, Vec<f32>)>) -> Result<()>;

    /// Search for similar vectors
    async fn search_vectors(
        &self,
        query: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<(Uuid, f32)>>;

    /// Remove vectors from the store
    async fn remove_vectors(&mut self, ids: Vec<Uuid>) -> Result<()>;

    /// Get the size of the vector store
    async fn size(&self) -> Result<usize>;

    /// Clear all vectors
    async fn clear(&mut self) -> Result<()>;
}

/// Memory store statistics
#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub embedding_dimension: usize,
    pub store_size_bytes: Option<usize>,
}

/// SQLite-based memory store implementation
pub struct SqliteMemoryStore {
    pool: Option<SqlitePool>,
    config: MemoryConfig,
}

impl SqliteMemoryStore {
    /// Create a new SQLite memory store
    pub fn new(config: MemoryConfig) -> Self {
        Self { pool: None, config }
    }

    /// Get database pool
    fn pool(&self) -> Result<&SqlitePool> {
        self.pool.as_ref().ok_or(MemoryError::NotInitialized.into())
    }

    /// Calculate cosine similarity between two vectors
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Serialize embedding for storage
    pub fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
        embedding
            .iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect()
    }

    /// Deserialize embedding from storage
    pub fn deserialize_embedding(data: &[u8]) -> Vec<f32> {
        data.chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }

    /// Serialize metadata for storage
    fn serialize_metadata(metadata: &HashMap<String, String>) -> Result<String> {
        Ok(serde_json::to_string(metadata)?)
    }

    /// Deserialize metadata from storage
    fn deserialize_metadata(data: &str) -> Result<HashMap<String, String>> {
        Ok(serde_json::from_str(data).unwrap_or_default())
    }
}

#[async_trait]
impl MemoryStore for SqliteMemoryStore {
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing SQLite memory store");

        let database_url =
            self.config.database_url.as_ref().ok_or_else(|| {
                MemoryError::StorageFailed("No database URL provided".to_string())
            })?;

        // Ensure the directory for the database file exists
        if let Some(db_path) = database_url.strip_prefix("sqlite:") {
            // Remove any query parameters to get just the file path
            let db_path = db_path.split('?').next().unwrap_or(db_path);
            if let Some(parent) = std::path::Path::new(db_path).parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        MemoryError::StorageFailed(format!(
                            "Failed to create database directory: {}",
                            e
                        ))
                    })?;
                }
            }
        }

        // Add SQLite create mode if not present
        let database_url = if database_url.contains('?') {
            if !database_url.contains("mode=") {
                format!("{}&mode=rwc", database_url)
            } else {
                database_url.clone()
            }
        } else {
            format!("{}?mode=rwc", database_url)
        };

        let pool = SqlitePool::connect(&database_url).await?;

        // Create the memories table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                embedding BLOB NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create index for faster searches (though not optimal for vector similarity)
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at)")
            .execute(&pool)
            .await?;

        self.pool = Some(pool);
        info!("SQLite memory store initialized");

        Ok(())
    }

    async fn store(
        &mut self,
        content: String,
        embedding: Vec<f32>,
        metadata: HashMap<String, String>,
    ) -> Result<Uuid> {
        let pool = self.pool()?;

        if embedding.len() != self.config.embedding_dimension {
            return Err(MemoryError::InvalidDimension {
                expected: self.config.embedding_dimension,
                actual: embedding.len(),
            }
            .into());
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        let embedding_blob = Self::serialize_embedding(&embedding);
        let metadata_json = Self::serialize_metadata(&metadata)?;

        sqlx::query(
            r#"
            INSERT INTO memories (id, content, embedding, metadata, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(id.to_string())
        .bind(&content)
        .bind(&embedding_blob)
        .bind(&metadata_json)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;

        debug!("Stored memory entry with ID: {}", id);
        Ok(id)
    }

    async fn search(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        let pool = self.pool()?;

        if query_embedding.len() != self.config.embedding_dimension {
            return Err(MemoryError::InvalidDimension {
                expected: self.config.embedding_dimension,
                actual: query_embedding.len(),
            }
            .into());
        }

        // For SQLite without vector extensions, we need to do brute-force similarity search
        let rows = sqlx::query("SELECT * FROM memories")
            .fetch_all(pool)
            .await?;

        let mut results = Vec::new();

        for row in rows {
            let id: String = row.get("id");
            let content: String = row.get("content");
            let embedding_blob: Vec<u8> = row.get("embedding");
            let metadata_json: String = row.get("metadata");
            let created_at: String = row.get("created_at");
            let updated_at: String = row.get("updated_at");

            let embedding = Self::deserialize_embedding(&embedding_blob);
            let similarity = Self::cosine_similarity(&query_embedding, &embedding);

            if similarity >= threshold {
                let entry = MemoryEntry {
                    id: Uuid::parse_str(&id)
                        .map_err(|e| MemoryError::StorageFailed(e.to_string()))?,
                    content,
                    embedding,
                    metadata: Self::deserialize_metadata(&metadata_json)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map_err(|e| MemoryError::StorageFailed(e.to_string()))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map_err(|e| MemoryError::StorageFailed(e.to_string()))?
                        .with_timezone(&Utc),
                };

                results.push(SearchResult { entry, similarity });
            }
        }

        // Sort by similarity (highest first) and limit results
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        debug!(
            "Found {} similar memories above threshold {}",
            results.len(),
            threshold
        );
        Ok(results)
    }

    async fn get(&self, id: Uuid) -> Result<Option<MemoryEntry>> {
        let pool = self.pool()?;

        let row = sqlx::query("SELECT * FROM memories WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?;

        if let Some(row) = row {
            let content: String = row.get("content");
            let embedding_blob: Vec<u8> = row.get("embedding");
            let metadata_json: String = row.get("metadata");
            let created_at: String = row.get("created_at");
            let updated_at: String = row.get("updated_at");

            let embedding = Self::deserialize_embedding(&embedding_blob);
            let metadata = Self::deserialize_metadata(&metadata_json)?;

            Ok(Some(MemoryEntry {
                id,
                content,
                embedding,
                metadata,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| MemoryError::StorageFailed(e.to_string()))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)
                    .map_err(|e| MemoryError::StorageFailed(e.to_string()))?
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update(
        &mut self,
        id: Uuid,
        content: Option<String>,
        embedding: Option<Vec<f32>>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let pool = self.pool()?;
        let now = Utc::now();

        if let Some(ref emb) = embedding {
            if emb.len() != self.config.embedding_dimension {
                return Err(MemoryError::InvalidDimension {
                    expected: self.config.embedding_dimension,
                    actual: emb.len(),
                }
                .into());
            }
        }

        let mut query_parts = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Some(content) = content {
            query_parts.push("content = ?");
            values.push(content);
        }

        if let Some(embedding) = embedding {
            query_parts.push("embedding = ?");
            let embedding_blob = Self::serialize_embedding(&embedding);
            use base64::Engine;
            values.push(base64::engine::general_purpose::STANDARD.encode(embedding_blob));
            // Simplified for example
        }

        if let Some(metadata) = metadata {
            query_parts.push("metadata = ?");
            values.push(Self::serialize_metadata(&metadata)?);
        }

        if query_parts.is_empty() {
            return Ok(());
        }

        query_parts.push("updated_at = ?");
        values.push(now.to_rfc3339());

        let query_str = format!(
            "UPDATE memories SET {} WHERE id = ?",
            query_parts.join(", ")
        );

        let mut query = sqlx::query(&query_str);
        for value in values {
            query = query.bind(value);
        }
        query = query.bind(id.to_string());

        let result = query.execute(pool).await?;

        if result.rows_affected() == 0 {
            warn!("No memory found with ID: {}", id);
        } else {
            debug!("Updated memory entry with ID: {}", id);
        }

        Ok(())
    }

    async fn delete(&mut self, id: Uuid) -> Result<()> {
        let pool = self.pool()?;

        let result = sqlx::query("DELETE FROM memories WHERE id = ?1")
            .bind(id.to_string())
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            warn!("No memory found with ID: {}", id);
        } else {
            debug!("Deleted memory entry with ID: {}", id);
        }

        Ok(())
    }

    async fn list(&self, limit: Option<usize>) -> Result<Vec<MemoryEntry>> {
        let pool = self.pool()?;

        let query = if let Some(limit) = limit {
            format!(
                "SELECT * FROM memories ORDER BY created_at DESC LIMIT {}",
                limit
            )
        } else {
            "SELECT * FROM memories ORDER BY created_at DESC".to_string()
        };

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let mut entries = Vec::new();

        for row in rows {
            let id: String = row.get("id");
            let content: String = row.get("content");
            let embedding_blob: Vec<u8> = row.get("embedding");
            let metadata_json: String = row.get("metadata");
            let created_at: String = row.get("created_at");
            let updated_at: String = row.get("updated_at");

            let embedding = Self::deserialize_embedding(&embedding_blob);
            let metadata = Self::deserialize_metadata(&metadata_json)?;

            entries.push(MemoryEntry {
                id: Uuid::parse_str(&id).map_err(|e| MemoryError::StorageFailed(e.to_string()))?,
                content,
                embedding,
                metadata,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| MemoryError::StorageFailed(e.to_string()))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)
                    .map_err(|e| MemoryError::StorageFailed(e.to_string()))?
                    .with_timezone(&Utc),
            });
        }

        debug!("Listed {} memory entries", entries.len());
        Ok(entries)
    }

    async fn clear(&mut self) -> Result<()> {
        let pool = self.pool()?;

        let result = sqlx::query("DELETE FROM memories").execute(pool).await?;

        info!("Cleared {} memory entries", result.rows_affected());
        Ok(())
    }

    async fn stats(&self) -> Result<MemoryStats> {
        let pool = self.pool()?;

        let row = sqlx::query("SELECT COUNT(*) as count FROM memories")
            .fetch_one(pool)
            .await?;

        let total_memories: i64 = row.get("count");

        Ok(MemoryStats {
            total_memories: total_memories as usize,
            embedding_dimension: self.config.embedding_dimension,
            store_size_bytes: None, // Could be calculated by examining the database file
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    async fn create_test_store() -> SqliteMemoryStore {
        // Use in-memory SQLite database for tests
        let database_url = "sqlite::memory:".to_string();

        let config = MemoryConfig {
            database_url: Some(database_url),
            embedding_dimension: 384,
            ..Default::default()
        };

        let mut store = SqliteMemoryStore::new(config);
        store.initialize().await.unwrap();
        store
    }

    #[tokio::test]
    async fn test_memory_store_lifecycle() {
        let mut store = create_test_store().await;

        // Store a memory
        let embedding = vec![0.1; 384];
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let id = store
            .store(
                "This is a test memory".to_string(),
                embedding.clone(),
                metadata.clone(),
            )
            .await
            .unwrap();

        // Retrieve the memory
        let retrieved = store.get(id).await.unwrap().unwrap();
        assert_eq!(retrieved.content, "This is a test memory");
        assert_eq!(retrieved.embedding.len(), 384);
        assert_eq!(retrieved.metadata.get("source").unwrap(), "test");

        // Update the memory
        let mut new_metadata = HashMap::new();
        new_metadata.insert("updated".to_string(), "true".to_string());

        store
            .update(
                id,
                Some("Updated content".to_string()),
                None,
                Some(new_metadata),
            )
            .await
            .unwrap();

        let updated = store.get(id).await.unwrap().unwrap();
        assert_eq!(updated.content, "Updated content");

        // Delete the memory
        store.delete(id).await.unwrap();
        let deleted = store.get(id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_vector_search() {
        let mut store = create_test_store().await;

        // Store similar vectors
        let mut base_embedding = vec![1.0, 0.0, 0.0, 0.0]; // Pad to 384
        base_embedding.resize(384, 0.0);
        let mut similar_embedding = vec![1.0, 0.1, 0.0, 0.0]; // Similar to base
        let mut different_embedding = vec![0.0, 0.0, 1.0, 0.0];

        // Pad vectors to required dimension
        similar_embedding.resize(384, 0.0);
        different_embedding.resize(384, 0.0);

        store
            .store(
                "Base document".to_string(),
                base_embedding.clone(),
                HashMap::new(),
            )
            .await
            .unwrap();
        store
            .store(
                "Similar document".to_string(),
                similar_embedding,
                HashMap::new(),
            )
            .await
            .unwrap();
        store
            .store(
                "Different document".to_string(),
                different_embedding,
                HashMap::new(),
            )
            .await
            .unwrap();

        // Search for similar vectors
        let results = store.search(base_embedding, 10, 0.5).await.unwrap();

        // Should find at least the exact match and similar document
        assert!(!results.is_empty());
        assert!(results[0].similarity > 0.8);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert_eq!(SqliteMemoryStore::cosine_similarity(&a, &b), 1.0);
        assert_eq!(SqliteMemoryStore::cosine_similarity(&a, &c), 0.0);
    }

    #[test]
    fn test_embedding_serialization() {
        let embedding = vec![1.5, -2.3, 0.0, 42.1];
        let serialized = SqliteMemoryStore::serialize_embedding(&embedding);
        let deserialized = SqliteMemoryStore::deserialize_embedding(&serialized);

        assert_eq!(embedding.len(), deserialized.len());
        for (orig, deser) in embedding.iter().zip(deserialized.iter()) {
            assert!((orig - deser).abs() < f32::EPSILON);
        }
    }
}
