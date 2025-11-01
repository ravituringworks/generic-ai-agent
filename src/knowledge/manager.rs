//! Adaptive knowledge management with pruning and retention

use crate::config::LearningConfig;
use crate::error::Result;
use crate::memory::{MemoryEntry, MemoryStore};
use chrono::{DateTime, Utc};

/// Manages knowledge lifecycle with adaptive limits
pub struct AdaptiveKnowledgeManager {
    config: LearningConfig,
}

impl AdaptiveKnowledgeManager {
    pub fn new(config: LearningConfig) -> Self {
        Self { config }
    }

    /// Check if knowledge store needs management
    pub async fn needs_management(&self, _role: &str, store: &dyn MemoryStore) -> Result<bool> {
        let stats = store.stats().await?;
        Ok(stats.total_memories >= self.config.soft_limit_best_practices)
    }

    /// Manage knowledge store based on current size
    pub async fn manage_knowledge(
        &self,
        role: &str,
        store: &mut Box<dyn MemoryStore>,
    ) -> Result<ManagementResult> {
        let stats = store.stats().await?;
        let count = stats.total_memories;

        if count < self.config.soft_limit_best_practices {
            return Ok(ManagementResult::NoActionNeeded {
                current_count: count,
            });
        }

        if count >= self.config.hard_limit_best_practices {
            // Aggressive pruning
            let pruned = self.prune_aggressively(role, store).await?;
            Ok(ManagementResult::Pruned {
                removed_count: pruned,
                remaining_count: stats.total_memories - pruned,
            })
        } else if self.config.enable_auto_consolidation {
            // Consolidation mode
            Ok(ManagementResult::ConsolidationNeeded {
                current_count: count,
                threshold: self.config.soft_limit_best_practices,
            })
        } else {
            Ok(ManagementResult::NoActionNeeded {
                current_count: count,
            })
        }
    }

    /// Prune low-value knowledge aggressively
    async fn prune_aggressively(
        &self,
        role: &str,
        store: &mut Box<dyn MemoryStore>,
    ) -> Result<usize> {
        // Get all memories for this role
        let all_memories = store.list(None).await?;

        let role_memories: Vec<_> = all_memories
            .into_iter()
            .filter(|m| m.metadata.get("role").map(|r| r == role).unwrap_or(false))
            .collect();

        if role_memories.is_empty() {
            return Ok(0);
        }

        // Calculate retention scores
        let mut scored_memories: Vec<_> = role_memories
            .into_iter()
            .map(|m| {
                let score = self.calculate_retention_score(&m);
                (score, m)
            })
            .collect();

        // Sort by score (highest first)
        scored_memories.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Keep top N based on soft limit
        let to_keep_count = self.config.soft_limit_best_practices;
        let to_prune = if scored_memories.len() > to_keep_count {
            scored_memories.split_off(to_keep_count)
        } else {
            vec![]
        };

        // Delete low-scoring memories
        let pruned_count = to_prune.len();
        for (_, memory) in to_prune {
            store.delete(memory.id).await?;
        }

        Ok(pruned_count)
    }

    /// Calculate retention score for a memory entry
    fn calculate_retention_score(&self, entry: &MemoryEntry) -> f32 {
        // Parse metadata
        let reuse_count = entry
            .metadata
            .get("reuse_count")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let quality_score = entry
            .metadata
            .get("quality_score")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.5);

        let created_at = entry
            .metadata
            .get("timestamp")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // Calculate recency score (0.0 - 1.0)
        let recency_score = if let Some(created) = created_at {
            let age_days = (Utc::now() - created).num_days();
            let max_age = self.config.max_age_days_if_unused;
            1.0 - (age_days as f32 / max_age as f32).min(1.0)
        } else {
            0.5
        };

        // Calculate usage score (0.0 - 1.0)
        let usage_score = (reuse_count as f32 / 10.0).min(1.0);

