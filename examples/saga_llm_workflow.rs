//! Saga pattern example with LLM integration
//!
//! This example demonstrates a research workflow saga where each step uses LLMs:
//! 1. Generate research plan using LLM
//! 2. Conduct research analysis using LLM
//! 3. Generate final report using LLM
//! 4. Publish results
//!
//! If any step fails, compensations execute in reverse order.

use std::sync::Arc;
use the_agency::{
    Agent, AgentBuilder, AgentConfig, AgentError, Result, SagaContext, SagaOrchestrator,
    SagaResult, SagaStep, WorkflowContext,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🤖 Saga Pattern with LLM Integration Example\n");
    println!("Scenario: AI Research Workflow\n");

    // Initialize LLM agent for research tasks with in-memory storage
    let mut config =
        AgentConfig::from_file("config.toml").unwrap_or_else(|_| AgentConfig::default());
    config.memory.persistent = false; // Disable persistent storage to avoid file system issues
    config.memory.database_url = Some("sqlite::memory:".to_string()); // Use in-memory database

    let agent = Arc::new(tokio::sync::Mutex::new(AgentBuilder::new()
        .with_config(config)
        .with_system_prompt(
            "You are an expert AI research assistant. Provide detailed, accurate, and well-structured responses.".to_string()
        )
        .build()
        .await?));

    // Test scenarios
    println!("=== Test 1: Successful Research Workflow ===");
    run_successful_research(Arc::clone(&agent)).await?;

    println!("\n=== Test 2: Failed Analysis (with compensation) ===");
    run_failed_analysis(Arc::clone(&agent)).await?;

    println!("\n=== Test 3: Failed Report Generation (with compensation) ===");
    run_failed_report(Arc::clone(&agent)).await?;

    Ok(())
}

/// Simulate a successful research workflow
async fn run_successful_research(agent: Arc<tokio::sync::Mutex<Agent>>) -> Result<()> {
    let orchestrator = create_research_saga(agent.clone(), false, false);
    let context = WorkflowContext::new(10);
    let saga_context = SagaContext::new("successful-research".to_string(), context);

    match orchestrator.execute(saga_context).await? {
        SagaResult::Completed(result) => {
            println!("✅ Research workflow completed successfully!");
            println!(
                "   Final result: {}",
                result
                    .get("status")
                    .unwrap_or(&serde_json::Value::String("unknown".to_string()))
            );
        }
        _ => println!("❌ Unexpected result"),
    }

    Ok(())
}

/// Simulate research analysis failure
async fn run_failed_analysis(agent: Arc<tokio::sync::Mutex<Agent>>) -> Result<()> {
    let orchestrator = create_research_saga(agent.clone(), true, false);
    let context = WorkflowContext::new(10);
    let saga_context = SagaContext::new("failed-analysis-research".to_string(), context);

    match orchestrator.execute(saga_context).await? {
        SagaResult::Compensated {
            error,
            compensated_steps,
        } => {
            println!("⚠️  Research workflow failed but successfully compensated");
            println!("   Error: {}", error);
            println!("   Compensated steps: {:?}", compensated_steps);
        }
        _ => println!("❌ Unexpected result"),
    }

    Ok(())
}

/// Simulate report generation failure
async fn run_failed_report(agent: Arc<tokio::sync::Mutex<Agent>>) -> Result<()> {
    let orchestrator = create_research_saga(agent.clone(), false, true);
    let context = WorkflowContext::new(10);
    let saga_context = SagaContext::new("failed-report-research".to_string(), context);

    match orchestrator.execute(saga_context).await? {
        SagaResult::Compensated {
            error,
            compensated_steps,
        } => {
            println!("⚠️  Research workflow failed but successfully compensated");
            println!("   Error: {}", error);
            println!("   Compensated steps: {:?}", compensated_steps);
        }
        _ => println!("❌ Unexpected result"),
    }

    Ok(())
}

