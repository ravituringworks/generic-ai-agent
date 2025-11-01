//! Knowledge Management Helpers for Organization
//!
//! Utilities for querying and storing organizational knowledge

use super::{OrganizationRole, WorkspaceTask};
use crate::memory::MemoryEntry;
use crate::organization::coordinator::TaskResult;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Format past experiences for inclusion in agent prompt
pub fn format_past_experiences(memories: &[MemoryEntry]) -> String {
    if memories.is_empty() {
        return "No relevant past experiences found.".to_string();
    }

    let mut formatted = String::from("### Relevant Past Experiences:\n\n");

    for (idx, memory) in memories.iter().enumerate() {
        let quality = memory
            .metadata
            .get("quality_score")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.5);

        let reuse_count = memory
            .metadata
            .get("reuse_count")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        formatted.push_str(&format!(
            "{}. {} (Quality: {:.2}, Used {} times)\n\n",
            idx + 1,
            memory.content,
            quality,
            reuse_count
        ));
    }

    formatted
}

/// Create a knowledge entry from a completed task
pub fn create_knowledge_entry(
    role: &OrganizationRole,
    task: &WorkspaceTask,
    result: &TaskResult,
) -> MemoryEntry {
    // Calculate quality score based on success and error count
    let quality_score = if result.success {
        if result.errors.is_empty() {
            0.9 // High quality - success with no errors
        } else {
            0.7 // Good quality - success but with some issues
        }
    } else {
        0.3 // Low quality - failure
    };

    // Extract task type from title (simplified)
    let task_type = extract_task_type(&task.title);

    let content = format!(
        "Task: {}\n\
        Description: {}\n\
        Priority: {:?}\n\
        Outcome: {}\n\
        Key Insights: {}",
        task.title,
        task.description,
        task.priority,
        if result.success { "Success" } else { "Failed" },
        summarize_result(result)
    );

    MemoryEntry {
        id: Uuid::new_v4(),
        content,
        embedding: vec![], // Empty for now - would be populated by embedding service
        metadata: HashMap::from([
            ("role".to_string(), format!("{:?}", role)),
            ("task_type".to_string(), task_type),
            ("task_id".to_string(), task.id.clone()),
            ("priority".to_string(), format!("{:?}", task.priority)),
            ("quality_score".to_string(), quality_score.to_string()),
            ("reuse_count".to_string(), "0".to_string()),
            ("timestamp".to_string(), Utc::now().to_rfc3339()),
            ("success".to_string(), result.success.to_string()),
        ]),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Extract task type from title using simple heuristics
pub fn extract_task_type(title: &str) -> String {
    let lower = title.to_lowercase();

    if lower.contains("design") || lower.contains("architect") {
        "design".to_string()
    } else if lower.contains("implement") || lower.contains("build") || lower.contains("develop") {
        "implementation".to_string()
    } else if lower.contains("test") || lower.contains("qa") {
        "testing".to_string()
    } else if lower.contains("optimize") || lower.contains("improve") {
        "optimization".to_string()
    } else if lower.contains("debug") || lower.contains("fix") {
        "debugging".to_string()
    } else if lower.contains("refactor") {
        "refactoring".to_string()
    } else if lower.contains("research") || lower.contains("investigate") {
        "research".to_string()
    } else if lower.contains("document") {
        "documentation".to_string()
    } else {
        "general".to_string()
    }
}

/// Summarize task result for storage
fn summarize_result(result: &TaskResult) -> String {
    let mut summary = String::new();

    // Add output summary (truncate if too long)
    if result.output.len() > 200 {
        summary.push_str(&result.output[..200]);
        summary.push_str("...");
    } else {
        summary.push_str(&result.output);
    }

    // Add metrics if any
    if !result.metrics.is_empty() {
        summary.push_str("\nMetrics: ");
        for (key, value) in &result.metrics {
            summary.push_str(&format!("{}={:.2}, ", key, value));
        }
    }

    // Add error summary if any
    if !result.errors.is_empty() {
        summary.push_str(&format!("\nErrors encountered: {}", result.errors.len()));
    }

    summary
}

/// Build an enhanced prompt that includes past knowledge
pub fn build_knowledge_enhanced_prompt(
    role: &OrganizationRole,
    task: &WorkspaceTask,
    past_experiences: &[MemoryEntry],
) -> String {
    let role_context = role.system_prompt();
    let experiences = format_past_experiences(past_experiences);

    format!(
        "{}\n\n\
        {}\n\n\
        ### Current Task:\n\
        **Title:** {}\n\
        **Description:** {}\n\
        **Priority:** {:?}\n\n\
        Please leverage the past experiences above to inform your approach. \
        Consider what worked well and what challenges were encountered.\n\n\
        Provide a comprehensive solution.",
        role_context, experiences, task.title, task.description, task.priority
    )
}

/// Query similar tasks from memory by content matching (simple text-based)
/// This is a simplified version - a real implementation would use embeddings
pub fn find_similar_tasks(
    all_memories: &[MemoryEntry],
    task_description: &str,
    task_type: &str,
    limit: usize,
) -> Vec<MemoryEntry> {
    let desc_lower = task_description.to_lowercase();
    let keywords: Vec<&str> = desc_lower.split_whitespace().collect();

    let mut scored_memories: Vec<(f32, MemoryEntry)> = all_memories
        .iter()
        .filter(|m| {
            // Filter by task type if it matches
            m.metadata
                .get("task_type")
                .map_or(false, |t| t == task_type)
                || m.metadata
                    .get("task_type")
                    .map_or(false, |t| t == "general")
        })
        .map(|memory| {
            let content_lower = memory.content.to_lowercase();

            // Calculate simple keyword match score
            let keyword_matches = keywords
                .iter()
                .filter(|keyword| content_lower.contains(*keyword))
                .count();

            let score = (keyword_matches as f32 / keywords.len() as f32) * 100.0;

            (score, memory.clone())
        })
        .filter(|(score, _)| *score > 10.0) // Minimum relevance threshold
        .collect();

    // Sort by score descending
    scored_memories.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Take top N and return entries
    scored_memories
        .into_iter()
        .take(limit)
        .map(|(_, entry)| entry)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organization::{TaskPriority, TaskStatus};

    #[test]
    fn test_extract_task_type() {
        assert_eq!(extract_task_type("Design System Architecture"), "design");
        assert_eq!(extract_task_type("Implement Feature X"), "implementation");
        assert_eq!(extract_task_type("Optimize Performance"), "optimization");
        assert_eq!(extract_task_type("Debug Issue"), "debugging");
    }

    #[test]
    fn test_format_empty_experiences() {
        let result = format_past_experiences(&[]);
        assert!(result.contains("No relevant"));
    }

    #[test]
    fn test_create_knowledge_entry() {
        let role = OrganizationRole::SoftwareEngineerSimulation;
        let task = WorkspaceTask::new(
            "Test Task".to_string(),
            "Test description".to_string(),
            vec!["agent1".to_string()],
        );
        let result = TaskResult {
            success: true,
            output: "Task completed successfully".to_string(),
            metrics: HashMap::new(),
            errors: vec![],
        };

        let entry = create_knowledge_entry(&role, &task, &result);

        assert_eq!(entry.metadata.get("success").unwrap(), "true");
        assert!(entry.content.contains("Test Task"));
        assert!(
            entry
                .metadata
                .get("quality_score")
                .unwrap()
                .parse::<f32>()
                .unwrap()
                > 0.8
        );
    }
}
