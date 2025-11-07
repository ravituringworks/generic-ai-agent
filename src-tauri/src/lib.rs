use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeType {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub config_schema: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Output {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Node {
    pub id: String,
    pub node_type: String,
    pub position: Position,
    pub config: HashMap<String, String>,
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Connection {
    pub id: String,
    pub from_node: String,
    pub from_output: String,
    pub to_node: String,
    pub to_input: String,
}

fn load_workflows() -> Vec<Workflow> {
    let path = "workflows.json";
    if Path::new(path).exists() {
        let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
        serde_json::from_str(&data).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    }
}

fn save_workflows(workflows: &[Workflow]) {
    let data = serde_json::to_string_pretty(workflows).unwrap();
    fs::write("workflows.json", data).unwrap();
}

#[tauri::command]
fn get_node_types() -> Result<Vec<NodeType>, String> {
    let config_schema: serde_json::Value = serde_json::json!({
        "properties": {
            "prompt": {
                "type": "string",
                "default": "Enter your prompt"
            },
            "model": {
                "type": "string",
                "default": "gpt-3.5-turbo"
            }
        }
    });

    Ok(vec![
        NodeType {
            id: "start".to_string(),
            name: "Start".to_string(),
            category: "Flow Control".to_string(),
            description: "Initiates workflow execution".to_string(),
            inputs: vec![],
            outputs: vec![Output {
                name: "output".to_string(),
            }],
            config_schema: None,
        },
        NodeType {
            id: "end".to_string(),
            name: "End".to_string(),
            category: "Flow Control".to_string(),
            description: "Terminates workflow execution".to_string(),
            inputs: vec![Input {
                name: "input".to_string(),
            }],
            outputs: vec![],
            config_schema: None,
        },
        NodeType {
            id: "agent_task".to_string(),
            name: "Agent Task".to_string(),
            category: "AI Agents".to_string(),
            description: "Executes a task using an AI agent".to_string(),
            inputs: vec![Input {
                name: "input".to_string(),
            }],
            outputs: vec![Output {
                name: "output".to_string(),
            }],
            config_schema: Some(config_schema),
        },
        NodeType {
            id: "condition".to_string(),
            name: "Condition".to_string(),
            category: "Flow Control".to_string(),
            description: "Branches workflow based on a condition".to_string(),
            inputs: vec![Input {
                name: "input".to_string(),
            }],
            outputs: vec![
                Output {
                    name: "true".to_string(),
                },
                Output {
                    name: "false".to_string(),
                },
            ],
            config_schema: Some(serde_json::json!({
                "properties": {
                    "condition": {
                        "type": "string",
                        "default": "true"
                    }
                }
            })),
        },
        NodeType {
            id: "transform".to_string(),
            name: "Transform".to_string(),
            category: "Data Processing".to_string(),
            description: "Transforms data using a template".to_string(),
            inputs: vec![Input {
                name: "input".to_string(),
            }],
            outputs: vec![Output {
                name: "output".to_string(),
            }],
            config_schema: Some(serde_json::json!({
                "properties": {
                    "template": {
                        "type": "string",
                        "default": "{{ input }}"
                    }
                }
            })),
        },
    ])
}

#[tauri::command]
fn get_workflows() -> Result<Vec<Workflow>, String> {
    Ok(load_workflows())
}

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

#[tauri::command]
fn get_workflow(id: String) -> Result<Workflow, String> {
    let workflows = load_workflows();
    workflows
        .into_iter()
        .find(|w| w.id == id)
        .ok_or_else(|| "Workflow not found".to_string())
}

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

#[tauri::command]
fn delete_workflow(id: String) -> Result<(), String> {
    let mut workflows = load_workflows();
    workflows.retain(|w| w.id != id);
    save_workflows(&workflows);
    Ok(())
}

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
            "condition" => {
                let condition = node
                    .config
                    .get("condition")
                    .cloned()
                    .unwrap_or("true".to_string());
                execution_log.push(format!("Condition evaluated: {}", condition));
            }
            "transform" => {
                let template = node
                    .config
                    .get("template")
                    .cloned()
                    .unwrap_or("{{ input }}".to_string());
                execution_log.push(format!("Transform applied: {}", template));
            }
            _ => execution_log.push(format!("Unknown node type: {}", node.node_type)),
        }
    }

    execution_log.push("Workflow execution completed".to_string());

    Ok(serde_json::to_string(&execution_log).unwrap())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Starting Workflow Builder Tauri application...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
