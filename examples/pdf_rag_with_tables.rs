//! Advanced RAG Example: PDF Processing with Table Extraction
//!
//! This example demonstrates a comprehensive Retrieval-Augmented Generation (RAG) system
//! that can process PDF documents containing complex tables, extract structured data,
//! and provide intelligent answers based on the document content.
//!
//! Key Features:
//! - PDF text and table extraction
//! - Structured data processing
//! - Vector embeddings for semantic search
//! - RAG-based question answering
//! - Table-aware content parsing

use std::{collections::HashMap, fs, path::Path};
use the_agency::{
    config::MemoryConfig,
    llm::{user_message, LlmClient, OllamaClient},
    memory::{MemoryStore, SqliteMemoryStore},
};

// PDF parsing
use anyhow::{Context, Result};
#[cfg(feature = "pdf")]
use pdf_extract::extract_text;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents extracted content from a PDF document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub sections: Vec<DocumentSection>,
    pub tables: Vec<ExtractedTable>,
    pub figures: Vec<DocumentFigure>,
    pub references: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Represents a section of the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    pub title: String,
    pub level: u8,
    pub content: String,
    pub page_number: Option<u32>,
    pub subsections: Vec<DocumentSection>,
}

/// Represents an extracted table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTable {
    pub table_id: String,
    pub caption: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub page_number: Option<u32>,
    pub table_type: TableType,
    pub context: String, // Surrounding text for context
}

/// Types of tables that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableType {
    DataTable,       // Numerical data tables
    ComparisonTable, // Comparison matrices
    ResultsTable,    // Experimental results
    ParameterTable,  // Configuration/parameter tables
    Unknown,
}

/// Represents a figure or diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFigure {
    pub figure_id: String,
    pub caption: Option<String>,
    pub page_number: Option<u32>,
    pub figure_type: String,
}

/// Advanced PDF processor with table extraction capabilities
pub struct AdvancedPDFProcessor {
    table_patterns: Vec<Regex>,
    section_patterns: Vec<Regex>,
    #[allow(dead_code)]
    reference_patterns: Vec<Regex>,
}

impl Default for AdvancedPDFProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvancedPDFProcessor {
    pub fn new() -> Self {
        let table_patterns = vec![
            Regex::new(r"Table\s+\d+[:.]").unwrap(),
            Regex::new(r"TABLE\s+\d+[:.]").unwrap(),
            Regex::new(r"\|\s*[^\|]+\s*\|").unwrap(), // ASCII table detection
        ];

        let section_patterns = vec![
            Regex::new(r"^\d+\.?\s+[A-Z][^.]*$").unwrap(),
            Regex::new(r"^[A-Z][A-Z\s]+$").unwrap(),
            Regex::new(r"^\d+\.\d+\.?\s+").unwrap(),
        ];

        let reference_patterns = vec![
            Regex::new(r"\[\d+\]").unwrap(),
            Regex::new(r"\(\d{4}\)").unwrap(),
        ];

        Self {
            table_patterns,
            section_patterns,
            reference_patterns,
        }
    }

    /// Extract content from PDF file
    pub async fn extract_pdf_content(&self, pdf_path: &Path) -> Result<DocumentContent> {
        println!("🔍 Extracting content from PDF: {}", pdf_path.display());

        // For this example, we'll use a simplified text-based extraction
        // In a production system, you'd use libraries like pdf-extract or lopdf
        let raw_text = self.extract_raw_text(pdf_path).await?;

        // Parse the document structure
        let content = self.parse_document_structure(&raw_text).await?;

        println!(
            "✅ Extracted {} sections and {} tables",
            content.sections.len(),
            content.tables.len()
        );

        Ok(content)
    }

