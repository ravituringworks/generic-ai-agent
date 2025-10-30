//! Type definitions for knowledge management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Source of external knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Web page or article
    Web {
        url: String,
        title: Option<String>,
        crawl_depth: u32,
    },
    /// Document file
    Document {
        path: String,
        format: DocumentFormat,
    },
    /// API endpoint
    Api {
        endpoint: String,
        description: String,
    },
    /// Git repository
    Git {
        repo_url: String,
        branch: String,
        file_patterns: Vec<String>,
    },
    /// RSS/Atom feed
    RSS {
        feed_url: String,
        categories: Vec<String>,
    },
}

impl std::fmt::Display for KnowledgeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Web { url, .. } => write!(f, "Web: {}", url),
            Self::Document { path, .. } => write!(f, "Document: {}", path),
            Self::Api { endpoint, .. } => write!(f, "API: {}", endpoint),
            Self::Git { repo_url, .. } => write!(f, "Git: {}", repo_url),
            Self::RSS { feed_url, .. } => write!(f, "RSS: {}", feed_url),
        }
    }
}

/// Supported document formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DocumentFormat {
    PDF,
    EPUB,
    Markdown,
    HTML,
    DOCX,
    TXT,
    Code { language: String },
}

/// A chunk of knowledge with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    pub id: Uuid,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub source: String,
    pub source_type: String,
    pub metadata: HashMap<String, String>,
    pub quality_score: f32,
    pub created_at: DateTime<Utc>,
}

impl KnowledgeChunk {
    pub fn new(content: String, source: String, source_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            embedding: None,
            source,
            source_type,
            metadata: HashMap::new(),
            quality_score: 0.5, // neutral default
            created_at: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_quality_score(mut self, score: f32) -> Self {
        self.quality_score = score;
        self
    }
}

/// Configuration for knowledge ingestion
#[derive(Debug, Clone)]
pub struct IngestionConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub quality_threshold: f32,
    pub max_chunks: Option<usize>,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            chunk_overlap: 200,
            quality_threshold: 0.6,
            max_chunks: None,
        }
    }
}

/// Result of knowledge ingestion
#[derive(Debug, Clone)]
pub struct IngestionResult {
    pub source: String,
    pub chunks_stored: usize,
    pub chunks_filtered: usize,
    pub timestamp: DateTime<Utc>,
}

/// Consolidated knowledge from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedKnowledge {
    pub id: Uuid,
    pub topic: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub sources: Vec<String>,
    pub confidence_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConsolidatedKnowledge {
    pub fn new(topic: String, summary: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            topic,
            summary,
            key_points: Vec::new(),
            sources: Vec::new(),
            confidence_score: 0.5,
            created_at: now,
            updated_at: now,
        }
    }
}