/// Create a research workflow saga with LLM integration
fn create_research_saga(
    agent: Arc<tokio::sync::Mutex<Agent>>,
    fail_analysis: bool,
    fail_report: bool,
) -> SagaOrchestrator {
    // Step 1: Generate Research Plan using LLM
    let agent_clone1 = Arc::clone(&agent);
    let generate_plan = SagaStep::new(
        "generate_plan",
        "Generate Research Plan",
        move |_ctx| {
            println!("   🧠 Generating research plan using LLM...");

            let agent_clone = Arc::clone(&agent_clone1);
            let prompt = "Create a detailed research plan for investigating the impact of transformer architectures on natural language processing tasks. Include methodology, timeline, and success criteria. Format your response as a structured plan.";

            // Make actual LLM call
            let response = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut agent_guard = agent_clone.lock().await;
                    agent_guard.process(prompt).await
                })
            })?;

            println!("      → Generated comprehensive research plan");

            Ok(serde_json::json!({
                "plan_id": "PLAN-2024-001",
                "topic": "Transformer Architectures in NLP",
                "plan_content": response,
                "phases": ["Literature Review", "Model Design", "Implementation", "Evaluation", "Analysis"],
                "timeline_weeks": 8,
                "researcher": "AI Assistant"
            }))
        },
        |_ctx, result| {
            println!("   🔄 Compensating research plan generation...");
            if let Some(plan_id) = result.get("plan_id") {
                println!("      → Archived research plan: {}", plan_id);
            }
            Ok(())
        },
    )
    .with_retries(2);

    // Step 2: Conduct Research Analysis using LLM
    let agent_clone2 = Arc::clone(&agent);
    let conduct_analysis = SagaStep::new(
        "conduct_analysis",
        "Conduct Research Analysis",
        move |_ctx| {
            println!("   🔬 Conducting research analysis using LLM...");

            if fail_analysis {
                println!("      ❌ Analysis failed: Insufficient data quality");
                return Err(AgentError::Workflow(
                    "Research analysis failed: Data quality insufficient for reliable conclusions".to_string(),
                ));
            }

            let agent_clone = Arc::clone(&agent_clone2);
            let prompt = "Analyze the current state of transformer architectures in NLP. Review key papers, compare different approaches, and identify the most promising directions for future research. Provide specific findings and recommendations.";

            // Make actual LLM call
            let response = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut agent_guard = agent_clone.lock().await;
                    agent_guard.process(prompt).await
                })
            })?;

            println!("      → Completed comprehensive research analysis");

            Ok(serde_json::json!({
                "analysis_id": "ANALYSIS-2024-001",
                "analysis_content": response,
                "papers_reviewed": 50,
                "models_analyzed": 10,
                "key_findings": ["Transformers outperform RNNs by 15%", "Attention mechanisms are crucial", "Scaling laws apply"],
                "confidence_score": 0.87
            }))
        },
        |_ctx, result| {
            println!("   🔄 Compensating research analysis...");
            if let Some(analysis_id) = result.get("analysis_id") {
                println!("      → Deleted analysis results: {}", analysis_id);
            }
            Ok(())
        },
    )
    .with_retries(1);

    // Step 3: Generate Final Report using LLM
    let agent_clone3 = Arc::clone(&agent);
    let generate_report = SagaStep::new(
        "generate_report",
        "Generate Final Report",
        move |_ctx| {
            println!("   📄 Generating final report using LLM...");

            if fail_report {
                println!("      ❌ Report generation failed: Content validation error");
                return Err(AgentError::Network(
                    "Report generation failed: Content validation did not pass quality checks".to_string(),
                ));
            }

            let agent_clone = Arc::clone(&agent_clone3);
            let prompt = "Based on the research plan and analysis conducted, generate a comprehensive final report on transformer architectures in NLP. Include executive summary, methodology, key findings, discussion, and recommendations for future work.";

            // Make actual LLM call
            let response = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut agent_guard = agent_clone.lock().await;
                    agent_guard.process(prompt).await
                })
            })?;

            println!("      → Generated comprehensive final report");

            Ok(serde_json::json!({
                "report_id": "REPORT-2024-001",
                "title": "Impact of Transformer Architectures on NLP Tasks",
                "report_content": response,
                "sections": ["Executive Summary", "Methodology", "Results", "Discussion", "Recommendations"],
                "word_count": 12500,
                "citations": 67
            }))
        },
        |_ctx, result| {
            println!("   🔄 Compensating report generation...");
            if let Some(report_id) = result.get("report_id") {
                println!("      → Removed draft report: {}", report_id);
            }
            Ok(())
        },
    )
    .with_retries(2);

    // Step 4: Publish Results
    let publish_results = SagaStep::new(
        "publish_results",
        "Publish Results",
        |_ctx| {
            println!("   🌐 Publishing research results...");
            std::thread::sleep(std::time::Duration::from_millis(150));
            println!("      → Published to research repository and notified stakeholders");

            Ok(serde_json::json!({
                "publication_id": "PUB-2024-001",
                "platform": "Research Repository",
                "visibility": "public",
                "doi": "10.1234/research.2024.001",
                "stakeholders_notified": 15
            }))
        },
        |_ctx, result| {
            println!("   🔄 Compensating publication...");
            if let Some(publication_id) = result.get("publication_id") {
                println!("      → Unpublished and removed: {}", publication_id);
            }
            Ok(())
        },
    )
    .with_retries(1);

    // Build the orchestrator
    SagaOrchestrator::new()
        .add_step(generate_plan)
        .add_step(conduct_analysis)
        .add_step(generate_report)
        .add_step(publish_results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use the_agency::{AgentBuilder, AgentConfig};

    #[tokio::test]
    async fn test_successful_research_saga() {
        let config = AgentConfig::default();
        let agent = AgentBuilder::new()
            .with_config(config)
            .build()
            .await
            .unwrap();

        let result = run_successful_research(&agent).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_failed_analysis_compensation() {
        let config = AgentConfig::default();
        let agent = AgentBuilder::new()
            .with_config(config)
            .build()
            .await
            .unwrap();

        let result = run_failed_analysis(&agent).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_failed_report_compensation() {
        let config = AgentConfig::default();
        let agent = AgentBuilder::new()
            .with_config(config)
            .build()
            .await
            .unwrap();

        let result = run_failed_report(&agent).await;
        assert!(result.is_ok());
    }
}
