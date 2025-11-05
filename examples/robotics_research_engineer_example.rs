//! Robotics Research Engineer Role Example
//!
//! This example demonstrates the Research Engineer specializing in Robotics Design and Development role.
//! It shows the role's capabilities, system prompt, and collaboration patterns.

use the_agency::organization::OrganizationRole;
use tracing::{info, Level};

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Robotics Research Engineer Role Example");

    // Create the role instance
    let role = OrganizationRole::ResearchEngineerRobotics;

    info!("Role: {:?}", role);
    info!("Category: {:?}", role.category());
    info!("Capabilities: {:?}", role.capabilities());
    info!("Typical collaborators: {:?}", role.typical_collaborators());

    // Display the system prompt
    info!("System Prompt Preview:");
    let prompt = role.system_prompt();
    info!("{}", &prompt[..500]); // Show first 500 chars
    info!("... (truncated for display)");

    // Show key responsibilities from the prompt
    info!("Key Responsibilities:");
    if prompt.contains("Design and analyze mechanical and electrical systems") {
        info!("✓ Mechanical and electrical design");
        info!("✓ System integration and prototyping");
        info!("✓ Failure analysis and testing");
        info!("✓ CAD modeling (SOLIDWORKS, NX)");
        info!("✓ Tolerance analysis and GD&T");
        info!("✓ Manufacturing support");
    }

    // Demonstrate collaboration capabilities
    info!("Collaboration Patterns:");
    let collaborators = role.typical_collaborators();
    for collaborator in collaborators {
        info!("- Collaborates with: {:?}", collaborator);
    }

    // Show learning behaviors
    info!("Learning Behaviors:");
    if prompt.contains("LEARNING BEHAVIORS") {
        info!("- Documents design iterations and failure analysis results");
        info!("- Shares prototyping lessons learned with team");
        info!("- Records manufacturing feedback for design improvements");
        info!("- Builds repository of design patterns and solutions");
    }

    info!("Robotics Research Engineer role example completed");
}