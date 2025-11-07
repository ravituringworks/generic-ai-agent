//! Workflow UI - Visual Workflow Builder
//!
//! A web-based interface for creating and managing complex workflows,
//! inspired by draw.io and ComfyUI. Features drag-and-drop workflow
//! creation, visual node editing, and real-time execution monitoring.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
// use utoipa::{OpenApi, ToSchema};
// use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
struct AppState {
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
    nodes: Arc<RwLock<HashMap<String, NodeType>>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Workflow {
    id: String,
    name: String,
    description: String,
    nodes: Vec<WorkflowNode>,
    connections: Vec<Connection>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
struct WorkflowNode {
    id: String,
    node_type: String,
    position: Position,
    config: serde_json::Value,
    label: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Connection {
    id: String,
    from_node: String,
    from_output: String,
    to_node: String,
    to_input: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct NodeType {
    id: String,
    name: String,
    category: String,
    description: String,
    inputs: Vec<NodeInput>,
    outputs: Vec<NodeOutput>,
    config_schema: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
struct NodeInput {
    name: String,
    r#type: String,
    required: bool,
    description: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct NodeOutput {
    name: String,
    r#type: String,
    description: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct CreateWorkflowRequest {
    name: String,
    description: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct UpdateWorkflowRequest {
    name: Option<String>,
    description: Option<String>,
    nodes: Option<Vec<WorkflowNode>>,
    connections: Option<Vec<Connection>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Workflow UI Server");
    println!("📍 Access the UI at: http://localhost:3000");
    println!("📚 API docs at: http://localhost:3000/docs");

    // Initialize application state
    let state = AppState {
        workflows: Arc::new(RwLock::new(HashMap::new())),
        nodes: Arc::new(RwLock::new(HashMap::new())),
    };

    // Initialize default node types
    initialize_node_types(&state).await;

    // Create router
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/workflows", get(list_workflows).post(create_workflow))
        .route(
            "/workflows/{id}",
            get(get_workflow)
                .put(update_workflow)
                .delete(delete_workflow),
        )
        .route("/workflows/{id}/execute", post(execute_workflow))
        .route("/nodes", get(list_node_types))
        .route("/api/health", get(health_check))
        // .merge(SwaggerUi::new("/docs"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("✅ Server started on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn initialize_node_types(state: &AppState) {
    let mut nodes = state.nodes.write().await;

    // LLM Nodes
    nodes.insert(
        "llm_generate".to_string(),
        NodeType {
            id: "llm_generate".to_string(),
            name: "LLM Generate".to_string(),
            category: "LLM".to_string(),
            description: "Generate text using a language model".to_string(),
            inputs: vec![
                NodeInput {
                    name: "prompt".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    description: "The prompt to send to the LLM".to_string(),
                },
                NodeInput {
                    name: "model".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                    description: "LLM model to use (optional)".to_string(),
                },
            ],
            outputs: vec![NodeOutput {
                name: "response".to_string(),
                r#type: "string".to_string(),
                description: "Generated text response".to_string(),
            }],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "temperature": {
                        "type": "number",
                        "minimum": 0,
                        "maximum": 2,
                        "default": 0.7
                    },
                    "max_tokens": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 4096,
                        "default": 1000
                    }
                }
            }),
        },
    );

    // Model Configuration Node
    nodes.insert(
        "model_config".to_string(),
        NodeType {
            id: "model_config".to_string(),
            name: "Model Configuration".to_string(),
            category: "LLM".to_string(),
            description: "Configure LLM server connection (Ollama, OpenAI, etc.)".to_string(),
            inputs: vec![],
            outputs: vec![NodeOutput {
                name: "config".to_string(),
                r#type: "object".to_string(),
                description: "LLM configuration".to_string(),
            }],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "server_url": {
                        "type": "string",
                        "default": "http://localhost:11434",
                        "description": "Ollama or LLM server URL"
                    },
                    "model": {
                        "type": "string",
                        "default": "llama2",
                        "description": "Model name to use"
                    },
                    "temperature": {
                        "type": "number",
                        "minimum": 0,
                        "maximum": 2,
                        "default": 0.7,
                        "description": "Temperature for generation"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 32000,
                        "default": 2048,
                        "description": "Maximum tokens to generate"
                    },
                    "top_p": {
                        "type": "number",
                        "minimum": 0,
                        "maximum": 1,
                        "default": 0.9,
                        "description": "Top-p sampling parameter"
                    },
                    "stream": {
                        "type": "boolean",
                        "default": false,
                        "description": "Enable streaming responses"
                    }
                }
            }),
        },
    );

    // Data Processing Nodes
    nodes.insert(
        "text_splitter".to_string(),
        NodeType {
            id: "text_splitter".to_string(),
            name: "Text Splitter".to_string(),
            category: "Data Processing".to_string(),
            description: "Split text into chunks".to_string(),
            inputs: vec![NodeInput {
                name: "text".to_string(),
                r#type: "string".to_string(),
                required: true,
                description: "Text to split".to_string(),
            }],
            outputs: vec![NodeOutput {
                name: "chunks".to_string(),
                r#type: "array".to_string(),
                description: "Array of text chunks".to_string(),
            }],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chunk_size": {
                        "type": "integer",
                        "minimum": 100,
                        "maximum": 10000,
                        "default": 1000
                    },
                    "overlap": {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 500,
                        "default": 200
                    }
                }
            }),
        },
    );

    // Control Flow Nodes
    nodes.insert(
        "conditional".to_string(),
        NodeType {
            id: "conditional".to_string(),
            name: "Conditional".to_string(),
            category: "Control Flow".to_string(),
            description: "Route execution based on conditions".to_string(),
            inputs: vec![
                NodeInput {
                    name: "condition".to_string(),
                    r#type: "boolean".to_string(),
                    required: true,
                    description: "Boolean condition to evaluate".to_string(),
                },
                NodeInput {
                    name: "true_value".to_string(),
                    r#type: "any".to_string(),
                    required: false,
                    description: "Value to pass when condition is true".to_string(),
                },
                NodeInput {
                    name: "false_value".to_string(),
                    r#type: "any".to_string(),
                    required: false,
                    description: "Value to pass when condition is false".to_string(),
                },
            ],
            outputs: vec![NodeOutput {
                name: "output".to_string(),
                r#type: "any".to_string(),
                description: "Selected output value".to_string(),
            }],
            config_schema: serde_json::json!({}),
        },
    );

    // I/O Nodes
    nodes.insert(
        "file_input".to_string(),
        NodeType {
            id: "file_input".to_string(),
            name: "File Input".to_string(),
            category: "I/O".to_string(),
            description: "Read data from a file".to_string(),
            inputs: vec![],
            outputs: vec![NodeOutput {
                name: "content".to_string(),
                r#type: "string".to_string(),
                description: "File content".to_string(),
            }],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the input file"
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["utf8", "binary"],
                        "default": "utf8"
                    }
                },
                "required": ["file_path"]
            }),
        },
    );

    nodes.insert(
        "file_output".to_string(),
        NodeType {
            id: "file_output".to_string(),
            name: "File Output".to_string(),
            category: "I/O".to_string(),
            description: "Write data to a file".to_string(),
            inputs: vec![NodeInput {
                name: "content".to_string(),
                r#type: "string".to_string(),
                required: true,
                description: "Content to write".to_string(),
            }],
            outputs: vec![],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the output file"
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["utf8", "binary"],
                        "default": "utf8"
                    },
                    "append": {
                        "type": "boolean",
                        "default": false,
                        "description": "Append to file instead of overwriting"
                    }
                },
                "required": ["file_path"]
            }),
        },
    );
}

