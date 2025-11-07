//! Workflow Builder - Desktop Application
//!
//! A native desktop application for creating and managing complex workflows,
//! built with Tauri for a native experience with web-based UI.

#[cfg(feature = "tauri")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "tauri")]
use serde_json;
#[cfg(feature = "tauri")]
use std::collections::HashMap;
#[cfg(feature = "tauri")]
use std::fs;
#[cfg(feature = "tauri")]
use std::path::Path;
#[cfg(feature = "tauri")]
use tauri;

#[cfg(feature = "tauri")]
#[derive(Serialize, Deserialize, Clone)]
struct NodeType {
    id: String,
    name: String,
    category: String,
    description: String,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    config_schema: Option<String>,
}

#[cfg(feature = "tauri")]
#[derive(Serialize, Deserialize, Clone)]
struct Input {
    name: String,
}

#[cfg(feature = "tauri")]
#[derive(Serialize, Deserialize, Clone)]
struct Output {
    name: String,
}

#[cfg(feature = "tauri")]
#[derive(Serialize, Deserialize, Clone)]
struct Workflow {
    id: String,
    name: String,
    description: String,
    nodes: Vec<Node>,
    connections: Vec<Connection>,
}

#[cfg(feature = "tauri")]
#[derive(Serialize, Deserialize, Clone)]
struct Node {
    id: String,
    node_type: String,
    position: Position,
    config: HashMap<String, String>,
    label: String,
}

#[cfg(feature = "tauri")]
#[derive(Serialize, Deserialize, Clone)]
struct Position {
    x: f64,
    y: f64,
}

#[cfg(feature = "tauri")]
#[derive(Serialize, Deserialize, Clone)]
struct Connection {
    id: String,
    from_node: String,
    from_output: String,
    to_node: String,
    to_input: String,
}

#[cfg(feature = "tauri")]
fn load_workflows() -> Vec<Workflow> {
    let path = "workflows.json";
    if Path::new(path).exists() {
        let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
        serde_json::from_str(&data).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    }
}

#[cfg(feature = "tauri")]
fn save_workflows(workflows: &[Workflow]) {
    let data = serde_json::to_string(workflows).unwrap();
    fs::write("workflows.json", data).unwrap();
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn get_node_types() -> Result<Vec<NodeType>, String> {
    Ok(vec![
        NodeType {
            id: "start".to_string(),
            name: "Start".to_string(),
            category: "Flow Control".to_string(),
            description: "Initiates workflow execution".to_string(),
            inputs: vec![],
            outputs: vec![Output { name: "output".to_string() }],
            config_schema: None,
        },
        NodeType {
            id: "end".to_string(),
            name: "End".to_string(),
            category: "Flow Control".to_string(),
            description: "Terminates workflow execution".to_string(),
            inputs: vec![Input { name: "input".to_string() }],
            outputs: vec![],
            config_schema: None,
        },
        NodeType {
            id: "agent_task".to_string(),
            name: "Agent Task".to_string(),
            category: "AI Agents".to_string(),
            description: "Executes a task using an AI agent".to_string(),
            inputs: vec![Input { name: "input".to_string() }],
            outputs: vec![Output { name: "output".to_string() }],
            config_schema: Some(r#"{"properties":{"prompt":{"type":"string","default":"Enter your prompt"},"model":{"type":"string","default":"gpt-3.5-turbo"}}}"#.to_string()),
        },
        NodeType {
            id: "model_config".to_string(),
            name: "Model Configuration".to_string(),
            category: "LLM".to_string(),
            description: "Configure LLM server connection (Ollama, OpenAI, etc.)".to_string(),
            inputs: vec![],
            outputs: vec![Output { name: "config".to_string() }],
            config_schema: Some(r#"{"type":"object","properties":{"server_url":{"type":"string","default":"http://localhost:11434","description":"Ollama or LLM server URL"},"model":{"type":"string","default":"llama2","description":"Model name to use"},"temperature":{"type":"number","minimum":0,"maximum":2,"default":0.7,"description":"Temperature for generation"},"max_tokens":{"type":"integer","minimum":1,"maximum":32000,"default":2048,"description":"Maximum tokens to generate"},"top_p":{"type":"number","minimum":0,"maximum":1,"default":0.9,"description":"Top-p sampling parameter"},"stream":{"type":"boolean","default":false,"description":"Enable streaming responses"}}}"#.to_string()),
        },
    ])
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn get_workflows() -> Result<Vec<Workflow>, String> {
    Ok(load_workflows())
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn create_workflow(name: String, description: String) -> Result<Workflow, String> {
    let id = format!(
        "workflow_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let workflow = Workflow {
        id: id.clone(),
        name,
        description,
        nodes: vec![],
        connections: vec![],
    };
    let mut workflows = load_workflows();
    workflows.push(workflow.clone());
    save_workflows(&workflows);
    Ok(workflow)
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn get_workflow(id: String) -> Result<Workflow, String> {
    let workflows = load_workflows();
    workflows
        .into_iter()
        .find(|w| w.id == id)
        .ok_or_else(|| "Workflow not found".to_string())
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn update_workflow(
    id: String,
    name: String,
    description: String,
    nodes: Vec<Node>,
    connections: Vec<Connection>,
) -> Result<(), String> {
    let mut workflows = load_workflows();
    if let Some(workflow) = workflows.iter_mut().find(|w| w.id == id) {
        workflow.name = name;
        workflow.description = description;
        workflow.nodes = nodes;
        workflow.connections = connections;
        save_workflows(&workflows);
        Ok(())
    } else {
        Err("Workflow not found".to_string())
    }
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn delete_workflow(id: String) -> Result<(), String> {
    let mut workflows = load_workflows();
    workflows.retain(|w| w.id != id);
    save_workflows(&workflows);
    Ok(())
}

#[cfg(feature = "tauri")]
#[tauri::command]
fn execute_workflow(id: String) -> Result<String, String> {
    let workflows = load_workflows();
    let workflow = workflows
        .into_iter()
        .find(|w| w.id == id)
        .ok_or_else(|| "Workflow not found".to_string())?;

    // Basic execution: simulate running the workflow
    let mut execution_log = vec!["Starting workflow execution".to_string()];

    for node in &workflow.nodes {
        match node.node_type.as_str() {
            "start" => execution_log.push("Start node executed".to_string()),
            "agent_task" => {
                let prompt = node
                    .config
                    .get("prompt")
                    .cloned()
                    .unwrap_or("No prompt".to_string());
                execution_log.push(format!("Agent task executed with prompt: {}", prompt));
            }
            "end" => execution_log.push("End node reached".to_string()),
            _ => execution_log.push(format!("Unknown node type: {}", node.node_type)),
        }
    }

    execution_log.push("Workflow execution completed".to_string());

    Ok(serde_json::to_string(&execution_log).unwrap())
}

#[cfg(feature = "tauri")]
fn main() {
    println!("Starting Tauri application...");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_node_types,
            get_workflows,
            create_workflow,
            get_workflow,
            update_workflow,
            delete_workflow,
            execute_workflow
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(feature = "tauri"))]
fn main() {
    println!("Tauri feature not enabled. Use --features tauri to build the desktop app.");
    println!("To build the desktop app, run: cargo build --bin workflow-builder --features tauri");
}
