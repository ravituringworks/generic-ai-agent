//! LLM response caching for improved performance on repeated queries

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use tracing::{debug, info};

/// Configuration for LLM response caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCacheConfig {
    /// Enable or disable caching
    pub enabled: bool,

    /// Maximum number of cache entries
    pub max_entries: usize,

    /// Time-to-live for cache entries in seconds
    pub ttl_seconds: i64,

    /// SQLite database path
    pub db_path: String,

    /// Minimum temperature threshold for caching (only cache deterministic queries)
    pub min_temperature_threshold: f32,
}

impl Default for LlmCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            ttl_seconds: 3600, // 1 hour
            db_path: "cache.db".to_string(),
            min_temperature_threshold: 0.3,
        }
    }
}

/// A cache entry stored in SQLite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub response: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub hit_count: i64,
    pub model: String,
    pub temperature: f32,
}

/// LLM response cache with SQLite backend
pub struct LlmCache {
    pool: SqlitePool,
    config: LlmCacheConfig,
}

impl LlmCache {
    /// Create a new LLM cache
    pub async fn new(config: LlmCacheConfig) -> Result<Self> {
        if !config.enabled {
            info!("LLM cache is disabled");
        }

        info!("Initializing LLM cache at: {}", config.db_path);

        // Create SQLite connection
        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", config.db_path))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Create cache table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS llm_cache (
                key TEXT PRIMARY KEY,
                response TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_accessed TEXT NOT NULL,
                hit_count INTEGER NOT NULL DEFAULT 0,
                model TEXT NOT NULL,
                temperature REAL NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create index for faster lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_last_accessed ON llm_cache(last_accessed)
            "#,
        )
        .execute(&pool)
        .await?;

        info!("LLM cache initialized successfully");

