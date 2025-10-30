//! Content chunking for text processing

use super::types::{IngestionConfig, KnowledgeChunk};

/// Content chunker for splitting text into manageable pieces
pub struct ContentChunker {
    config: IngestionConfig,
}

impl ContentChunker {
    pub fn new(config: IngestionConfig) -> Self {
        Self { config }
    }

    /// Chunk text content with overlap for context preservation
    pub fn chunk_text(&self, text: &str, source: String, source_type: String) -> Vec<KnowledgeChunk> {
        let chunk_size = self.config.chunk_size;
        let overlap = self.config.chunk_overlap;

        if text.len() <= chunk_size {
            return vec![KnowledgeChunk::new(text.to_string(), source, source_type)];
        }

        let mut chunks = Vec::new();
        let mut start = 0;

        while start < text.len() {
            let end = (start + chunk_size).min(text.len());
            
            // Try to break at sentence boundary
            let chunk_text = if end < text.len() {
                self.find_sentence_boundary(&text[start..end])
            } else {
                text[start..end].to_string()
            };

            if !chunk_text.trim().is_empty() {
                chunks.push(KnowledgeChunk::new(
                    chunk_text.clone(),
                    source.clone(),
                    source_type.clone(),
                ));
            }

            // Move start forward, accounting for overlap
            // Ensure we always make progress to avoid infinite loops
            let actual_chunk_len = chunk_text.len();
            let advance = actual_chunk_len.saturating_sub(overlap).max(1);
            start += advance;
            
            // Check if we've processed all text or reached max chunks
            if start >= text.len() || (self.config.max_chunks.is_some() && chunks.len() >= self.config.max_chunks.unwrap()) {
                break;
            }
        }

        // Limit chunks if configured
        if let Some(max_chunks) = self.config.max_chunks {
            chunks.truncate(max_chunks);
        }

        chunks
    }

    /// Find a good sentence boundary to avoid mid-sentence splits
    fn find_sentence_boundary(&self, text: &str) -> String {
        // Look for sentence endings in last 100 chars
        let search_start = text.len().saturating_sub(100);
        let search_text = &text[search_start..];

        // Find last sentence ending
        if let Some(pos) = search_text.rfind(|c: char| c == '.' || c == '!' || c == '?') {
            return text[..search_start + pos + 1].to_string();
        }

        // Fall back to word boundary
        if let Some(pos) = text.rfind(char::is_whitespace) {
            return text[..pos].to_string();
        }

        // No good boundary found, return as is
        text.to_string()
    }

    /// Chunk markdown content, preserving structure
    pub fn chunk_markdown(&self, markdown: &str, source: String, source_type: String) -> Vec<KnowledgeChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_size = 0;

        for line in markdown.lines() {
            let line_len = line.len() + 1; // +1 for newline

            // Start new chunk on headers if current chunk is large enough
            if line.starts_with('#') && current_size > self.config.chunk_size / 2 {
                if !current_chunk.trim().is_empty() {
                    chunks.push(KnowledgeChunk::new(
                        current_chunk.trim().to_string(),
                        source.clone(),
                        source_type.clone(),
                    ));
                }
                current_chunk = String::new();
                current_size = 0;
            }

            current_chunk.push_str(line);
            current_chunk.push('\n');
            current_size += line_len;

            // Split if chunk gets too large
            if current_size >= self.config.chunk_size {
                chunks.push(KnowledgeChunk::new(
                    current_chunk.trim().to_string(),
                    source.clone(),
                    source_type.clone(),
                ));
                current_chunk = String::new();
                current_size = 0;
            }
        }

        // Add remaining content
        if !current_chunk.trim().is_empty() {
            chunks.push(KnowledgeChunk::new(
                current_chunk.trim().to_string(),
                source,
                source_type,
            ));
        }

        chunks
    }

    /// Chunk code, preserving function boundaries
    pub fn chunk_code(&self, code: &str, language: &str, source: String) -> Vec<KnowledgeChunk> {
        // For now, use simple text chunking
        // TODO: Implement language-aware chunking using tree-sitter
        self.chunk_text(code, source, format!("code:{}", language))
    }
}

impl Default for ContentChunker {
    fn default() -> Self {
        Self::new(IngestionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_short_text() {
        let chunker = ContentChunker::default();
        let text = "This is a short text.";
        let chunks = chunker.chunk_text(text, "test".to_string(), "test".to_string());
        
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
    }

    #[test]
    fn test_chunk_long_text() {
        let mut config = IngestionConfig::default();
        config.chunk_size = 50;
        config.chunk_overlap = 10;
        config.max_chunks = Some(5); // Limit to avoid memory issues
        
        let chunker = ContentChunker::new(config);
        let text = "This is sentence one. This is sentence two. This is sentence three. This is sentence four.";
        let chunks = chunker.chunk_text(text, "test".to_string(), "test".to_string());
        
        assert!(chunks.len() > 1, "Long text should be chunked");
        assert!(chunks.len() <= 5, "Should respect max_chunks limit");
    }

    #[test]
    fn test_chunk_markdown() {
        let chunker = ContentChunker::default();
        let markdown = "# Header 1\nContent 1\n\n## Header 2\nContent 2\n\n### Header 3\nContent 3";
        let chunks = chunker.chunk_markdown(markdown, "test".to_string(), "markdown".to_string());
        
        assert!(!chunks.is_empty(), "Should produce chunks");
        assert!(chunks[0].content.contains("Header"), "Should preserve headers");
    }
}