    /// Extract raw text from PDF
    async fn extract_raw_text(&self, pdf_path: &Path) -> Result<String> {
        // Try real PDF extraction first
        #[cfg(feature = "pdf")]
        {
            match self.extract_real_pdf_text(pdf_path).await {
                Ok(text) => {
                    println!(
                        "📄 Successfully extracted real PDF text: {} chars",
                        text.len()
                    );
                    return Ok(text);
                }
                Err(e) => {
                    println!("⚠️  PDF extraction failed: {}", e);
                }
            }
        }

        // Check if there's a text version of the file for testing
        let text_path = pdf_path.with_extension("txt");
        if text_path.exists() {
            println!("📄 Using text version of PDF");
            return fs::read_to_string(text_path).context("Failed to read text version of PDF");
        }

        // Fallback to simulated content
        println!("⚠️  Falling back to simulated PDF content");
        self.simulate_pdf_extraction(pdf_path).await
    }

    /// Extract text from real PDF file using pdf-extract
    #[cfg(feature = "pdf")]
    async fn extract_real_pdf_text(&self, pdf_path: &Path) -> Result<String> {
        let text = extract_text(pdf_path)
            .with_context(|| format!("Failed to extract text from PDF: {}", pdf_path.display()))?;

        Ok(text)
    }

    /// Simulate PDF text extraction for demonstration
    async fn simulate_pdf_extraction(&self, pdf_path: &Path) -> Result<String> {
        // For demonstration, create sample academic content with tables
        let sample_content = r#"
Efficient Multi-Agent Reinforcement Learning with Communication Networks

Abstract
We present a novel approach to multi-agent reinforcement learning that leverages communication networks to improve coordination and performance. Our method demonstrates significant improvements over baseline approaches across multiple environments.

1. Introduction
Multi-agent reinforcement learning (MARL) has become increasingly important in various domains including robotics, autonomous vehicles, and distributed systems. The key challenge in MARL is enabling effective coordination between agents while maintaining scalability.

2. Related Work
Previous approaches to MARL have focused on centralized training with decentralized execution. However, these methods often struggle with communication overhead and scalability issues.

3. Methodology
Our approach introduces a communication protocol that allows agents to share relevant information while minimizing bandwidth usage.

3.1 Communication Protocol
The communication protocol consists of three main components:
- Message encoding
- Routing strategy  
- Information filtering

Table 1: Experimental Results - Environment Performance
Environment | Baseline | Our Method | Improvement
CartPole    | 180.5    | 195.2      | +8.1%
LunarLander | 245.8    | 278.4      | +13.3%
MultiAgent  | 167.2    | 201.7      | +20.6%

Table 2: Communication Overhead Analysis
Method         | Messages/Episode | Bandwidth (KB) | Latency (ms)
Baseline       | 150             | 12.3          | 5.2
Our Method     | 89              | 7.8           | 3.1
Improvement    | -40.7%          | -36.6%        | -40.4%

4. Experimental Results
We evaluated our approach on three different environments. The results show consistent improvements in both performance and communication efficiency.

4.1 Performance Analysis
Our method achieves superior performance across all tested environments while reducing communication overhead by approximately 40%.

Table 3: Statistical Significance Tests
Environment | p-value | Effect Size | Confidence Interval
CartPole    | 0.001   | 1.23       | [0.85, 1.61]
LunarLander | 0.003   | 0.98       | [0.64, 1.32]
MultiAgent  | 0.0001  | 1.87       | [1.45, 2.29]

5. Conclusion
Our communication-aware MARL approach demonstrates significant improvements in both performance and efficiency. Future work will explore applications to larger multi-agent systems.

References
[1] Smith et al. "Multi-agent learning in complex environments." ICML 2023.
[2] Johnson, A. "Communication protocols for distributed AI." NeurIPS 2023.
[3] Brown et al. "Scalable reinforcement learning." ICLR 2023.
"#;

        println!("📄 Using simulated PDF content for: {}", pdf_path.display());
        Ok(sample_content.to_string())
    }

