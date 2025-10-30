//! Web content fetcher using MCP browser tools
//!
//! This module provides functionality to fetch and extract content from web sources
//! using MCP server tools (e.g., browser automation, web scraping).

use crate::error::Result;
use crate::mcp::{McpClient, ToolCall, ToolContent};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Configuration for web content fetching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetcherConfig {
    /// Maximum content size to fetch (in bytes)
    pub max_content_size: usize,
    /// User agent string for requests
    pub user_agent: String,
    /// Timeout for fetch operations (seconds)
    pub timeout: u64,
    /// Whether to extract main content only (remove nav, ads, etc.)
    pub extract_main_content: bool,
    /// Whether to follow redirects
    pub follow_redirects: bool,
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            max_content_size: 1024 * 1024, // 1MB
            user_agent: "the-agency/0.1.0".to_string(),
            timeout: 30,
            extract_main_content: true,
            follow_redirects: true,
        }
    }
}

/// A fetched web content item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedContent {
    /// Source URL
    pub url: String,
    /// Content text
    pub content: String,
    /// Title if available
    pub title: Option<String>,
    /// Metadata (author, date, etc.)
    pub metadata: serde_json::Value,
    /// Content type (html, markdown, text, etc.)
    pub content_type: String,
    /// Timestamp of fetch
    pub fetched_at: String,
}

/// Web content fetcher
pub struct WebFetcher {
    config: FetcherConfig,
}

impl WebFetcher {
    /// Create a new web fetcher
    pub fn new(config: FetcherConfig) -> Self {
        Self { config }
    }

    /// Fetch content from a URL using MCP client
    pub async fn fetch_url(
        &self,
        mcp_client: &McpClient,
        url: &str,
    ) -> Result<FetchedContent> {
        info!("Fetching content from URL: {}", url);

        // First try to use a browser automation tool if available
        if let Some(content) = self.try_browser_fetch(mcp_client, url).await? {
            return Ok(content);
        }

        // Fallback to simple HTTP fetch tool
        if let Some(content) = self.try_http_fetch(mcp_client, url).await? {
            return Ok(content);
        }

        Err(crate::error::McpError::ToolNotFound(
            "No suitable fetch tool available (fetch, http_get, curl, browser_navigate)".to_string(),
        ).into())
    }

    /// Try to fetch using browser automation tool (e.g., puppeteer, playwright)
    async fn try_browser_fetch(
        &self,
        mcp_client: &McpClient,
        url: &str,
    ) -> Result<Option<FetchedContent>> {
        // Check if browser tools are available
        let browser_tools = ["browser_navigate", "puppeteer_fetch", "playwright_fetch"];

        for tool_name in &browser_tools {
            if mcp_client.find_tool_server(tool_name).is_some() {
                debug!("Using browser tool: {}", tool_name);

                let tool_call = ToolCall {
                    id: Uuid::new_v4().to_string(),
                    name: tool_name.to_string(),
                    arguments: serde_json::json!({
                        "url": url,
                        "wait_for": "networkidle",
                        "extract_content": self.config.extract_main_content,
                        "timeout": self.config.timeout * 1000, // Convert to milliseconds
                    }),
                };

                match mcp_client.call_tool(tool_call).await {
                    Ok(result) if !result.is_error => {
                        return Ok(Some(self.parse_browser_result(url, result.content)?));
                    }
                    Ok(_) => warn!("Browser tool returned error, trying next option"),
                    Err(e) => warn!("Browser tool failed: {}, trying next option", e),
                }
            }
        }

        Ok(None)
    }

    /// Try to fetch using simple HTTP fetch tool
    async fn try_http_fetch(
        &self,
        mcp_client: &McpClient,
        url: &str,
    ) -> Result<Option<FetchedContent>> {
        let http_tools = ["fetch", "http_get", "curl"];

        for tool_name in &http_tools {
            if mcp_client.find_tool_server(tool_name).is_some() {
                debug!("Using HTTP tool: {}", tool_name);

                let tool_call = ToolCall {
                    id: Uuid::new_v4().to_string(),
                    name: tool_name.to_string(),
                    arguments: serde_json::json!({
                        "url": url,
                        "timeout": self.config.timeout,
                        "user_agent": self.config.user_agent,
                        "follow_redirects": self.config.follow_redirects,
                    }),
                };

                match mcp_client.call_tool(tool_call).await {
                    Ok(result) if !result.is_error => {
                        return Ok(Some(self.parse_http_result(url, result.content)?));
                    }
                    Ok(_) => warn!("HTTP tool returned error, trying next option"),
                    Err(e) => warn!("HTTP tool failed: {}, trying next option", e),
                }
            }
        }

        Ok(None)
    }

