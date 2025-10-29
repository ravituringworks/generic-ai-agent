//! Model Context Protocol (MCP) client implementation

use crate::config::{McpConfig, McpServerConfig};
use crate::error::{McpError, Result};
use async_trait::async_trait;
use jsonrpc_core::{Id, MethodCall, Params, Response, Version};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Tool definition from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: String,
    pub content: Vec<ToolContent>,
    pub is_error: bool,
}

/// Content in a tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { uri: String, text: Option<String> },
}

/// MCP server connection types
#[derive(Debug, Clone)]
pub enum McpTransport {
    Http { url: String, auth_token: Option<String> },
    WebSocket { url: String, auth_token: Option<String> },
    Stdio { command: Vec<String>, env: HashMap<String, String> },
}

/// MCP server connection
#[async_trait]
pub trait McpConnection: Send + Sync {
    /// Send a JSON-RPC request and get response
    async fn call(&self, method: &str, params: Value) -> Result<Value>;
    
    /// Check if the connection is healthy
    async fn health_check(&self) -> Result<bool>;
    
    /// Close the connection
    async fn close(&mut self) -> Result<()>;
}

/// HTTP-based MCP connection
pub struct HttpMcpConnection {
    client: reqwest::Client,
    url: String,
    auth_token: Option<String>,
}

impl HttpMcpConnection {
    pub fn new(url: String, auth_token: Option<String>, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            url,
            auth_token,
        }
    }
}

#[async_trait]
impl McpConnection for HttpMcpConnection {
    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let id = Id::Str(Uuid::new_v4().to_string());
        
        let request = MethodCall {
            jsonrpc: Some(Version::V2),
            method: method.to_string(),
            params: Params::Map(
                params.as_object().unwrap_or(&Map::new()).clone()
            ),
            id: id.clone(),
        };

        let mut http_request = self.client
            .post(&self.url)
            .json(&request);

        if let Some(token) = &self.auth_token {
            http_request = http_request.bearer_auth(token);
        }

        let response = http_request
            .send()
            .await
            .map_err(|e| McpError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(McpError::ProtocolError(
                format!("HTTP error: {}", response.status())
            ).into());
        }

        let json_response: Response = response
            .json()
            .await
            .map_err(|e| McpError::ProtocolError(e.to_string()))?;

        match json_response {
            Response::Single(output) => {
                if output.id() != &id {
                    return Err(McpError::ProtocolError("ID mismatch".to_string()).into());
                }
                
                match output {
                    jsonrpc_core::Output::Success(success) => Ok(success.result),
                    jsonrpc_core::Output::Failure(failure) => {
                        Err(McpError::ProtocolError(
                            format!("JSON-RPC error: {:?} - {}", failure.error.code, failure.error.message)
                        ).into())
                    }
                }
            }
            Response::Batch(_) => {
                Err(McpError::ProtocolError("Unexpected batch response".to_string()).into())
            }
        }
    }

    async fn health_check(&self) -> Result<bool> {
        match self.call("ping", Value::Object(Map::new())).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn close(&mut self) -> Result<()> {
        // HTTP connections are stateless, nothing to close
        Ok(())
    }
}

/// MCP client for managing multiple server connections
pub struct McpClient {
    servers: HashMap<String, Box<dyn McpConnection>>,
    tools_cache: HashMap<String, Vec<McpTool>>,
    config: McpConfig,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(config: McpConfig) -> Self {
        Self {
            servers: HashMap::new(),
            tools_cache: HashMap::new(),
            config,
        }
    }