    /// Parse document structure from raw text
    async fn parse_document_structure(&self, raw_text: &str) -> Result<DocumentContent> {
        let lines: Vec<&str> = raw_text.lines().collect();

        // Extract title (usually the first non-empty line)
        let title = lines
            .iter()
            .find(|line| !line.trim().is_empty())
            .unwrap_or(&"Untitled Document")
            .trim()
            .to_string();

        // Extract abstract
        let abstract_text = self.extract_abstract(&lines);

        // Extract sections
        let sections = self.extract_sections(&lines).await?;

        // Extract tables
        let tables = self.extract_tables(&lines).await?;

        // Extract references
        let references = self.extract_references(&lines);

        Ok(DocumentContent {
            title,
            authors: vec!["Sample Author".to_string()], // Simplified
            abstract_text,
            sections,
            tables,
            figures: vec![], // Simplified
            references,
            metadata: HashMap::new(),
        })
    }

    /// Extract abstract from document lines
    fn extract_abstract(&self, lines: &[&str]) -> String {
        let mut in_abstract = false;
        let mut abstract_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.to_lowercase() == "abstract" {
                in_abstract = true;
                continue;
            }
            if in_abstract {
                if trimmed.is_empty() && !abstract_lines.is_empty() {
                    break;
                }
                if self.section_patterns.iter().any(|p| p.is_match(trimmed))
                    && !abstract_lines.is_empty()
                {
                    break;
                }
                if !trimmed.is_empty() {
                    abstract_lines.push(trimmed);
                }
            }
        }

        abstract_lines.join(" ")
    }

    /// Extract document sections
    async fn extract_sections(&self, lines: &[&str]) -> Result<Vec<DocumentSection>> {
        let mut sections = Vec::new();
        let mut current_section: Option<DocumentSection> = None;
        let mut content_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            // Check if this line is a section header
            if self.is_section_header(trimmed) {
                // Save previous section if exists
                if let Some(mut section) = current_section.take() {
                    section.content = content_lines.join("\n");
                    sections.push(section);
                }

                // Start new section
                current_section = Some(DocumentSection {
                    title: trimmed.to_string(),
                    level: self.get_section_level(trimmed),
                    content: String::new(),
                    page_number: None,
                    subsections: vec![],
                });
                content_lines.clear();
            } else if current_section.is_some() && !trimmed.is_empty() {
                // Add content to current section
                content_lines.push(trimmed);
            }
        }

        // Save last section
        if let Some(mut section) = current_section {
            section.content = content_lines.join("\n");
            sections.push(section);
        }

        Ok(sections)
    }

    /// Check if a line is a section header
    fn is_section_header(&self, line: &str) -> bool {
        self.section_patterns
            .iter()
            .any(|pattern| pattern.is_match(line))
    }

    /// Determine section level (1, 2, 3, etc.)
    fn get_section_level(&self, line: &str) -> u8 {
        if line.starts_with(char::is_numeric) {
            let dots = line
                .chars()
                .take_while(|c| c.is_numeric() || *c == '.')
                .count();
            dots.div_ceil(2).min(6) as u8
        } else {
            1
        }
    }

    /// Extract tables from document
    async fn extract_tables(&self, lines: &[&str]) -> Result<Vec<ExtractedTable>> {
        let mut tables = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Look for table indicators
            if self.table_patterns.iter().any(|p| p.is_match(line)) {
                if let Some(table) = self.parse_table(&lines[i..]).await? {
                    let rows_len = table.rows.len();
                    tables.push(table);
                    // Skip processed lines
                    i += rows_len + 5; // Rough estimate
                }
            }
            i += 1;
        }

        Ok(tables)
    }

    /// Parse a table from lines starting at the given position
    async fn parse_table(&self, lines: &[&str]) -> Result<Option<ExtractedTable>> {
        let mut table_lines = Vec::new();
        let mut caption = None;
        let mut found_table_content = false;

        // Look for table caption
        if lines[0].trim().to_lowercase().starts_with("table") {
            caption = Some(lines[0].trim().to_string());
        }

        // Find table content (lines with | characters or specific patterns)
        for (_idx, line) in lines.iter().enumerate().take(20) {
            // Look ahead max 20 lines
            let trimmed = line.trim();

            if trimmed.contains("|") && trimmed.len() > 5 {
                table_lines.push(trimmed);
                found_table_content = true;
            } else if found_table_content && (trimmed.is_empty() || !trimmed.contains("|")) {
                break;
            }
        }

        if !found_table_content {
            // Look for space-separated tabular data
            for (_idx, line) in lines.iter().enumerate().take(15) {
                let trimmed = line.trim();
                let parts: Vec<&str> = trimmed.split_whitespace().collect();

                // Heuristic: if line has 3+ parts and contains numbers, might be table data
                if parts.len() >= 3 && parts.iter().any(|p| p.chars().any(char::is_numeric)) {
                    table_lines.push(trimmed);
                    found_table_content = true;
                } else if found_table_content && trimmed.is_empty() {
                    break;
                }
            }
        }

        if table_lines.is_empty() {
            return Ok(None);
        }

        // Parse headers and rows
        let (headers, rows) = self.parse_table_content(&table_lines)?;

        let table = ExtractedTable {
            table_id: Uuid::new_v4().to_string(),
            caption,
            headers,
            rows,
            page_number: None,
            table_type: TableType::DataTable, // Simplified classification
            context: lines.first().map(|s| s.to_string()).unwrap_or_default(),
        };

        Ok(Some(table))
    }

    /// Parse table content into headers and rows
    fn parse_table_content(&self, table_lines: &[&str]) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        if table_lines.is_empty() {
            return Ok((vec![], vec![]));
        }

        let mut rows = Vec::new();

        // Parse first line as headers
        let first_line = table_lines[0];
        let headers: Vec<String> = if first_line.contains("|") {
            first_line
                .split("|")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            first_line
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        };

        // Parse remaining lines as data rows
        for line in &table_lines[1..] {
            if line.contains("|") {
                let row: Vec<String> = line
                    .split("|")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !row.is_empty() {
                    rows.push(row);
                }
            } else {
                let row: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
                if row.len() >= headers.len() && !row.is_empty() {
                    rows.push(row);
                }
            }
        }

        Ok((headers, rows))
    }

    /// Extract references from document
    fn extract_references(&self, lines: &[&str]) -> Vec<String> {
        let mut references = Vec::new();
        let mut in_references = false;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.to_lowercase() == "references" || trimmed.to_lowercase() == "bibliography" {
                in_references = true;
                continue;
            }

            if in_references && !trimmed.is_empty() {
                references.push(trimmed.to_string());
            }
        }

        references
    }
}

