//! Tests for datetime and location tools

use chrono::{DateTime, Utc};
use the_agency::tools::{execute_datetime_info, execute_location_info, BuiltinTools};

#[tokio::test]
async fn test_datetime_info_tool() {
    let result = execute_datetime_info().await;

    assert!(!result.is_error);
    assert_eq!(result.content.len(), 1);

    if let Some(content) = result.content.first() {
        match content {
            the_agency::mcp::ToolContent::Text { text } => {
                // Check that the response contains expected datetime fields
                assert!(text.contains("local_time"));
                assert!(text.contains("utc_time"));
                assert!(text.contains("iso8601_local"));
                assert!(text.contains("iso8601_utc"));
                assert!(text.contains("timestamp"));
                assert!(text.contains("day_of_week"));
                assert!(text.contains("month"));
                assert!(text.contains("timezone"));
            }
            _ => panic!("Expected text content"),
        }
    }
}

#[tokio::test]
async fn test_location_info_tool() {
    let result = execute_location_info().await;

    assert!(!result.is_error);
    assert_eq!(result.content.len(), 1);

    if let Some(content) = result.content.first() {
        match content {
            the_agency::mcp::ToolContent::Text { text } => {
                // Check that the response contains expected location fields
                assert!(text.contains("method"));
                assert!(text.contains("available_methods"));
                assert!(text.contains("current_local_time"));

                // Should contain at least one of timezone or locale information
                assert!(text.contains("timezone") || text.contains("locale"));
            }
            _ => panic!("Expected text content"),
        }
    }
}

#[tokio::test]
async fn test_builtin_tools_includes_new_tools() {
    let tools = BuiltinTools::new();
    let tool_list = tools.list_tools();

    // Should contain all three built-in tools
    assert!(tool_list.contains(&"system_info".to_string()));
    assert!(tool_list.contains(&"datetime_info".to_string()));
    assert!(tool_list.contains(&"location_info".to_string()));

    // Should be exactly 3 tools
    assert_eq!(tool_list.len(), 3);
}

#[tokio::test]
async fn test_datetime_tool_execution_via_builtin_tools() {
    let tools = BuiltinTools::new();

    let result = tools.execute("datetime_info").await;
    assert!(result.is_some());

    let result = result.unwrap();
    assert!(!result.is_error);
    assert_eq!(result.content.len(), 1);
}

#[tokio::test]
async fn test_location_tool_execution_via_builtin_tools() {
    let tools = BuiltinTools::new();

    let result = tools.execute("location_info").await;
    assert!(result.is_some());

    let result = result.unwrap();
    assert!(!result.is_error);
    assert_eq!(result.content.len(), 1);
}

#[tokio::test]
async fn test_nonexistent_tool_returns_none() {
    let tools = BuiltinTools::new();

    let result = tools.execute("nonexistent_tool").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_datetime_tool_json_parsing() {
    let result = execute_datetime_info().await;

    if let Some(content) = result.content.first() {
        match content {
            the_agency::mcp::ToolContent::Text { text } => {
                // Extract JSON part from the response
                if let Some(json_start) = text.find("{") {
                    let json_str = &text[json_start..];
                    if let Some(json_end) = json_str.rfind("}") {
                        let json_str = &json_str[..=json_end];

                        // Parse the JSON to ensure it's valid
                        let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
                        assert!(
                            parsed.is_ok(),
                            "Failed to parse datetime JSON: {}",
                            json_str
                        );

                        let json = parsed.unwrap();

                        // Verify specific fields exist and have reasonable values
                        assert!(json["timestamp"].is_number());
                        assert!(json["local_time"].is_string());
                        assert!(json["utc_time"].is_string());
                        assert!(json["iso8601_local"].is_string());
                        assert!(json["iso8601_utc"].is_string());

                        // Verify ISO8601 format can be parsed
                        let iso8601_utc = json["iso8601_utc"].as_str().unwrap();
                        let parsed_datetime: Result<DateTime<Utc>, _> = iso8601_utc.parse();
                        assert!(
                            parsed_datetime.is_ok(),
                            "Failed to parse ISO8601 UTC datetime: {}",
                            iso8601_utc
                        );
                    }
                }
            }
            _ => panic!("Expected text content"),
        }
    }
}

#[tokio::test]
async fn test_location_tool_json_parsing() {
    let result = execute_location_info().await;

    if let Some(content) = result.content.first() {
        match content {
            the_agency::mcp::ToolContent::Text { text } => {
                // Extract JSON part from the response
                if let Some(json_start) = text.find("{") {
                    let json_str = &text[json_start..];
                    if let Some(json_end) = json_str.rfind("}") {
                        let json_str = &json_str[..=json_end];

                        // Parse the JSON to ensure it's valid
                        let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
                        assert!(
                            parsed.is_ok(),
                            "Failed to parse location JSON: {}",
                            json_str
                        );

                        let json = parsed.unwrap();

                        // Verify required fields exist
                        assert!(json["method"].is_string());
                        assert!(json["available_methods"].is_array());
                        assert!(json["current_local_time"].is_string());
                    }
                }
            }
            _ => panic!("Expected text content"),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;
    use the_agency::{Agent, AgentConfig};

    #[tokio::test]
    async fn test_agent_can_use_datetime_tool() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_agent.db");

        let mut config = AgentConfig::default();
        config.memory.database_url = Some(format!("sqlite://{}?mode=rwc", db_path.display()));

        let agent = Agent::new(config).await.expect("Failed to create agent");

        // Get agent stats to verify tools are available
        let stats = agent.stats().await;
        assert!(stats.builtin_tools_count >= 3); // Should have at least our 3 built-in tools
    }

    #[tokio::test]
    async fn test_datetime_and_location_tools_are_async() {
        // Test that both tools can be called concurrently
        let (datetime_result, location_result) =
            tokio::join!(execute_datetime_info(), execute_location_info());

        assert!(!datetime_result.is_error);
        assert!(!location_result.is_error);

        // Both should have content
        assert!(!datetime_result.content.is_empty());
        assert!(!location_result.content.is_empty());
    }
}
