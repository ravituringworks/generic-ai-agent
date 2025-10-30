//! Knowledge ingestion and management system
//!
//! This module provides functionality for:
//! - Ingesting external knowledge from web, documents, code repos
//! - Chunking content for embedding
//! - Consolidating and deduplicating knowledge
//! - Managing knowledge lifecycle with adaptive limits

pub mod chunker;
pub mod consolidator;
pub mod manager;
pub mod types;

pub use chunker::ContentChunker;
pub use consolidator::KnowledgeConsolidator;
pub use manager::{AdaptiveKnowledgeManager, KnowledgeStats, ManagementResult};
pub use types::*;