/// RAG system for PDF documents with table support
pub struct DocumentRAGSystem {
    memory_store: SqliteMemoryStore,
    llm_client: OllamaClient,
    pdf_processor: AdvancedPDFProcessor,
    indexed_documents: HashMap<String, DocumentContent>,
}

impl DocumentRAGSystem {
    pub async fn new(_database_path: &str) -> Result<Self> {
        let memory_config = MemoryConfig {
            store_type: "sqlite".to_string(),
            database_url: Some("sqlite::memory:".to_string()),
            embedding_dimension: 768,
            max_search_results: 10,
            similarity_threshold: 0.7,
            persistent: true,
        };

        let mut memory_store = SqliteMemoryStore::new(memory_config);
        memory_store.initialize().await?;

        let llm_config = the_agency::config::LlmConfig {
            ollama_url: "http://localhost:11434".to_string(),
            text_model: "qwen3-coder:480b-cloud".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            timeout: 120,
            stream: false,
            task_models: HashMap::new(),
            cache: the_agency::cache::LlmCacheConfig::default(),
        };

        let llm_client = OllamaClient::new(llm_config);

        Ok(Self {
            memory_store,
            llm_client,
            pdf_processor: AdvancedPDFProcessor::new(),
            indexed_documents: HashMap::new(),
        })
    }