// API Handlers

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

async fn list_workflows(State(state): State<AppState>) -> Json<Vec<Workflow>> {
    let workflows = state.workflows.read().await;
    let workflow_list: Vec<Workflow> = workflows.values().cloned().collect();
    Json(workflow_list)
}

async fn create_workflow(
    State(state): State<AppState>,
    Json(request): Json<CreateWorkflowRequest>,
) -> Result<Json<Workflow>, StatusCode> {
    let now = chrono::Utc::now();
    let workflow = Workflow {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        description: request.description,
        nodes: vec![],
        connections: vec![],
        created_at: now,
        updated_at: now,
    };

    let mut workflows = state.workflows.write().await;
    workflows.insert(workflow.id.clone(), workflow.clone());

    Ok(Json(workflow))
}

async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Workflow>, StatusCode> {
    let workflows = state.workflows.read().await;
    match workflows.get(&id) {
        Some(workflow) => Ok(Json(workflow.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateWorkflowRequest>,
) -> Result<Json<Workflow>, StatusCode> {
    let mut workflows = state.workflows.write().await;
    if let Some(workflow) = workflows.get_mut(&id) {
        if let Some(name) = request.name {
            workflow.name = name;
        }
        if let Some(description) = request.description {
            workflow.description = description;
        }
        if let Some(nodes) = request.nodes {
            workflow.nodes = nodes;
        }
        if let Some(connections) = request.connections {
            workflow.connections = connections;
        }
        workflow.updated_at = chrono::Utc::now();
        Ok(Json(workflow.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut workflows = state.workflows.write().await;
    if workflows.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn execute_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let workflows = state.workflows.read().await;
    match workflows.get(&id) {
        Some(workflow) => {
            // Call agency daemon to execute the workflow
            let daemon_url = std::env::var("AGENCY_DAEMON_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string());

            let execute_url = format!("{}/api/v1/workflows/execute", daemon_url);

            // Prepare request payload
            let request_payload = serde_json::json!({
                "id": workflow.id,
                "name": workflow.name,
                "description": workflow.description,
                "nodes": workflow.nodes,
                "connections": workflow.connections
            });

            // Make HTTP request to agency daemon
            let client = reqwest::Client::new();
            match client.post(&execute_url)
                .json(&request_payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(result) => Ok(Json(result)),
                            Err(e) => {
                                eprintln!("Failed to parse execution response: {}", e);
                                // Return mock result as fallback
                                Ok(Json(serde_json::json!({
                                    "status": "error",
                                    "output": format!("Failed to parse response: {}", e)
                                })))
                            }
                        }
                    } else {
                        // Daemon returned error
                        let status_code = response.status().as_u16();
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        Ok(Json(serde_json::json!({
                            "status": "error",
                            "output": format!("Agency daemon error ({}): {}", status_code, error_text)
                        })))
                    }
                }
                Err(e) => {
                    // Daemon is not available - return informative error
                    eprintln!("Failed to connect to agency daemon at {}: {}", execute_url, e);
                    Ok(Json(serde_json::json!({
                        "status": "error",
                        "output": format!("Agency daemon not available at {}. Please start the daemon with: cargo run --bin agency-daemon --release\n\nError: {}", daemon_url, e)
                    })))
                }
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn list_node_types(State(state): State<AppState>) -> Json<Vec<NodeType>> {
    let nodes = state.nodes.read().await;
    let node_list: Vec<NodeType> = nodes.values().cloned().collect();
    Json(node_list)
}

// TODO: Add OpenAPI documentation
// struct ApiDoc;