    /// Add a server connection
    pub async fn add_server(&mut self, name: String, server_config: McpServerConfig) -> Result<()> {
        if !server_config.enabled {
            debug!("Server {} is disabled, skipping", name);
            return Ok(());
        }

        info!("Adding MCP server: {}", name);

        let connection: Box<dyn McpConnection> = match server_config.transport.as_str() {
            "http" => {
                let url = server_config.url
                    .ok_or_else(|| McpError::ConnectionFailed("HTTP URL required".to_string()))?;
                
                let timeout = Duration::from_secs(
                    server_config.timeout.unwrap_or(self.config.default_timeout)
                );

                Box::new(HttpMcpConnection::new(url, server_config.auth_token, timeout))
            }
            "websocket" => {
                return Err(McpError::ConnectionFailed("WebSocket not implemented yet".to_string()).into());
            }
            "stdio" => {
                return Err(McpError::ConnectionFailed("Stdio not implemented yet".to_string()).into());
            }
            _ => {
                return Err(McpError::ConnectionFailed(
                    format!("Unsupported transport: {}", server_config.transport)
                ).into());
            }
        };

        // Test connection
        if !connection.health_check().await.unwrap_or(false) {
            warn!("Health check failed for server: {}", name);
        }

        // Initialize server and get tools
        self.initialize_server(&name, &*connection).await?;

        self.servers.insert(name, connection);
        info!("Successfully added MCP server");

        Ok(())
    }

    /// Initialize a server connection and cache its tools
    async fn initialize_server(&mut self, name: &str, connection: &dyn McpConnection) -> Result<()> {
        debug!("Initializing server: {}", name);

        // Initialize the MCP session
        let init_params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": {
                    "listChanged": true
                },
                "sampling": {}
            },
            "clientInfo": {
                "name": "the-agency",
                "version": "0.1.0"
            }
        });

        connection.call("initialize", init_params).await?;

        // Get available tools
        let tools_response = connection.call("tools/list", Value::Object(Map::new())).await?;
        
        if let Some(tools_array) = tools_response.get("tools").and_then(|t| t.as_array()) {
            let mut tools = Vec::new();
            
            for tool_value in tools_array {
                if let Ok(tool) = serde_json::from_value::<McpTool>(tool_value.clone()) {
                    tools.push(tool);
                } else {
                    warn!("Failed to parse tool definition: {:?}", tool_value);
                }
            }

            info!("Server {} provides {} tools", name, tools.len());
            self.tools_cache.insert(name.to_string(), tools);
        } else {
            warn!("No tools found for server: {}", name);
            self.tools_cache.insert(name.to_string(), Vec::new());
        }

        Ok(())
    }

    /// Get all available tools across all servers
    pub fn list_tools(&self) -> Vec<(String, &McpTool)> {
        let mut all_tools = Vec::new();
        
        for (server_name, tools) in &self.tools_cache {
            for tool in tools {
                all_tools.push((server_name.clone(), tool));
            }
        }
        
        all_tools
    }

    /// Find which server provides a specific tool
    pub fn find_tool_server(&self, tool_name: &str) -> Option<(&str, &McpTool)> {
        for (server_name, tools) in &self.tools_cache {
            for tool in tools {
                if tool.name == tool_name {
                    return Some((server_name, tool));
                }
            }
        }
        None
    }

    /// Call a tool by name
    pub async fn call_tool(&self, tool_call: ToolCall) -> Result<ToolResult> {
        debug!("Calling tool: {}", tool_call.name);

        let (server_name, _tool) = self.find_tool_server(&tool_call.name)
            .ok_or_else(|| McpError::ToolNotFound(tool_call.name.clone()))?;

        let connection = self.servers.get(server_name)
            .ok_or_else(|| McpError::ConnectionFailed(
                format!("Server {} not found", server_name)
            ))?;

        let call_params = serde_json::json!({
            "name": tool_call.name,
            "arguments": tool_call.arguments
        });

        let timeout_duration = Duration::from_secs(self.config.default_timeout);
        
        let result = timeout(
            timeout_duration,
            connection.call("tools/call", call_params)
        )
        .await
        .map_err(|_| McpError::Timeout(format!("Tool call timed out: {}", tool_call.name)))?;

        match result {
            Ok(response) => {
                // Parse the MCP tool response
                let content = if let Some(content_array) = response.get("content").and_then(|c| c.as_array()) {
                    let mut parsed_content = Vec::new();
                    
                    for content_value in content_array {
                        if let Ok(content_item) = serde_json::from_value::<ToolContent>(content_value.clone()) {
                            parsed_content.push(content_item);
                        } else {
                            // Fallback to text content
                            if let Some(text) = content_value.get("text").and_then(|t| t.as_str()) {
                                parsed_content.push(ToolContent::Text { text: text.to_string() });
                            }
                        }
                    }
                    parsed_content
                } else {
                    // Fallback: treat entire response as text
                    vec![ToolContent::Text { text: response.to_string() }]
                };

                let is_error = response.get("isError")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(false);

                Ok(ToolResult {
                    id: tool_call.id,
                    content,
                    is_error,
                })
            }
            Err(e) => {
                error!("Tool call failed: {}", e);
                Ok(ToolResult {
                    id: tool_call.id,
                    content: vec![ToolContent::Text { 
                        text: format!("Tool call failed: {}", e) 
                    }],
                    is_error: true,
                })
            }
        }
    }

    /// Call multiple tools concurrently
    pub async fn call_tools(&self, tool_calls: Vec<ToolCall>) -> Vec<ToolResult> {
        let max_concurrent = self.config.max_concurrent_calls;
        let mut results = Vec::new();
        
        // Process in chunks to respect concurrency limits
        for chunk in tool_calls.chunks(max_concurrent) {
            let futures: Vec<_> = chunk.iter().map(|call| self.call_tool(call.clone())).collect();
            let chunk_results = futures::future::join_all(futures).await;
            
            for result in chunk_results {
                match result {
                    Ok(tool_result) => results.push(tool_result),
                    Err(e) => {
                        error!("Tool call error: {}", e);
                        results.push(ToolResult {
                            id: Uuid::new_v4().to_string(),
                            content: vec![ToolContent::Text { 
                                text: format!("Error: {}", e) 
                            }],
                            is_error: true,
                        });
                    }
                }
            }
        }
        
        results
    }

    /// Remove a server connection
    pub async fn remove_server(&mut self, name: &str) -> Result<()> {
        if let Some(mut connection) = self.servers.remove(name) {
            connection.close().await?;
            self.tools_cache.remove(name);
            info!("Removed MCP server: {}", name);
        }
        Ok(())
    }

    /// Get server statistics
    pub fn stats(&self) -> McpStats {
        let total_tools = self.tools_cache.values()
            .map(|tools| tools.len())
            .sum();

        McpStats {
            connected_servers: self.servers.len(),
            total_tools,
            servers: self.tools_cache.iter()
                .map(|(name, tools)| (name.clone(), tools.len()))
                .collect(),
        }
    }
}

