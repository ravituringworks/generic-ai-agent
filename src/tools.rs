//! Tool management and execution

use crate::mcp::{ToolCall, ToolContent, ToolResult};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Built-in tool for system information
pub fn create_system_info_tool() -> ToolCall {
    ToolCall {
        id: Uuid::new_v4().to_string(),
        name: "system_info".to_string(),
        arguments: serde_json::json!({}),
    }
}

/// Execute a system info tool call
pub async fn execute_system_info() -> ToolResult {
    let info = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "family": std::env::consts::FAMILY,
    });

    ToolResult {
        id: Uuid::new_v4().to_string(),
        content: vec![ToolContent::Text {
            text: format!("System Info: {}", info),
        }],
        is_error: false,
    }
}

/// Built-in tool registry
pub struct BuiltinTools {
    tools: HashMap<String, Box<dyn Fn() -> Box<dyn std::future::Future<Output = ToolResult> + Send + Unpin> + Send + Sync>>,
}

impl BuiltinTools {
    pub fn new() -> Self {
        let mut tools = HashMap::new();
        
        // Add system info tool
        tools.insert(
            "system_info".to_string(),
            Box::new(|| {
                Box::new(Box::pin(execute_system_info()))
                    as Box<dyn std::future::Future<Output = ToolResult> + Send + Unpin>
            }) as Box<dyn Fn() -> Box<dyn std::future::Future<Output = ToolResult> + Send + Unpin> + Send + Sync>
        );

        Self { tools }
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub async fn execute(&self, tool_name: &str) -> Option<ToolResult> {
        if let Some(executor) = self.tools.get(tool_name) {
            Some(executor().await)
        } else {
            None
        }
    }
}

impl Default for BuiltinTools {
    fn default() -> Self {
        Self::new()
    }
}