    /// Parse browser automation result
    fn parse_browser_result(
        &self,
        url: &str,
        content: Vec<ToolContent>,
    ) -> Result<FetchedContent> {
        let mut text_content = String::new();
        let mut title = None;
        let mut metadata = serde_json::json!({});

        for item in content {
            match item {
                ToolContent::Text { text } => {
                    // Try to parse as JSON first for structured response
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(content_text) = json.get("content").and_then(|c| c.as_str()) {
                            text_content = content_text.to_string();
                        }
                        if let Some(title_text) = json.get("title").and_then(|t| t.as_str()) {
                            title = Some(title_text.to_string());
                        }
                        if let Some(meta) = json.get("metadata") {
                            metadata = meta.clone();
                        }
                    } else {
                        // Plain text response
                        text_content = text;
                    }
                }
                ToolContent::Resource { text: Some(text), .. } => {
                    text_content.push_str(&text);
                }
                _ => {}
            }
        }

        // Truncate if too large
        if text_content.len() > self.config.max_content_size {
            warn!(
                "Content truncated from {} to {} bytes",
                text_content.len(),
                self.config.max_content_size
            );
            text_content.truncate(self.config.max_content_size);
        }

        Ok(FetchedContent {
            url: url.to_string(),
            content: text_content,
            title,
            metadata,
            content_type: "html".to_string(),
            fetched_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Parse HTTP fetch result
    fn parse_http_result(&self, url: &str, content: Vec<ToolContent>) -> Result<FetchedContent> {
        let mut text_content = String::new();

        for item in content {
            match item {
                ToolContent::Text { text } => {
                    text_content.push_str(&text);
                }
                ToolContent::Resource { text: Some(text), .. } => {
                    text_content.push_str(&text);
                }
                _ => {}
            }
        }

        // Truncate if too large
        if text_content.len() > self.config.max_content_size {
            warn!(
                "Content truncated from {} to {} bytes",
                text_content.len(),
                self.config.max_content_size
            );
            text_content.truncate(self.config.max_content_size);
        }

        Ok(FetchedContent {
            url: url.to_string(),
            content: text_content,
            title: None,
            metadata: serde_json::json!({}),
            content_type: "text".to_string(),
            fetched_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Fetch multiple URLs concurrently
    pub async fn fetch_urls(
        &self,
        mcp_client: &McpClient,
        urls: Vec<String>,
    ) -> Vec<Result<FetchedContent>> {
        let mut results = Vec::new();

        // Fetch concurrently in chunks to avoid overwhelming the system
        const MAX_CONCURRENT: usize = 5;

        for chunk in urls.chunks(MAX_CONCURRENT) {
            let futures: Vec<_> = chunk
                .iter()
                .map(|url| self.fetch_url(mcp_client, url))
                .collect();

            let chunk_results = futures::future::join_all(futures).await;
            results.extend(chunk_results);
        }

        results
    }
}

/// Extract clean text from HTML content
pub fn extract_text_from_html(html: &str) -> String {
    // Simple HTML text extraction (in production, use a proper HTML parser like scraper)
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    let text = re.replace_all(html, " ");

    // Clean up whitespace
    let text = text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetcher_config_default() {
        let config = FetcherConfig::default();
        assert_eq!(config.max_content_size, 1024 * 1024);
        assert_eq!(config.timeout, 30);
        assert!(config.extract_main_content);
    }

    #[test]
    fn test_extract_text_from_html() {
        let html = r#"
            <html>
                <body>
                    <h1>Title</h1>
                    <p>Paragraph 1</p>
                    <p>Paragraph 2</p>
                </body>
            </html>
        "#;

        let text = extract_text_from_html(html);
        assert!(text.contains("Title"));
        assert!(text.contains("Paragraph 1"));
        assert!(text.contains("Paragraph 2"));
        assert!(!text.contains("<html>"));
    }

    #[test]
    fn test_fetched_content_creation() {
        let content = FetchedContent {
            url: "https://example.com".to_string(),
            content: "Test content".to_string(),
            title: Some("Test".to_string()),
            metadata: serde_json::json!({"author": "Test Author"}),
            content_type: "html".to_string(),
            fetched_at: chrono::Utc::now().to_rfc3339(),
        };

        assert_eq!(content.url, "https://example.com");
        assert_eq!(content.content, "Test content");
        assert_eq!(content.title, Some("Test".to_string()));
    }
}