    /// Index a PDF document for RAG retrieval
    pub async fn index_document(&mut self, pdf_path: &Path) -> Result<String> {
        println!("📚 Indexing document: {}", pdf_path.display());

        // Extract content from PDF
        let document = self.pdf_processor.extract_pdf_content(pdf_path).await?;
        let document_id = Uuid::new_v4().to_string();

        // Index document sections
        for (section_idx, section) in document.sections.iter().enumerate() {
            let content = format!(
                "Document: {}\nSection: {}\nContent: {}",
                document.title, section.title, section.content
            );

            println!(
                "📄 Indexing section '{}': {} chars",
                section.title,
                content.len()
            );

            let embedding = self.llm_client.embed(&content).await?.embedding;

            let metadata = HashMap::from([
                ("document_id".to_string(), document_id.clone()),
                ("document_title".to_string(), document.title.clone()),
                ("section_title".to_string(), section.title.clone()),
                ("section_index".to_string(), section_idx.to_string()),
                ("content_type".to_string(), "section".to_string()),
            ]);

            self.memory_store
                .store(content, embedding, metadata)
                .await?;
        }

        // Index tables separately with structured content
        for (table_idx, table) in document.tables.iter().enumerate() {
            let table_content = self.format_table_for_indexing(table);
            let embedding = self.llm_client.embed(&table_content).await?.embedding;

            let metadata = HashMap::from([
                ("document_id".to_string(), document_id.clone()),
                ("document_title".to_string(), document.title.clone()),
                ("table_id".to_string(), table.table_id.clone()),
                ("table_index".to_string(), table_idx.to_string()),
                ("content_type".to_string(), "table".to_string()),
                (
                    "table_caption".to_string(),
                    table.caption.clone().unwrap_or_default(),
                ),
            ]);

            self.memory_store
                .store(table_content, embedding, metadata)
                .await?;
        }

        // Index abstract separately
        if !document.abstract_text.is_empty() {
            let abstract_content = format!(
                "Document: {}\nAbstract: {}",
                document.title, document.abstract_text
            );

            println!("🎯 Indexing abstract: {} chars", abstract_content.len());

            let embedding = self.llm_client.embed(&abstract_content).await?.embedding;

            let metadata = HashMap::from([
                ("document_id".to_string(), document_id.clone()),
                ("document_title".to_string(), document.title.clone()),
                ("content_type".to_string(), "abstract".to_string()),
            ]);

            self.memory_store
                .store(abstract_content, embedding, metadata)
                .await?;
        }

        // Store document for reference
        self.indexed_documents.insert(document_id.clone(), document);

        println!(
            "✅ Successfully indexed document with {} sections and {} tables",
            self.indexed_documents[&document_id].sections.len(),
            self.indexed_documents[&document_id].tables.len()
        );

        Ok(document_id)
    }

    /// Format table for indexing with structured representation
    fn format_table_for_indexing(&self, table: &ExtractedTable) -> String {
        let mut formatted = String::new();

        if let Some(caption) = &table.caption {
            formatted.push_str(&format!("Table Caption: {}\n", caption));
        }

        // Add headers
        if !table.headers.is_empty() {
            formatted.push_str(&format!("Headers: {}\n", table.headers.join(" | ")));
        }

        // Add rows in a readable format
        for (row_idx, row) in table.rows.iter().enumerate() {
            if !table.headers.is_empty() && table.headers.len() == row.len() {
                // Create key-value pairs
                let row_data: Vec<String> = table
                    .headers
                    .iter()
                    .zip(row.iter())
                    .map(|(header, value)| format!("{}: {}", header, value))
                    .collect();
                formatted.push_str(&format!("Row {}: {}\n", row_idx + 1, row_data.join(", ")));
            } else {
                formatted.push_str(&format!("Row {}: {}\n", row_idx + 1, row.join(" | ")));
            }
        }

        // Add context
        if !table.context.is_empty() {
            formatted.push_str(&format!("Context: {}\n", table.context));
        }

        formatted
    }

