use anyhow::Result;
use the_agency::organization::coordinator::AgentCoordinator;
use the_agency::{
    AgentConfig, CollaborativeWorkspace, Organization, OrganizationAgent, OrganizationRole,
    TaskPriority, WorkspaceTask,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to see debug output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("Testing coordinator with 2 agents...\n");

    // Create organization
    let mut org = Organization::new("Test Org".to_string());

    // Add just 2 agents
    let alice = OrganizationAgent::new(
        "EMP001".to_string(),
        OrganizationRole::ResearchEngineerScaling,
    );
    let alice_id = alice.id.clone();
    org.add_agent(alice);

    let bob = OrganizationAgent::new(
        "EMP002".to_string(),
        OrganizationRole::ResearchEngineerAutonomy,
    );
    let bob_id = bob.id.clone();
    org.add_agent(bob);

    // Create workspace
    let workspace =
        CollaborativeWorkspace::new("Test Workspace".to_string(), "Simple test".to_string());
    let ws_id = workspace.id.clone();
    org.create_workspace(workspace);
    org.assign_agent_to_workspace(&alice_id, &ws_id)?;

    println!("✅ Organization created with 2 agents\n");

    // Create coordinator
    let coordinator = AgentCoordinator::new(org);

    println!("🚀 Spawning agents...\n");

    // Spawn agents
    let mut config = AgentConfig::default();
    config.llm.ollama_url = "http://127.0.0.1:11434".to_string();
    config.memory.database_url = Some(":memory:".to_string());
    config.memory.persistent = false;
    config.agent.use_memory = false;
    config.agent.use_tools = false;
    config.agent.max_thinking_steps = 1;
    config.workflow.enable_suspend_resume = false;

    coordinator
        .spawn_agent(alice_id.clone(), config.clone())
        .await?;
    println!("  ✓ Spawned EMP001");

    coordinator.spawn_agent(bob_id.clone(), config).await?;
    println!("  ✓ Spawned EMP002\n");

    println!("🎯 Creating and executing task...\n");

    // Create a simple task
    let task = WorkspaceTask::new(
        "Say hello".to_string(),
        "Just say hello in 5 words".to_string(),
        vec![alice_id.clone()],
    )
    .with_priority(TaskPriority::High);

    // Execute task
    let results = coordinator
        .coordinate_workspace_project(&ws_id, vec![task])
        .await?;

    println!("\n✅ SUCCESS! Results:");
    for result in &results {
        println!("  Response: {}", result.output);
    }

    Ok(())
}
