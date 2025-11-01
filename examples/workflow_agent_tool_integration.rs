//! Comprehensive example demonstrating workflow integration with agents and tools
//!
//! NOTE: This example is currently disabled as it requires refactoring to match
//! the current API. The Agent struct is not a trait, and some tool APIs have changed.
//!
//! TODO: Refactor this example to use the current Agent API

#![allow(dead_code, unused_imports, unused_variables)]

// Placeholder main function
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("This example is currently disabled pending API refactoring.");
    println!("Please see other workflow examples for current usage patterns.");
    Ok(())
}

// Original example code commented out pending refactoring
/*
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use the_agency::{
    agent::Agent,
    config::AgentConfig,
    error::Result,
    llm::{user_message, LlmClient},
    mcp::ToolResult,
    workflow::{
        MapperFn, StepSchema, WorkflowBuilder, WorkflowContext, WorkflowDecision, WorkflowStep,
    },
};

// ... rest of original code would go here ...
*/
