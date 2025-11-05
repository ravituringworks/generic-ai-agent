//! Connection pool for managing concurrent Ollama requests
//!
//! This module provides a semaphore-based connection pool to prevent
//! overwhelming the Ollama server with too many concurrent requests.

use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, warn};

/// Connection pool for rate-limiting Ollama requests
#[derive(Clone)]
pub struct OllamaConnectionPool {
    /// Semaphore to limit concurrent requests
    semaphore: Arc<Semaphore>,
    /// Maximum concurrent connections
    max_connections: usize,
}

impl Default for OllamaConnectionPool {
    fn default() -> Self {
        Self::new(5)
    }
}

impl OllamaConnectionPool {
    /// Create a new connection pool with specified max connections
    pub fn new(max_connections: usize) -> Self {
        debug!(
            "Creating Ollama connection pool with {} max connections",
            max_connections
        );
        Self {
            semaphore: Arc::new(Semaphore::new(max_connections)),
            max_connections,
        }
    }

    /// Acquire a permit to make a request
    /// This will block if all connections are in use
    pub async fn acquire(&self) -> ConnectionPermit {
        let available = self.semaphore.available_permits();
        if available == 0 {
            warn!(
                "All {} Ollama connections in use, waiting for available slot...",
                self.max_connections
            );
        }

        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("Semaphore should not be closed");

        debug!(
            "Acquired Ollama connection permit ({}/{} in use)",
            self.max_connections - self.semaphore.available_permits(),
            self.max_connections
        );

        ConnectionPermit {
            _permit: permit,
            max_connections: self.max_connections,
            semaphore: self.semaphore.clone(),
        }
    }

    /// Get current available connections
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get maximum connections
    pub fn max_connections(&self) -> usize {
        self.max_connections
    }
}

/// A permit that represents an active connection
/// When dropped, the permit is automatically returned to the pool
pub struct ConnectionPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    max_connections: usize,
    semaphore: Arc<Semaphore>,
}

impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        debug!(
            "Released Ollama connection permit ({}/{} in use)",
            self.max_connections - self.semaphore.available_permits(),
            self.max_connections
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_connection_pool() {
        let pool = OllamaConnectionPool::new(2);

        // Acquire first permit
        let permit1 = pool.acquire().await;
        assert_eq!(pool.available_permits(), 1);

        // Acquire second permit
        let permit2 = pool.acquire().await;
        assert_eq!(pool.available_permits(), 0);

        // Drop first permit
        drop(permit1);
        assert_eq!(pool.available_permits(), 1);

        // Drop second permit
        drop(permit2);
        assert_eq!(pool.available_permits(), 2);
    }

    #[tokio::test]
    async fn test_connection_pool_blocking() {
        let pool = OllamaConnectionPool::new(1);

        // Acquire the only permit
        let _permit = pool.acquire().await;
        assert_eq!(pool.available_permits(), 0);

        // Try to acquire another in a separate task
        let pool_clone = pool.clone();
        let task = tokio::spawn(async move {
            let _permit = pool_clone.acquire().await;
            "acquired"
        });

        // Should not complete immediately
        sleep(Duration::from_millis(10)).await;
        assert!(!task.is_finished());

        // Release the permit
        drop(_permit);

        // Now the task should complete
        let result = task.await.unwrap();
        assert_eq!(result, "acquired");
    }
}