        Ok(Self { pool, config })
    }

    /// Compute a deterministic cache key from query parameters
    pub fn compute_cache_key(
        messages: &str,
        model: &str,
        temperature: f32,
        max_tokens: u32,
        system_prompt: Option<&str>,
    ) -> String {
        let mut hasher = DefaultHasher::new();

        // Hash all relevant parameters
        messages.hash(&mut hasher);
        model.hash(&mut hasher);
        // Convert float to bits for consistent hashing
        ((temperature * 1000.0) as u32).hash(&mut hasher);
        max_tokens.hash(&mut hasher);
        if let Some(prompt) = system_prompt {
            prompt.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Get a cached response if available and not expired
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        if !self.config.enabled {
            return Ok(None);
        }

        let result = sqlx::query_as::<_, (String, String, i64, String, f32)>(
            r#"
            SELECT response, created_at, hit_count, model, temperature
            FROM llm_cache
            WHERE key = ?
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((response, created_at_str, hit_count, model, temperature)) = result {
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let age = Utc::now().signed_duration_since(created_at);
            let ttl = Duration::seconds(self.config.ttl_seconds);

            // Check if entry has expired
            if age > ttl {
                debug!(
                    "Cache entry expired (age: {}s, ttl: {}s)",
                    age.num_seconds(),
                    ttl.num_seconds()
                );
                self.invalidate(key).await?;
                return Ok(None);
            }

            // Update last accessed time and hit count
            sqlx::query(
                r#"
                UPDATE llm_cache
                SET last_accessed = ?, hit_count = ?
                WHERE key = ?
                "#,
            )
            .bind(Utc::now().to_rfc3339())
            .bind(hit_count + 1)
            .bind(key)
            .execute(&self.pool)
            .await?;

            debug!(
                "Cache hit! (model: {}, temp: {:.2}, hits: {})",
                model,
                temperature,
                hit_count + 1
            );

            Ok(Some(response))
        } else {
            debug!("Cache miss for key: {}", key);
            Ok(None)
        }
    }

    /// Store a response in the cache
    pub async fn set(
        &self,
        key: String,
        response: String,
        model: String,
        temperature: f32,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Don't cache high-temperature (creative) responses
        if temperature > self.config.min_temperature_threshold {
            debug!(
                "Skipping cache for high temperature query (temp: {:.2})",
                temperature
            );
            return Ok(());
        }

        let now = Utc::now().to_rfc3339();

        // Insert or replace cache entry
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO llm_cache (key, response, created_at, last_accessed, hit_count, model, temperature)
            VALUES (?, ?, ?, ?, 0, ?, ?)
            "#,
        )
        .bind(&key)
        .bind(&response)
        .bind(&now)
        .bind(&now)
        .bind(&model)
        .bind(temperature)
        .execute(&self.pool)
        .await?;

        debug!(
            "Cached response (model: {}, temp: {:.2})",
            model, temperature
        );

        // Enforce max entries limit (LRU eviction)
        self.enforce_size_limit().await?;

        Ok(())
    }

    /// Enforce the maximum cache size by removing least recently used entries
    async fn enforce_size_limit(&self) -> Result<()> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM llm_cache")
            .fetch_one(&self.pool)
            .await?;

        if count.0 > self.config.max_entries as i64 {
            let to_remove = count.0 - self.config.max_entries as i64;

            sqlx::query(
                r#"
                DELETE FROM llm_cache
                WHERE key IN (
                    SELECT key FROM llm_cache
                    ORDER BY last_accessed ASC
                    LIMIT ?
                )
                "#,
            )
            .bind(to_remove)
            .execute(&self.pool)
            .await?;

            info!("Evicted {} old cache entries (LRU)", to_remove);
        }

        Ok(())
    }

    /// Remove a specific cache entry
    pub async fn invalidate(&self, key: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        sqlx::query("DELETE FROM llm_cache WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;

        debug!("Invalidated cache entry: {}", key);
        Ok(())
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        sqlx::query("DELETE FROM llm_cache")
            .execute(&self.pool)
            .await?;

        info!("Cleared all cache entries");
        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Result<CacheStats> {
        let total_entries: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM llm_cache")
            .fetch_one(&self.pool)
            .await?;

        let total_hits: (i64,) =
            sqlx::query_as("SELECT COALESCE(SUM(hit_count), 0) FROM llm_cache")
                .fetch_one(&self.pool)
                .await?;

        let avg_age: (Option<f64>,) = sqlx::query_as(
            r#"
            SELECT AVG(
                (julianday('now') - julianday(created_at)) * 86400
            ) FROM llm_cache
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(CacheStats {
            total_entries: total_entries.0 as usize,
            total_hits: total_hits.0 as usize,
            avg_age_seconds: avg_age.0.unwrap_or(0.0) as i64,
            max_entries: self.config.max_entries,
            ttl_seconds: self.config.ttl_seconds,
        })
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> Result<usize> {
        if !self.config.enabled {
            return Ok(0);
        }

        let cutoff = Utc::now() - Duration::seconds(self.config.ttl_seconds);

        let result = sqlx::query(
            r#"
            DELETE FROM llm_cache
            WHERE created_at < ?
            "#,
        )
        .bind(cutoff.to_rfc3339())
        .execute(&self.pool)
        .await?;

        let removed = result.rows_affected() as usize;

        if removed > 0 {
            info!("Cleaned up {} expired cache entries", removed);
        }

        Ok(removed)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: usize,
    pub avg_age_seconds: i64,
    pub max_entries: usize,
    pub ttl_seconds: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_cache_key_generation() {
        let key1 = LlmCache::compute_cache_key(
            "Hello world",
            "llama3.2",
            0.7,
            1000,
            Some("You are helpful"),
        );
        let key2 = LlmCache::compute_cache_key(
            "Hello world",
            "llama3.2",
            0.7,
            1000,
            Some("You are helpful"),
        );

        assert_eq!(key1, key2, "Same inputs should produce same cache key");

        let key3 = LlmCache::compute_cache_key(
            "Different message",
            "llama3.2",
            0.7,
            1000,
            Some("You are helpful"),
        );

        assert_ne!(key1, key3, "Different inputs should produce different keys");
    }

    #[tokio::test]
    async fn test_cache_operations() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let db_path = temp_file.path().to_str().unwrap().to_string();

        let config = LlmCacheConfig {
            enabled: true,
            max_entries: 10,
            ttl_seconds: 3600,
            db_path,
            min_temperature_threshold: 0.3,
        };

        let cache = LlmCache::new(config).await?;

        let key = "test_key";
        let response = "Test response";

        // Test cache miss
        assert!(cache.get(key).await?.is_none());

        // Test cache set
        cache
            .set(
                key.to_string(),
                response.to_string(),
                "llama3.2".to_string(),
                0.2,
            )
            .await?;

        // Test cache hit
        let cached = cache.get(key).await?;
        assert_eq!(cached, Some(response.to_string()));

        // Test stats
        let stats = cache.stats().await?;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_hits, 1);

        // Test invalidation
        cache.invalidate(key).await?;
        assert!(cache.get(key).await?.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_temperature_threshold() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let db_path = temp_file.path().to_str().unwrap().to_string();

        let config = LlmCacheConfig {
            enabled: true,
            max_entries: 10,
            ttl_seconds: 3600,
            db_path,
            min_temperature_threshold: 0.3,
        };

        let cache = LlmCache::new(config).await?;

        // High temperature should not be cached
        cache
            .set(
                "key1".to_string(),
                "response".to_string(),
                "llama3.2".to_string(),
                0.8,
            )
            .await?;

        let stats = cache.stats().await?;
        assert_eq!(
            stats.total_entries, 0,
            "High temp queries should not be cached"
        );

        // Low temperature should be cached
        cache
            .set(
                "key2".to_string(),
                "response".to_string(),
                "llama3.2".to_string(),
                0.2,
            )
            .await?;

        let stats = cache.stats().await?;
        assert_eq!(stats.total_entries, 1, "Low temp queries should be cached");

        Ok(())
    }
}