    /// Answer a question using RAG with document and table context
    pub async fn answer_question(&self, question: &str) -> Result<String> {
        println!("🤔 Processing question: {}", question);

        // Create embedding for the question
        let question_embedding = self.llm_client.embed(question).await?.embedding;

        // Retrieve relevant content (limit to 3 results for faster processing)
        let search_results = self.memory_store.search(question_embedding, 3, 0.3).await?;

        println!("🔍 Found {} search results", search_results.len());
        for (i, result) in search_results.iter().enumerate() {
            println!(
                "  Result {}: similarity={:.3}, type={}",
                i + 1,
                result.similarity,
                result
                    .entry
                    .metadata
                    .get("content_type")
                    .unwrap_or(&"unknown".to_string())
            );
        }

        if search_results.is_empty() {
            return Ok("I couldn't find relevant information in the indexed documents to answer your question.".to_string());
        }

        // Prepare context from search results
        let mut context_parts = Vec::new();
        let mut table_contexts = Vec::new();

        for result in &search_results {
            if let Some(content_type) = result.entry.metadata.get("content_type") {
                match content_type.as_str() {
                    "table" => {
                        // Truncate table content if too long
                        let content = if result.entry.content.len() > 800 {
                            format!("{}...[truncated]", &result.entry.content[..800])
                        } else {
                            result.entry.content.clone()
                        };
                        table_contexts.push(format!(
                            "TABLE from {}:\n{}\n(Similarity: {:.3})",
                            result
                                .entry
                                .metadata
                                .get("document_title")
                                .unwrap_or(&"Unknown".to_string()),
                            content,
                            result.similarity
                        ));
                    }
                    _ => {
                        // Truncate section content if too long
                        let content = if result.entry.content.len() > 600 {
                            format!("{}...[truncated]", &result.entry.content[..600])
                        } else {
                            result.entry.content.clone()
                        };
                        context_parts.push(format!(
                            "From {} - {}:\n{}\n(Similarity: {:.3})",
                            result
                                .entry
                                .metadata
                                .get("document_title")
                                .unwrap_or(&"Unknown".to_string()),
                            result
                                .entry
                                .metadata
                                .get("section_title")
                                .unwrap_or(&"Unknown Section".to_string()),
                            content,
                            result.similarity
                        ));
                    }
                }
            }
        }

        // Combine all context
        let mut full_context = String::new();
        if !context_parts.is_empty() {
            full_context.push_str("RELEVANT DOCUMENT SECTIONS:\n");
            full_context.push_str(&context_parts.join("\n---\n"));
            full_context.push_str("\n\n");
        }

        if !table_contexts.is_empty() {
            full_context.push_str("RELEVANT TABLES:\n");
            full_context.push_str(&table_contexts.join("\n---\n"));
            full_context.push_str("\n\n");
        }

        // Generate response using LLM
        let system_prompt = "You are an AI assistant specialized in analyzing academic and technical documents. You have access to document content including text sections and structured tables. When answering questions:

1. Use information from both text sections and tables when relevant
2. When referencing tables, clearly explain the data and its significance
3. Provide specific details and numbers when available in tables
4. If comparing values, reference the exact table data
5. Be precise about what the tables show and their context
6. If you can't find sufficient information, say so clearly

Answer the user's question based on the provided context.";

        let messages = vec![
            the_agency::llm::system_message(system_prompt),
            user_message(format!(
                "Context:\n{}\n\nQuestion: {}",
                full_context, question
            )),
        ];

        let response = self.llm_client.generate(&messages).await?;

        println!(
            "✅ Generated response based on {} search results ({} table results)",
            search_results.len(),
            table_contexts.len()
        );

        Ok(response.text)
    }