/// MCP client statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct McpStats {
    pub connected_servers: usize,
    pub total_tools: usize,
    pub servers: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_content_serialization() {
        let text_content = ToolContent::Text {
            text: "Hello, world!".to_string(),
        };

        let json = serde_json::to_value(&text_content).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello, world!");

        let deserialized: ToolContent = serde_json::from_value(json).unwrap();
        if let ToolContent::Text { text } = deserialized {
            assert_eq!(text, "Hello, world!");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall {
            id: "test-id".to_string(),
            name: "test-tool".to_string(),
            arguments: json!({"param": "value"}),
        };

        assert_eq!(call.name, "test-tool");
        assert_eq!(call.arguments["param"], "value");
    }

    #[test]
    fn test_mcp_client_creation() {
        let config = McpConfig::default();
        let client = McpClient::new(config);
        
        assert_eq!(client.servers.len(), 0);
        assert_eq!(client.tools_cache.len(), 0);
    }

    #[tokio::test]
    async fn test_http_connection_creation() {
        let connection = HttpMcpConnection::new(
            "http://example.com".to_string(),
            Some("token".to_string()),
            Duration::from_secs(30),
        );

        assert_eq!(connection.url, "http://example.com");
        assert_eq!(connection.auth_token, Some("token".to_string()));
    }

    // Mock tests would require a test MCP server, which is beyond the scope
    // of this basic implementation. In practice, you'd use wiremock or similar
    // to create mock HTTP endpoints for testing.
}