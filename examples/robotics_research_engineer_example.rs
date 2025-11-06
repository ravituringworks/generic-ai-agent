//! Robotics Scientist Agent Example
//!
//! This example demonstrates the Robotics Scientist Agent with full functionality
//! for advanced robotics research, RL experimentation, and scientific validation.

mod robotics_scientist_agent;

use anyhow::Result;
use robotics_scientist_agent::RoboticsScientistAgent;
use the_agency::AgentConfig;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🔬 Starting Robotics Scientist Agent Example");

    // Load configuration (use default if config file not found)
    let mut config = AgentConfig::from_file("config.toml").unwrap_or_else(|_| {
        println!("⚠️  Config file not found, using defaults");
        AgentConfig::default()
    });

    // Use in-memory database to avoid file system issues
    config.memory.database_url = Some("sqlite::memory:".to_string());
    config.memory.persistent = false;

    // Create the robotics scientist agent
    let mut agent = RoboticsScientistAgent::new(config).await?;

    info!("Agent created successfully with robotics research capabilities");

    // Demonstrate core capabilities
    println!("\n🧪 Robotics Scientist Agent Capabilities:");
    println!("{}", "=".repeat(80));
    println!("• RL algorithm development and optimization (PPO, SAC, DDPG, TD3)");
    println!("• Robotic manipulation research (reach, grasp, pick-and-place)");
    println!("• Simulation environment design and validation");
    println!("• Hyperparameter optimization and experimental design");
    println!("• Performance benchmarking and statistical analysis");
    println!("• Sim-to-real transfer and domain adaptation");
    println!("• Control theory integration with learning methods");
    println!("• Research methodology and reproducibility validation");

    // Example task: Design RL experiment for manipulation
    println!("\n📋 Example Task: Design RL Experiment for Robotic Manipulation");
    println!("{}", "-".repeat(80));

    let _experiment_design = agent
        .design_rl_experiment(
            "Autonomous pick-and-place using Franka Panda with combined RL and traditional controls",
            "Franka Panda robot with PyBullet simulation and ROS integration",
            "Success rate >85%, sample efficiency, sim-to-real transfer accuracy, convergence stability",
        )
        .await?;

    // Example task: Analyze RL algorithm performance
    println!("\n📋 Example Task: Analyze PPO vs SAC for Robotic Tasks");
    println!("{}", "-".repeat(80));

    let _algorithm_analysis = agent
        .analyze_rl_algorithm(
            "Proximal Policy Optimization (PPO) vs Soft Actor-Critic (SAC)",
            "Robotic reach, grasp, and pick-and-place with continuous action spaces",
            "Sample efficiency differences, hyperparameter sensitivity, convergence properties",
        )
        .await?;

    // Example task: Develop simulation environment
    println!("\n📋 Example Task: Develop Research Simulation Environment");
    println!("{}", "-".repeat(80));

    let _simulation_development = agent
        .develop_simulation_environment(
            "Realistic manipulation tasks with physics-based object interactions and obstacle avoidance",
            "Accurate contact dynamics, friction modeling, sensor noise, real-time performance",
            "Visual fidelity, physics accuracy, ground truth data collection, ROS integration",
        )
        .await?;

    info!("✅ Robotics Scientist Agent example completed successfully!");
    info!("🔬 Agent is ready for advanced robotics research, experimentation, and validation");

    Ok(())
}