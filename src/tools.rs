//! Tool management and execution

use crate::mcp::{ToolCall, ToolContent, ToolResult};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{Local, Utc};
use tokio::process::Command;

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

/// Built-in tool for date and time information
pub fn create_datetime_tool() -> ToolCall {
    ToolCall {
        id: Uuid::new_v4().to_string(),
        name: "datetime_info".to_string(),
        arguments: serde_json::json!({}),
    }
}

/// Execute a datetime info tool call
pub async fn execute_datetime_info() -> ToolResult {
    let now_local = Local::now();
    let now_utc = Utc::now();
    
    let info = serde_json::json!({
        "local_time": now_local.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        "utc_time": now_utc.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        "iso8601_local": now_local.to_rfc3339(),
        "iso8601_utc": now_utc.to_rfc3339(),
        "timestamp": now_utc.timestamp(),
        "day_of_week": now_local.format("%A").to_string(),
        "month": now_local.format("%B").to_string(),
        "timezone": now_local.format("%Z").to_string(),
        "timezone_offset": now_local.format("%z").to_string(),
    });

    ToolResult {
        id: Uuid::new_v4().to_string(),
        content: vec![ToolContent::Text {
            text: format!("Date/Time Info: {}", serde_json::to_string_pretty(&info).unwrap_or_else(|_| info.to_string())),
        }],
        is_error: false,
    }
}

/// Built-in tool for location information
pub fn create_location_tool() -> ToolCall {
    ToolCall {
        id: Uuid::new_v4().to_string(),
        name: "location_info".to_string(),
        arguments: serde_json::json!({}),
    }
}

/// Execute a location info tool call
pub async fn execute_location_info() -> ToolResult {
    let mut location_info = serde_json::json!({
        "method": "system_detection",
        "available_methods": ["timezone", "system_locale", "network_detection"]
    });

    // Get timezone information
    if let Ok(tz_output) = get_system_timezone().await {
        location_info["timezone"] = serde_json::Value::String(tz_output);
    }

    // Get system locale information
    if let Ok(locale_info) = get_system_locale().await {
        location_info["locale"] = locale_info;
    }

    // Get network-based location (basic IP geolocation)
    if let Ok(network_info) = get_network_location().await {
        location_info["network_location"] = network_info;
    }

    // Get current local time with timezone
    let now_local = Local::now();
    location_info["current_local_time"] = serde_json::Value::String(
        now_local.format("%Y-%m-%d %H:%M:%S %Z (%z)").to_string()
    );

    ToolResult {
        id: Uuid::new_v4().to_string(),
        content: vec![ToolContent::Text {
            text: format!("Location Info: {}", serde_json::to_string_pretty(&location_info).unwrap_or_else(|_| location_info.to_string())),
        }],
        is_error: false,
    }
}

/// Get system timezone information
async fn get_system_timezone() -> Result<String, Box<dyn std::error::Error>> {
    // Try different methods based on the OS
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("systemsetup")
            .args(["-gettimezone"])
            .output()
            .await
        {
            if output.status.success() {
                let tz_output = String::from_utf8_lossy(&output.stdout);
                if let Some(tz) = tz_output.split("Time Zone: ").nth(1) {
                    return Ok(tz.trim().to_string());
                }
            }
        }
        
        // Fallback: try reading timezone link
        if let Ok(output) = Command::new("readlink")
            .args(["/etc/localtime"])
            .output()
            .await
        {
            if output.status.success() {
                let link_output = String::from_utf8_lossy(&output.stdout);
                if let Some(tz) = link_output.split("/zoneinfo/").nth(1) {
                    return Ok(tz.trim().to_string());
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try reading /etc/timezone first
        if let Ok(content) = tokio::fs::read_to_string("/etc/timezone").await {
            return Ok(content.trim().to_string());
        }
        
        // Fallback: check /etc/localtime link
        if let Ok(output) = Command::new("readlink")
            .args(["/etc/localtime"])
            .output()
            .await
        {
            if output.status.success() {
                let link_output = String::from_utf8_lossy(&output.stdout);
                if let Some(tz) = link_output.split("/zoneinfo/").nth(1) {
                    return Ok(tz.trim().to_string());
                }
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("powershell")
            .args(["-Command", "(Get-TimeZone).Id"])
            .output()
            .await
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
    }
    
    // Final fallback: use chrono's local timezone
    let local_now = Local::now();
    Ok(format!("{} ({})", local_now.format("%Z"), local_now.format("%z")))
}

/// Get system locale information
async fn get_system_locale() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut locale_info = serde_json::json!({});
    
    // Try to get locale information from environment variables
    if let Ok(lang) = std::env::var("LANG") {
        locale_info["LANG"] = serde_json::Value::String(lang);
    }
    
    if let Ok(lc_all) = std::env::var("LC_ALL") {
        locale_info["LC_ALL"] = serde_json::Value::String(lc_all);
    }
    
    if let Ok(lc_time) = std::env::var("LC_TIME") {
        locale_info["LC_TIME"] = serde_json::Value::String(lc_time);
    }
    
    // Try platform-specific locale detection
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("defaults")
            .args(["read", "-g", "AppleLocale"])
            .output()
            .await
        {
            if output.status.success() {
                locale_info["macos_locale"] = serde_json::Value::String(
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                );
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("locale")
            .output()
            .await
        {
            if output.status.success() {
                let locale_output = String::from_utf8_lossy(&output.stdout);
                locale_info["system_locale"] = serde_json::Value::String(locale_output.trim().to_string());
            }
        }
    }
    
    Ok(locale_info)
}

/// Get basic network-based location information
async fn get_network_location() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Note: This is a basic implementation that just provides network interface information
    // In a production environment, you might want to integrate with a geolocation API
    
    let mut network_info = serde_json::json!({
        "note": "Basic network information available. For precise geolocation, consider integrating with a geolocation service.",
        "privacy_note": "Network-based location detection requires external API calls which may have privacy implications."
    });
    
    // Get network interface information (available without external APIs)
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("networksetup")
            .args(["-listallhardwareports"])
            .output()
            .await
        {
            if output.status.success() {
                network_info["network_interfaces_available"] = serde_json::Value::Bool(true);
            }
        }
    }
    
    // Get public IP (basic network detection)
    if let Ok(output) = Command::new("dig")
        .args(["+short", "myip.opendns.com", "@resolver1.opendns.com"])
        .output()
        .await
    {
        if output.status.success() {
            let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !ip.is_empty() && ip.contains(".") {
                network_info["public_ip"] = serde_json::Value::String(ip);
                network_info["note"] = serde_json::Value::String(
                    "Public IP detected. For location details, integrate with IP geolocation service.".to_string()
                );
            }
        }
    }
    
    Ok(network_info)
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
        
        // Add datetime info tool
        tools.insert(
            "datetime_info".to_string(),
            Box::new(|| {
                Box::new(Box::pin(execute_datetime_info()))
                    as Box<dyn std::future::Future<Output = ToolResult> + Send + Unpin>
            }) as Box<dyn Fn() -> Box<dyn std::future::Future<Output = ToolResult> + Send + Unpin> + Send + Sync>
        );
        
        // Add location info tool
        tools.insert(
            "location_info".to_string(),
            Box::new(|| {
                Box::new(Box::pin(execute_location_info()))
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