    /// Get statistics about indexed documents
    pub fn get_statistics(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        stats.insert(
            "total_documents".to_string(),
            serde_json::Value::Number(self.indexed_documents.len().into()),
        );

        let total_sections: usize = self
            .indexed_documents
            .values()
            .map(|doc| doc.sections.len())
            .sum();
        stats.insert(
            "total_sections".to_string(),
            serde_json::Value::Number(total_sections.into()),
        );

        let total_tables: usize = self
            .indexed_documents
            .values()
            .map(|doc| doc.tables.len())
            .sum();
        stats.insert(
            "total_tables".to_string(),
            serde_json::Value::Number(total_tables.into()),
        );

        stats
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Advanced PDF RAG System with Table Extraction");
    println!("=================================================");

    // Initialize RAG system (use tmp directory for database)
    let mut rag_system = DocumentRAGSystem::new("/tmp/pdf_rag_database.db").await?;

    // Index the provided PDF
    let pdf_path = Path::new("examples/files/2409.05125v1.pdf");
    if pdf_path.exists() {
        println!("\n📖 Indexing PDF document...");
        let document_id = rag_system.index_document(pdf_path).await?;
        println!("Document indexed with ID: {}", document_id);

        // Display statistics
        let stats = rag_system.get_statistics();
        println!("\n📊 Index Statistics:");
        for (key, value) in &stats {
            println!("  • {}: {}", key, value);
        }

        println!("\n💬 Interactive Q&A Session");
        println!("Ask questions about the document. Type 'quit' to exit.");
        println!("Example questions:");
        println!("  • What is this paper about?");
        println!("  • What are the main experimental results?");
        println!("  • Can you explain the performance comparison table?");
        println!("  • What improvements does the method show?");
        println!("  • What are the statistical significance results?");

        // Interactive question-answering loop
        use std::io::{self, Write};

        loop {
            print!("\n❓ Your question: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let question = input.trim();

            if question.is_empty() {
                continue;
            }

            if question.eq_ignore_ascii_case("quit") || question.eq_ignore_ascii_case("exit") {
                println!("👋 Goodbye!");
                break;
            }

            println!("\n🔍 Searching for relevant information...");
            match rag_system.answer_question(question).await {
                Ok(answer) => {
                    println!("\n🤖 Answer:");
                    println!("{}", answer);
                }
                Err(e) => {
                    println!("❌ Error: {}", e);
                }
            }
        }
    } else {
        println!("❌ PDF file not found at: {}", pdf_path.display());
        println!("Please ensure the PDF file exists at the specified location.");

        // Demo with sample questions and synthetic content
        println!("\n🎭 Running demo mode with synthetic content...");
        demo_with_synthetic_content().await?;
    }

    Ok(())
}

/// Demo function with synthetic content to show RAG capabilities
async fn demo_with_synthetic_content() -> Result<()> {
    println!("This would demonstrate RAG with synthetic academic content including tables.");
    println!("The full functionality requires the PDF file to be present.");

    // You could add synthetic document creation here for testing
    println!("\n📚 Demo Features:");
    println!("  ✅ PDF text extraction");
    println!("  ✅ Table detection and parsing");
    println!("  ✅ Section-based content indexing");
    println!("  ✅ Vector embeddings for semantic search");
    println!("  ✅ Table-aware question answering");
    println!("  ✅ Multi-modal content retrieval");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_pdf_processor_creation() {
        let processor = AdvancedPDFProcessor::new();
        assert!(!processor.table_patterns.is_empty());
        assert!(!processor.section_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_table_parsing() {
        let processor = AdvancedPDFProcessor::new();
        let table_lines = vec![
            "Method | Accuracy | F1-Score",
            "Baseline | 0.85 | 0.82",
            "Our Method | 0.92 | 0.89",
        ];

        let (headers, rows) = processor.parse_table_content(&table_lines).unwrap();
        assert_eq!(headers.len(), 3);
        assert_eq!(rows.len(), 2);
        assert_eq!(headers[0], "Method");
        assert_eq!(rows[0][1], "0.85");
    }

    #[tokio::test]
    async fn test_rag_system_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let rag_system = DocumentRAGSystem::new(db_path.to_str().unwrap()).await;
        assert!(rag_system.is_ok());
    }
}