        // Weighted composite score
        (recency_score * 0.2) + (usage_score * 0.4) + (quality_score * 0.4)
    }

    /// Filter memories that should be kept based on retention policy
    pub fn should_keep(&self, entry: &MemoryEntry) -> bool {
        // Parse reuse count
        let reuse_count = entry
            .metadata
            .get("reuse_count")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        // Check minimum reuse count
        if reuse_count >= self.config.min_reuse_count_to_keep {
            return true;
        }

        // Parse quality score
        let quality_score = entry
            .metadata
            .get("quality_score")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);

        // Check minimum quality score
        if quality_score >= self.config.min_quality_score_to_keep {
            return true;
        }

        // Check age
        if let Some(timestamp) = entry.metadata.get("timestamp") {
            if let Ok(created) = DateTime::parse_from_rfc3339(timestamp) {
                let age = (Utc::now() - created.with_timezone(&Utc)).num_days();

                // If recently created, give it a chance
                if age < 7 {
                    return true;
                }

                // If old and unused, can be pruned
                if age > self.config.max_age_days_if_unused && reuse_count == 0 {
                    return false;
                }
            }
        }

        // Default to keeping if uncertain
        true
    }

    /// Get statistics about knowledge storage
    pub async fn get_storage_stats(
        &self,
        role: &str,
        store: &dyn MemoryStore,
    ) -> Result<KnowledgeStats> {
        let all_memories = store.list(None).await?;

        let role_memories: Vec<_> = all_memories
            .into_iter()
            .filter(|m| m.metadata.get("role").map(|r| r == role).unwrap_or(false))
            .collect();

        let total_count = role_memories.len();

        let high_quality_count = role_memories
            .iter()
            .filter(|m| {
                m.metadata
                    .get("quality")
                    .map(|q| q == "high")
                    .unwrap_or(false)
            })
            .count();

        let avg_quality = if !role_memories.is_empty() {
            let sum: f32 = role_memories
                .iter()
                .filter_map(|m| {
                    m.metadata
                        .get("quality_score")
                        .and_then(|s| s.parse::<f32>().ok())
                })
                .sum();
            sum / role_memories.len() as f32
        } else {
            0.0
        };

        Ok(KnowledgeStats {
            role: role.to_string(),
            total_memories: total_count,
            high_quality_count,
            avg_quality_score: avg_quality,
            soft_limit: self.config.soft_limit_best_practices,
            hard_limit: self.config.hard_limit_best_practices,
            utilization: total_count as f32 / self.config.soft_limit_best_practices as f32,
        })
    }
}

/// Result of knowledge management operation
#[derive(Debug, Clone)]
pub enum ManagementResult {
    NoActionNeeded {
        current_count: usize,
    },
    ConsolidationNeeded {
        current_count: usize,
        threshold: usize,
    },
    Pruned {
        removed_count: usize,
        remaining_count: usize,
    },
}

/// Statistics about knowledge storage for a role
#[derive(Debug, Clone)]
pub struct KnowledgeStats {
    pub role: String,
    pub total_memories: usize,
    pub high_quality_count: usize,
    pub avg_quality_score: f32,
    pub soft_limit: usize,
    pub hard_limit: usize,
    pub utilization: f32,
}

impl KnowledgeStats {
    pub fn is_over_limit(&self) -> bool {
        self.total_memories >= self.soft_limit
    }

    pub fn needs_pruning(&self) -> bool {
        self.total_memories >= self.hard_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryEntry;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_entry(quality_score: f32, reuse_count: u32) -> MemoryEntry {
        let mut metadata = HashMap::new();
        metadata.insert("role".to_string(), "test".to_string());
        metadata.insert("quality_score".to_string(), quality_score.to_string());
        metadata.insert("reuse_count".to_string(), reuse_count.to_string());
        metadata.insert("timestamp".to_string(), Utc::now().to_rfc3339());

        MemoryEntry {
            id: Uuid::new_v4(),
            content: "test content".to_string(),
            embedding: vec![],
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_retention_score() {
        let config = LearningConfig::default();
        let manager = AdaptiveKnowledgeManager::new(config);

        let high_quality_entry = create_test_entry(0.9, 5);
        let score1 = manager.calculate_retention_score(&high_quality_entry);
        assert!(score1 > 0.5, "High quality entry should have high score");

        let low_quality_entry = create_test_entry(0.3, 0);
        let score2 = manager.calculate_retention_score(&low_quality_entry);
        assert!(score2 < score1, "Low quality entry should have lower score");
    }

    #[test]
    fn test_should_keep() {
        let config = LearningConfig::default();
        let manager = AdaptiveKnowledgeManager::new(config);

        let high_reuse = create_test_entry(0.5, 10);
        assert!(
            manager.should_keep(&high_reuse),
            "High reuse should be kept"
        );

        let high_quality = create_test_entry(0.9, 0);
        assert!(
            manager.should_keep(&high_quality),
            "High quality should be kept"
        );

        let low_everything = create_test_entry(0.3, 0);
        // Still true because recently created
        assert!(manager.should_keep(&low_everything));
    }
}
