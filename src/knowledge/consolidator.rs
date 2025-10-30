//! Knowledge consolidation for deduplication and synthesis

use super::types::{ConsolidatedKnowledge, KnowledgeChunk};
use crate::error::Result;
use crate::memory::SqliteMemoryStore;

/// Consolidates and deduplicates knowledge chunks
pub struct KnowledgeConsolidator {
    similarity_threshold: f32,
}

impl KnowledgeConsolidator {
    pub fn new(similarity_threshold: f32) -> Self {
        Self {
            similarity_threshold,
        }
    }

    /// Find and remove duplicate chunks based on semantic similarity
    pub fn deduplicate(&self, mut chunks: Vec<KnowledgeChunk>) -> Vec<KnowledgeChunk> {
        if chunks.len() < 2 {
            return chunks;
        }

        let mut unique_chunks = Vec::new();
        
        for chunk in chunks.drain(..) {
            // Check if this chunk is similar to any existing unique chunk
            let is_duplicate = unique_chunks.iter().any(|existing: &KnowledgeChunk| {
                if let (Some(emb1), Some(emb2)) = (&chunk.embedding, &existing.embedding) {
                    let similarity = SqliteMemoryStore::cosine_similarity(emb1, emb2);
                    similarity >= self.similarity_threshold
                } else {
                    // Fall back to text similarity if embeddings not available
                    self.text_similarity(&chunk.content, &existing.content) >= 0.9
                }
            });

            if !is_duplicate {
                unique_chunks.push(chunk);
            }
        }

        unique_chunks
    }

    /// Calculate simple text similarity (Jaccard similarity on words)
    fn text_similarity(&self, text1: &str, text2: &str) -> f32 {
        let words1: std::collections::HashSet<_> = 
            text1.split_whitespace().map(|w| w.to_lowercase()).collect();
        let words2: std::collections::HashSet<_> = 
            text2.split_whitespace().map(|w| w.to_lowercase()).collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Rank chunks by quality score
    pub fn rank_by_quality(&self, mut chunks: Vec<KnowledgeChunk>) -> Vec<KnowledgeChunk> {
        chunks.sort_by(|a, b| {
            b.quality_score
                .partial_cmp(&a.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        chunks
    }

    /// Synthesize multiple chunks into consolidated knowledge
    pub fn synthesize(
        &self,
        topic: &str,
        chunks: Vec<KnowledgeChunk>,
    ) -> Result<ConsolidatedKnowledge> {
        if chunks.is_empty() {
            return Ok(ConsolidatedKnowledge::new(
                topic.to_string(),
                "No knowledge available".to_string(),
            ));
        }

        // Combine unique sources
        let mut sources: Vec<String> = chunks
            .iter()
            .map(|c| c.source.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        sources.sort();

        // Extract key points (first sentence from each chunk)
        let key_points: Vec<String> = chunks
            .iter()
            .filter_map(|c| {
                c.content
                    .lines()
                    .next()
                    .filter(|line| !line.trim().is_empty())
                    .map(|s| s.to_string())
            })
            .take(10) // Limit to top 10 key points
            .collect();

        // Create summary from top chunks
        let summary = if chunks.len() <= 3 {
            // For small number of chunks, concatenate them
            chunks
                .iter()
                .map(|c| c.content.clone())
                .collect::<Vec<_>>()
                .join("\n\n")
        } else {
            // For many chunks, create a brief summary
            format!(
                "Knowledge consolidated from {} sources covering: {}",
                sources.len(),
                key_points.join("; ")
            )
        };

        // Calculate confidence score based on quality and source count
        let avg_quality: f32 = chunks.iter().map(|c| c.quality_score).sum::<f32>() 
            / chunks.len() as f32;
        let source_boost = (sources.len() as f32).min(5.0) / 5.0; // Max boost from 5 sources
        let confidence_score = (avg_quality * 0.7 + source_boost * 0.3).min(1.0);

        let mut consolidated = ConsolidatedKnowledge::new(topic.to_string(), summary);
        consolidated.key_points = key_points;
        consolidated.sources = sources;
        consolidated.confidence_score = confidence_score;

        Ok(consolidated)
    }

    /// Full consolidation pipeline
    pub fn consolidate(
        &self,
        topic: &str,
        chunks: Vec<KnowledgeChunk>,
    ) -> Result<ConsolidatedKnowledge> {
        // 1. Deduplicate
        let unique_chunks = self.deduplicate(chunks);

        // 2. Rank by quality
        let ranked_chunks = self.rank_by_quality(unique_chunks);

        // 3. Synthesize
        self.synthesize(topic, ranked_chunks)
    }
}

impl Default for KnowledgeConsolidator {
    fn default() -> Self {
        Self::new(0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_empty() {
        let consolidator = KnowledgeConsolidator::default();
        let chunks = vec![];
        let result = consolidator.deduplicate(chunks);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_text_similarity() {
        let consolidator = KnowledgeConsolidator::default();
        
        let sim1 = consolidator.text_similarity(
            "the quick brown fox",
            "the quick brown fox"
        );
        assert_eq!(sim1, 1.0);

        let sim2 = consolidator.text_similarity(
            "the quick brown fox",
            "completely different text"
        );
        assert!(sim2 < 0.3);

        let sim3 = consolidator.text_similarity(
            "the quick brown fox",
            "the quick brown dog"
        );
        assert!(sim3 > 0.5 && sim3 < 1.0);
    }

    #[test]
    fn test_rank_by_quality() {
        let consolidator = KnowledgeConsolidator::default();
        
        let chunks = vec![
            KnowledgeChunk::new("text1".to_string(), "s1".to_string(), "t1".to_string())
                .with_quality_score(0.5),
            KnowledgeChunk::new("text2".to_string(), "s2".to_string(), "t2".to_string())
                .with_quality_score(0.9),
            KnowledgeChunk::new("text3".to_string(), "s3".to_string(), "t3".to_string())
                .with_quality_score(0.7),
        ];

        let ranked = consolidator.rank_by_quality(chunks);
        
        assert_eq!(ranked[0].quality_score, 0.9);
        assert_eq!(ranked[1].quality_score, 0.7);
        assert_eq!(ranked[2].quality_score, 0.5);
    }

    #[test]
    fn test_synthesize() {
        let consolidator = KnowledgeConsolidator::default();
        
        let chunks = vec![
            KnowledgeChunk::new(
                "First key point about Rust".to_string(),
                "source1".to_string(),
                "web".to_string()
            ).with_quality_score(0.8),
            KnowledgeChunk::new(
                "Second key point about Rust".to_string(),
                "source2".to_string(),
                "web".to_string()
            ).with_quality_score(0.9),
        ];

        let result = consolidator.synthesize("Rust Programming", chunks).unwrap();
        
        assert_eq!(result.topic, "Rust Programming");
        assert_eq!(result.sources.len(), 2);
        assert_eq!(result.key_points.len(), 2);
        assert!(result.confidence_score > 0.0);
    }
}
