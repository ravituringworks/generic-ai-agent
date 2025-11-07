//! REST API server for the Agent platform
//!
//! Provides HTTP endpoints for:
//! - Agent operations (process, query)
//! - Workflow management (create, execute, suspend, resume)
//! - A2A communication
//! - System monitoring

use crate::agent::{Agent, AgentBuilder};
use crate::config::AgentConfig;
use crate::error::{AgentError, Result};
use crate::ui_workflow_storage::UIWorkflowStorage;
use crate::workflow::{WorkflowContext, WorkflowEngine, WorkflowSnapshot};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// The agent instance
    pub agent: Arc<RwLock<Agent>>,
    /// Workflow engine
    pub workflow_engine: Arc<WorkflowEngine>,
    /// Visual workflows storage (SQLite)
    pub ui_workflow_storage: Arc<UIWorkflowStorage>,
    /// Node types for visual workflow builder (in-memory cache)
    pub ui_node_types: Arc<RwLock<HashMap<String, UINodeType>>>,
}

impl AppState {
    pub async fn new(config: AgentConfig) -> Result<Self> {
        let agent = AgentBuilder::new().with_config(config.clone()).build().await?;
        let workflow_engine = Arc::new(WorkflowEngine::default());

        // Initialize UI workflow storage with same database as agent
        let database_url = config
            .memory
            .database_url
            .as_ref()
            .ok_or_else(|| AgentError::Config("No database URL configured".to_string()))?;
        let ui_workflow_storage = Arc::new(UIWorkflowStorage::new(database_url).await?);

        Ok(Self {
            agent: Arc::new(RwLock::new(agent)),
            workflow_engine,
            ui_workflow_storage,
            ui_node_types: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

/// Request to process a message
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProcessRequest {
    /// The message to process
    #[schema(example = "What is Rust?")]
    pub message: String,
    /// Maximum number of steps to execute (optional)
    #[serde(default)]
    #[schema(example = 10)]
    pub max_steps: Option<usize>,
}

/// Response from processing a message
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProcessResponse {
    /// The agent's response
    #[schema(example = "Rust is a systems programming language...")]
    pub response: String,
    /// Number of steps executed
    #[schema(example = 1)]
    pub steps_executed: usize,
    /// Whether processing completed
    #[schema(example = true)]
    pub completed: bool,
}

/// Request to create a workflow
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateWorkflowRequest {
    /// Unique workflow identifier
    #[schema(example = "workflow-123")]
    pub workflow_id: String,
    /// Initial message to start workflow
    #[schema(example = "Process this data")]
    pub initial_message: String,
    /// Maximum steps to execute
    #[schema(example = 20)]
    pub max_steps: usize,
}

/// Response for workflow creation
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateWorkflowResponse {
    /// The created workflow ID
    pub workflow_id: String,
    /// Status of creation
    #[schema(example = "created")]
    pub status: String,
}

/// Request to suspend a workflow
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SuspendWorkflowRequest {
    /// Reason for suspension
    #[schema(example = "Manual suspension")]
    pub reason: String,
}

/// Request to resume a workflow
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ResumeWorkflowRequest {
    /// Snapshot ID to resume from
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub snapshot_id: String,
}

/// Health check response
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Health status
    #[schema(example = "ok")]
    pub status: String,
    /// API version
    #[schema(example = "0.2.0")]
    pub version: String,
}

/// Error response
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message
    #[schema(example = "Invalid request")]
    pub error: String,
    /// Additional error details
    pub details: Option<String>,
}

/// Visual workflow node definition
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct VisualWorkflowNode {
    pub id: String,
    pub node_type: String,
    pub position: Position,
    pub config: serde_json::Value,
    pub label: Option<String>,
}

/// Node position
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Visual workflow connection
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct VisualWorkflowConnection {
    pub id: String,
    pub from_node: String,
    pub from_output: String,
    pub to_node: String,
    pub to_input: String,
    pub label: Option<String>,
    pub description: Option<String>,
}

/// Request to execute a visual workflow
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExecuteVisualWorkflowRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub nodes: Vec<VisualWorkflowNode>,
    pub connections: Vec<VisualWorkflowConnection>,
}

/// Response from visual workflow execution
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExecuteVisualWorkflowResponse {
    pub status: String,
    pub output: serde_json::Value,
    pub execution_time_ms: u64,
    pub steps_executed: usize,
}

// ============= Visual Workflow UI Types =============

/// Visual workflow stored in UI
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UIWorkflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<UIWorkflowNode>,
    pub connections: Vec<UIConnection>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Node in visual workflow UI
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UIWorkflowNode {
    pub id: String,
    pub node_type: String,
    pub position: UIPosition,
    pub config: serde_json::Value,
    pub label: String,
}

/// Position of node in UI canvas
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UIPosition {
    pub x: f64,
    pub y: f64,
}

/// Connection between nodes in UI
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UIConnection {
    pub id: String,
    pub from_node: String,
    pub from_output: String,
    pub to_node: String,
    pub to_input: String,
}

/// Node type definition for UI palette
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UINodeType {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub inputs: Vec<UINodeInput>,
    pub outputs: Vec<UINodeOutput>,
    pub config_schema: serde_json::Value,
}

/// Node input definition
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UINodeInput {
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub description: String,
}

/// Node output definition
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UINodeOutput {
    pub name: String,
    pub r#type: String,
    pub description: String,
}

/// Request to create a new workflow
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUIWorkflowRequest {
    pub name: String,
    pub description: String,
}

/// Request to update an existing workflow
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUIWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub nodes: Option<Vec<UIWorkflowNode>>,
    pub connections: Option<Vec<UIConnection>>,
}

/// Custom error type for API responses
#[derive(Debug)]
pub struct ApiError(AgentError);

impl From<AgentError> for ApiError {
    fn from(err: AgentError) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            AgentError::Config(msg) => (StatusCode::BAD_REQUEST, msg),
            AgentError::Network(msg) => (StatusCode::BAD_GATEWAY, msg),
            AgentError::Workflow(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };

        let body = Json(ErrorResponse {
            error: error_message.clone(),
            details: Some(error_message),
        });

        (status, body).into_response()
    }
}

/// Convert Result<T> to Result<T, ApiError>
type ApiResult<T> = std::result::Result<T, ApiError>;

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        process_handler,
        create_workflow_handler,
        suspend_workflow_handler,
        resume_workflow_handler,
        list_snapshots_handler,
        get_snapshot_handler,
        delete_snapshot_handler,
        execute_visual_workflow_handler,
    ),
    components(
        schemas(
            ProcessRequest,
            ProcessResponse,
            CreateWorkflowRequest,
            CreateWorkflowResponse,
            SuspendWorkflowRequest,
            ResumeWorkflowRequest,
            HealthResponse,
            ErrorResponse,
            ExecuteVisualWorkflowRequest,
            ExecuteVisualWorkflowResponse,
            VisualWorkflowNode,
            VisualWorkflowConnection,
            Position,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "agent", description = "Agent operations"),
        (name = "workflows", description = "Workflow management")
    ),
    info(
        title = "The Agency API",
        version = "0.2.0",
        description = "REST API for The Agency AI Agent Platform\n\nProvides endpoints for:\n- Agent message processing\n- Workflow orchestration and management\n- Long-running workflow suspend/resume\n- Saga pattern support",
        contact(
            name = "Turing Works",
            email = "rboddipalli@turingworks.com",
            url = "https://turingworks.com"
        )
    )
)]
struct ApiDoc;

/// Create the API router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_handler))
        // Agent endpoints
        .route("/api/v1/agent/process", post(process_handler))
        // Workflow endpoints
        .route("/api/v1/workflows", post(create_workflow_handler))
        .route(
            "/api/v1/workflows/{id}/suspend",
            post(suspend_workflow_handler),
        )
        .route("/api/v1/workflows/resume", post(resume_workflow_handler))
        .route("/api/v1/workflows/snapshots", get(list_snapshots_handler))
        .route(
            "/api/v1/workflows/snapshots/{id}",
            get(get_snapshot_handler),
        )
        .route(
            "/api/v1/workflows/snapshots/{id}",
            axum::routing::delete(delete_snapshot_handler),
        )
        // Visual workflow execution endpoint
        .route("/api/v1/workflows/execute", post(execute_visual_workflow_handler))
        // Workflow UI endpoints
        .route("/workflow-ui", get(serve_workflow_ui))
        .route("/workflow-ui/workflows", get(list_ui_workflows).post(create_ui_workflow))
        .route(
            "/workflow-ui/workflows/{id}",
            get(get_ui_workflow)
                .put(update_ui_workflow)
                .delete(delete_ui_workflow),
        )
        .route("/workflow-ui/workflows/{id}/execute", post(execute_ui_workflow))
        .route("/workflow-ui/nodes", get(list_ui_node_types))
        .route("/workflow-ui/api/health", get(workflow_ui_health))
        // OpenAPI spec endpoint
        .route("/api-docs/openapi.json", get(openapi_spec_handler))
        .with_state(state)
        // Add middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

/// OpenAPI specification endpoint
async fn openapi_spec_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: crate::VERSION.to_string(),
    })
}

/// Process a message through the agent
#[utoipa::path(
    post,
    path = "/api/v1/agent/process",
    tag = "agent",
    request_body = ProcessRequest,
    responses(
        (status = 200, description = "Message processed successfully", body = ProcessResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn process_handler(
    State(state): State<AppState>,
    Json(request): Json<ProcessRequest>,
) -> ApiResult<Json<ProcessResponse>> {
    info!("Processing message: {}", request.message);

    let mut agent = state.agent.write().await;
    let response = agent
        .process(&request.message)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(ProcessResponse {
        response,
        steps_executed: 1, // TODO: Track actual steps
        completed: true,
    }))
}

/// Create a new workflow
#[utoipa::path(
    post,
    path = "/api/v1/workflows",
    tag = "workflows",
    request_body = CreateWorkflowRequest,
    responses(
        (status = 200, description = "Workflow created successfully", body = CreateWorkflowResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    )
)]
async fn create_workflow_handler(
    State(_state): State<AppState>,
    Json(request): Json<CreateWorkflowRequest>,
) -> ApiResult<Json<CreateWorkflowResponse>> {
    info!("Creating workflow: {}", request.workflow_id);

    let _context = WorkflowContext::new(request.max_steps);

    // TODO: Store workflow context and associate with workflow_id

    Ok(Json(CreateWorkflowResponse {
        workflow_id: request.workflow_id,
        status: "created".to_string(),
    }))
}

/// Suspend a running workflow
#[utoipa::path(
    post,
    path = "/api/v1/workflows/{id}/suspend",
    tag = "workflows",
    params(
        ("id" = String, Path, description = "Workflow ID")
    ),
    request_body = SuspendWorkflowRequest,
    responses(
        (status = 200, description = "Workflow suspended")
    )
)]
async fn suspend_workflow_handler(
    State(_state): State<AppState>,
    Path(_workflow_id): Path<String>,
    Json(_request): Json<SuspendWorkflowRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Implement workflow suspension
    Ok(Json(serde_json::json!({
        "status": "suspended",
        "message": "Workflow suspension not yet implemented"
    })))
}

/// Resume a suspended workflow
#[utoipa::path(
    post,
    path = "/api/v1/workflows/resume",
    tag = "workflows",
    request_body = ResumeWorkflowRequest,
    responses(
        (status = 200, description = "Workflow resumed successfully"),
        (status = 400, description = "Invalid snapshot ID", body = ErrorResponse),
        (status = 404, description = "Snapshot not found", body = ErrorResponse)
    )
)]
async fn resume_workflow_handler(
    State(state): State<AppState>,
    Json(request): Json<ResumeWorkflowRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Resuming workflow from snapshot: {}", request.snapshot_id);

    let snapshot_id = Uuid::parse_str(&request.snapshot_id)
        .map_err(|e| ApiError::from(AgentError::Workflow(format!("Invalid snapshot ID: {}", e))))?;

    let result = state
        .workflow_engine
        .resume_from_snapshot(snapshot_id)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(serde_json::json!({
        "response": result.response,
        "completed": result.completed,
        "steps_executed": result.steps_executed
    })))
}

/// List all workflow snapshots
#[utoipa::path(
    get,
    path = "/api/v1/workflows/snapshots",
    tag = "workflows",
    responses(
        (status = 200, description = "List of workflow snapshots"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn list_snapshots_handler(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<WorkflowSnapshot>>> {
    info!("Listing workflow snapshots");

    let snapshots = state
        .workflow_engine
        .list_snapshots(None)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(snapshots))
}

/// Get a specific workflow snapshot
#[utoipa::path(
    get,
    path = "/api/v1/workflows/snapshots/{id}",
    tag = "workflows",
    params(
        ("id" = String, Path, description = "Snapshot ID (UUID)")
    ),
    responses(
        (status = 200, description = "Snapshot details"),
        (status = 404, description = "Snapshot not found", body = ErrorResponse)
    )
)]
async fn get_snapshot_handler(
    State(_state): State<AppState>,
    Path(snapshot_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting snapshot: {}", snapshot_id);

    // TODO: Implement get snapshot
    Ok(Json(serde_json::json!({
        "snapshot_id": snapshot_id,
        "message": "Get snapshot not yet fully implemented"
    })))
}

/// Delete a workflow snapshot
#[utoipa::path(
    delete,
    path = "/api/v1/workflows/snapshots/{id}",
    tag = "workflows",
    params(
        ("id" = String, Path, description = "Snapshot ID (UUID) to delete")
    ),
    responses(
        (status = 200, description = "Snapshot deleted successfully"),
        (status = 404, description = "Snapshot not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn delete_snapshot_handler(
    State(state): State<AppState>,
    Path(snapshot_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Deleting snapshot: {}", snapshot_id);

    let uuid = Uuid::parse_str(&snapshot_id)
        .map_err(|e| ApiError::from(AgentError::Workflow(format!("Invalid snapshot ID: {}", e))))?;

    let deleted = state
        .workflow_engine
        .delete_snapshot(uuid)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(serde_json::json!({
        "deleted": deleted
    })))
}

/// Execute a visual workflow
#[utoipa::path(
    post,
    path = "/api/v1/workflows/execute",
    tag = "workflows",
    request_body = ExecuteVisualWorkflowRequest,
    responses(
        (status = 200, description = "Workflow executed successfully", body = ExecuteVisualWorkflowResponse),
        (status = 400, description = "Invalid workflow", body = ErrorResponse),
        (status = 500, description = "Execution failed", body = ErrorResponse)
    )
)]
async fn execute_visual_workflow_handler(
    State(state): State<AppState>,
    Json(request): Json<ExecuteVisualWorkflowRequest>,
) -> ApiResult<Json<ExecuteVisualWorkflowResponse>> {
    use std::collections::HashMap;
    use std::time::Instant;

    info!("Executing visual workflow: {}", request.name);
    let start_time = Instant::now();

    // Build execution graph
    let mut node_map: HashMap<String, &VisualWorkflowNode> = HashMap::new();
    for node in &request.nodes {
        node_map.insert(node.id.clone(), node);
    }

    // Store node outputs
    let mut node_outputs: HashMap<String, serde_json::Value> = HashMap::new();
    let mut steps_executed = 0;

    // Find nodes with no incoming connections (start nodes)
    let mut has_incoming: HashMap<String, bool> = HashMap::new();
    for conn in &request.connections {
        has_incoming.insert(conn.to_node.clone(), true);
    }

    let start_nodes: Vec<&VisualWorkflowNode> = request.nodes.iter()
        .filter(|n| !has_incoming.contains_key(&n.id))
        .collect();

    if start_nodes.is_empty() {
        return Err(AgentError::Config("No start nodes found in workflow".to_string()).into());
    }

    // Execute nodes in topological order (simplified - execute start nodes then connected nodes)
    let mut executed: HashMap<String, bool> = HashMap::new();
    let mut to_execute: Vec<String> = start_nodes.iter().map(|n| n.id.clone()).collect();

    info!("Starting workflow execution with {} nodes, {} connections", request.nodes.len(), request.connections.len());
    info!("Start nodes: {:?}", start_nodes.iter().map(|n| &n.id).collect::<Vec<_>>());

    while let Some(node_id) = to_execute.pop() {
        if executed.contains_key(&node_id) {
            continue;
        }

        let node = node_map.get(&node_id).ok_or_else(|| {
            AgentError::Config(format!("Node not found: {}", node_id))
        })?;

        // Gather inputs from connected nodes
        let mut inputs: HashMap<String, serde_json::Value> = HashMap::new();
        for conn in request.connections.iter().filter(|c| c.to_node == node_id) {
            if let Some(output) = node_outputs.get(&conn.from_node) {
                inputs.insert(conn.to_input.clone(), output.clone());
            }
        }

        info!("Executing node {} (type: {}), inputs keys: {:?}", node_id, node.node_type, inputs.keys().collect::<Vec<_>>());

        // Execute node based on type
        let output = execute_node(&state, node, &inputs).await?;
        node_outputs.insert(node_id.clone(), output.clone());
        executed.insert(node_id.clone(), true);
        steps_executed += 1;

        info!("Node {} completed, output: {:?}", node_id, output);

        // Find next nodes to execute
        for conn in request.connections.iter().filter(|c| c.from_node == node_id) {
            info!("Found outgoing connection from {} to {} (input: {})", node_id, conn.to_node, conn.to_input);
            if !executed.contains_key(&conn.to_node) {
                // Check if all inputs are ready
                let required_inputs: Vec<_> = request.connections.iter()
                    .filter(|c| c.to_node == conn.to_node)
                    .collect();
                let all_inputs_ready = required_inputs.iter()
                    .all(|c| executed.contains_key(&c.from_node));

                info!("Node {} has {} required inputs, {} ready. All ready: {}", conn.to_node, required_inputs.len(), required_inputs.iter().filter(|c| executed.contains_key(&c.from_node)).count(), all_inputs_ready);

                if all_inputs_ready {
                    info!("Adding node {} to execution queue", conn.to_node);
                    to_execute.push(conn.to_node.clone());
                }
            }
        }
    }

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    // Collect final outputs (from nodes with no outgoing connections)
    let mut final_output = serde_json::json!({});
    if let Some(obj) = final_output.as_object_mut() {
        for (node_id, output) in &node_outputs {
            let has_outgoing = request.connections.iter()
                .any(|c| c.from_node == *node_id);
            if !has_outgoing {
                obj.insert(node_id.clone(), output.clone());
            }
        }
    }

    Ok(Json(ExecuteVisualWorkflowResponse {
        status: "completed".to_string(),
        output: final_output,
        execution_time_ms,
        steps_executed,
    }))
}

/// Execute a single node
async fn execute_node(
    state: &AppState,
    node: &VisualWorkflowNode,
    inputs: &HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value> {
    match node.node_type.as_str() {
        "llm_generate" => {
            // Extract prompt from inputs or config
            // Handle case where input is an object from file_input ({"content": "...", "error": "..."})
            let prompt = if let Some(prompt_value) = inputs.get("prompt") {
                if let Some(s) = prompt_value.as_str() {
                    s.to_string()
                } else if let Some(obj) = prompt_value.as_object() {
                    // Extract "content" field from file_input output
                    obj.get("content")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "".to_string())
                } else {
                    "".to_string()
                }
            } else if let Some(config_prompt) = node.config.get("prompt").and_then(|v| v.as_str()) {
                config_prompt.to_string()
            } else {
                "".to_string()
            };

            if prompt.is_empty() {
                return Ok(serde_json::json!({
                    "response": "No prompt provided"
                }));
            }

            // Use the agent to process the message
            let mut agent = state.agent.write().await;
            let response = agent.process(&prompt).await?;

            Ok(serde_json::json!({
                "response": response
            }))
        },
        "file_input" => {
            let file_path = node.config.get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if file_path.is_empty() {
                return Ok(serde_json::json!({
                    "content": "",
                    "error": "No file path specified"
                }));
            }

            match std::fs::read_to_string(file_path) {
                Ok(content) => Ok(serde_json::json!({
                    "content": content
                })),
                Err(e) => Ok(serde_json::json!({
                    "content": "",
                    "error": e.to_string()
                }))
            }
        },
        "file_output" => {
            let file_path = node.config.get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Extract content - could be a string directly or from an object (e.g., {"response": "..."})
            let content = if let Some(content_value) = inputs.get("content") {
                if let Some(s) = content_value.as_str() {
                    s.to_string()
                } else if let Some(obj) = content_value.as_object() {
                    // Try to extract "response" field from object
                    obj.get("response")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| content_value.to_string())
                } else {
                    content_value.to_string()
                }
            } else {
                "".to_string()
            };

            if file_path.is_empty() {
                return Ok(serde_json::json!({
                    "success": false,
                    "error": "No file path specified"
                }));
            }

            match std::fs::write(file_path, &content) {
                Ok(_) => Ok(serde_json::json!({
                    "success": true,
                    "path": file_path
                })),
                Err(e) => Ok(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                }))
            }
        },
        "text_splitter" => {
            let text = inputs.get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let chunk_size = node.config.get("chunk_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(1000) as usize;

            let chunks: Vec<String> = text.chars()
                .collect::<Vec<char>>()
                .chunks(chunk_size)
                .map(|chunk| chunk.iter().collect())
                .collect();

            Ok(serde_json::json!({
                "chunks": chunks
            }))
        },
        "conditional" => {
            let condition = inputs.get("condition")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let true_value = inputs.get("true_value");
            let false_value = inputs.get("false_value");

            let output = if condition {
                true_value.cloned().unwrap_or(serde_json::json!(true))
            } else {
                false_value.cloned().unwrap_or(serde_json::json!(false))
            };

            Ok(serde_json::json!({
                "output": output
            }))
        },
        "model_config" => {
            // Return the configuration
            Ok(node.config.clone())
        },
        _ => {
            // Unknown node type - return empty output
            Ok(serde_json::json!({
                "warning": format!("Unknown node type: {}", node.node_type)
            }))
        }
    }
}

// ============= Visual Workflow UI Handlers =============

/// Serve the workflow UI HTML
async fn serve_workflow_ui() -> impl IntoResponse {
    // Read the HTML file at runtime so we don't need to recompile for UI changes
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            error!("Failed to read static/index.html: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load UI: {}", e),
            )
                .into_response()
        }
    }
}

/// List all visual workflows
async fn list_ui_workflows(State(state): State<AppState>) -> ApiResult<Json<Vec<UIWorkflow>>> {
    let stored_workflows = state.ui_workflow_storage.list().await?;

    let workflows: Vec<UIWorkflow> = stored_workflows
        .into_iter()
        .map(|stored| UIWorkflow {
            id: stored.id,
            name: stored.name,
            description: stored.description,
            nodes: serde_json::from_str(&stored.nodes_json).unwrap_or_default(),
            connections: serde_json::from_str(&stored.connections_json).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&stored.created_at)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            updated_at: chrono::DateTime::parse_from_rfc3339(&stored.updated_at)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
        })
        .collect();

    Ok(Json(workflows))
}

/// Create a new visual workflow
async fn create_ui_workflow(
    State(state): State<AppState>,
    Json(request): Json<CreateUIWorkflowRequest>,
) -> ApiResult<Json<UIWorkflow>> {
    use crate::ui_workflow_storage::StoredWorkflow;

    let now = chrono::Utc::now();
    let workflow = UIWorkflow {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        description: request.description,
        nodes: vec![],
        connections: vec![],
        created_at: now,
        updated_at: now,
    };

    let stored = StoredWorkflow {
        id: workflow.id.clone(),
        name: workflow.name.clone(),
        description: workflow.description.clone(),
        nodes_json: serde_json::to_string(&workflow.nodes).unwrap_or_else(|_| "[]".to_string()),
        connections_json: serde_json::to_string(&workflow.connections).unwrap_or_else(|_| "[]".to_string()),
        created_at: workflow.created_at.to_rfc3339(),
        updated_at: workflow.updated_at.to_rfc3339(),
    };

    state.ui_workflow_storage.create(&stored).await?;

    Ok(Json(workflow))
}

/// Get a specific visual workflow
async fn get_ui_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<UIWorkflow>> {
    let stored = state.ui_workflow_storage.get(&id).await?;

    match stored {
        Some(stored) => {
            let workflow = UIWorkflow {
                id: stored.id,
                name: stored.name,
                description: stored.description,
                nodes: serde_json::from_str(&stored.nodes_json).unwrap_or_default(),
                connections: serde_json::from_str(&stored.connections_json).unwrap_or_default(),
                created_at: chrono::DateTime::parse_from_rfc3339(&stored.created_at)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::parse_from_rfc3339(&stored.updated_at)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now),
            };
            Ok(Json(workflow))
        }
        None => Err(AgentError::NotFound("workflow not found".to_string()).into()),
    }
}

/// Update a visual workflow
async fn update_ui_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateUIWorkflowRequest>,
) -> ApiResult<Json<UIWorkflow>> {
    use crate::ui_workflow_storage::StoredWorkflow;

    // Get existing workflow
    let stored = state.ui_workflow_storage.get(&id).await?;
    let mut workflow = match stored {
        Some(stored) => UIWorkflow {
            id: stored.id,
            name: stored.name,
            description: stored.description,
            nodes: serde_json::from_str(&stored.nodes_json).unwrap_or_default(),
            connections: serde_json::from_str(&stored.connections_json).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&stored.created_at)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            updated_at: chrono::DateTime::parse_from_rfc3339(&stored.updated_at)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
        },
        None => return Err(AgentError::NotFound("workflow not found".to_string()).into()),
    };

    // Apply updates
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

    // Save to database
    let stored = StoredWorkflow {
        id: workflow.id.clone(),
        name: workflow.name.clone(),
        description: workflow.description.clone(),
        nodes_json: serde_json::to_string(&workflow.nodes).unwrap_or_else(|_| "[]".to_string()),
        connections_json: serde_json::to_string(&workflow.connections).unwrap_or_else(|_| "[]".to_string()),
        created_at: workflow.created_at.to_rfc3339(),
        updated_at: workflow.updated_at.to_rfc3339(),
    };

    state.ui_workflow_storage.update(&stored).await?;

    Ok(Json(workflow))
}

/// Delete a visual workflow
async fn delete_ui_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let deleted = state.ui_workflow_storage.delete(&id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AgentError::NotFound("workflow not found".to_string()).into())
    }
}

/// Execute a visual workflow (calls the execution handler)
async fn execute_ui_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let stored = state.ui_workflow_storage.get(&id).await?;

    match stored {
        Some(stored_workflow) => {
            // Convert StoredWorkflow to UIWorkflow
            let workflow = UIWorkflow {
                id: stored_workflow.id,
                name: stored_workflow.name,
                description: stored_workflow.description,
                nodes: serde_json::from_str(&stored_workflow.nodes_json).unwrap_or_default(),
                connections: serde_json::from_str(&stored_workflow.connections_json).unwrap_or_default(),
                created_at: chrono::DateTime::parse_from_rfc3339(&stored_workflow.created_at)
                    .ok().map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::parse_from_rfc3339(&stored_workflow.updated_at)
                    .ok().map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(chrono::Utc::now),
            };

            // Convert UI workflow to execution request
            let nodes: Vec<VisualWorkflowNode> = workflow.nodes.iter().map(|n| VisualWorkflowNode {
                id: n.id.clone(),
                node_type: n.node_type.clone(),
                position: Position { x: n.position.x, y: n.position.y },
                config: n.config.clone(),
                label: Some(n.label.clone()),
            }).collect();

            let connections: Vec<VisualWorkflowConnection> = workflow.connections.iter().map(|c| VisualWorkflowConnection {
                id: c.id.clone(),
                from_node: c.from_node.clone(),
                from_output: c.from_output.clone(),
                to_node: c.to_node.clone(),
                to_input: c.to_input.clone(),
                label: None,
                description: None,
            }).collect();

            let request = ExecuteVisualWorkflowRequest {
                id: workflow.id.clone(),
                name: workflow.name.clone(),
                description: Some(workflow.description.clone()),
                nodes,
                connections,
            };

            // Execute the workflow
            match execute_visual_workflow_handler(State(state.clone()), Json(request)).await {
                Ok(Json(response)) => Ok(Json(serde_json::to_value(response).unwrap())),
                Err(e) => Ok(Json(serde_json::json!({
                    "status": "error",
                    "output": format!("Execution failed: {:?}", e)
                }))),
            }
        }
        None => Err(AgentError::NotFound("workflow not found".to_string()).into()),
    }
}

/// List available node types for the workflow builder
async fn list_ui_node_types(State(state): State<AppState>) -> Json<Vec<UINodeType>> {
    let nodes = state.ui_node_types.read().await;
    let node_list: Vec<UINodeType> = nodes.values().cloned().collect();
    Json(node_list)
}

/// Health check for workflow UI
async fn workflow_ui_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Initialize default node types for workflow builder
pub async fn initialize_ui_node_types(state: &AppState) {
    let mut nodes = state.ui_node_types.write().await;

    // LLM Generate Node
    nodes.insert(
        "llm_generate".to_string(),
        UINodeType {
            id: "llm_generate".to_string(),
            name: "LLM Generate".to_string(),
            category: "LLM".to_string(),
            description: "Generate text using a language model".to_string(),
            inputs: vec![
                UINodeInput {
                    name: "prompt".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    description: "The prompt to send to the LLM".to_string(),
                },
                UINodeInput {
                    name: "model".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                    description: "LLM model to use (optional)".to_string(),
                },
            ],
            outputs: vec![UINodeOutput {
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
        UINodeType {
            id: "model_config".to_string(),
            name: "Model Configuration".to_string(),
            category: "LLM".to_string(),
            description: "Configure LLM server connection (Ollama, OpenAI, etc.)".to_string(),
            inputs: vec![],
            outputs: vec![UINodeOutput {
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

    // Text Splitter Node
    nodes.insert(
        "text_splitter".to_string(),
        UINodeType {
            id: "text_splitter".to_string(),
            name: "Text Splitter".to_string(),
            category: "Data Processing".to_string(),
            description: "Split text into chunks".to_string(),
            inputs: vec![UINodeInput {
                name: "text".to_string(),
                r#type: "string".to_string(),
                required: true,
                description: "Text to split".to_string(),
            }],
            outputs: vec![UINodeOutput {
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

    // Conditional Node
    nodes.insert(
        "conditional".to_string(),
        UINodeType {
            id: "conditional".to_string(),
            name: "Conditional".to_string(),
            category: "Control Flow".to_string(),
            description: "Route execution based on conditions".to_string(),
            inputs: vec![
                UINodeInput {
                    name: "condition".to_string(),
                    r#type: "boolean".to_string(),
                    required: true,
                    description: "Boolean condition to evaluate".to_string(),
                },
                UINodeInput {
                    name: "true_value".to_string(),
                    r#type: "any".to_string(),
                    required: false,
                    description: "Value to pass when condition is true".to_string(),
                },
                UINodeInput {
                    name: "false_value".to_string(),
                    r#type: "any".to_string(),
                    required: false,
                    description: "Value to pass when condition is false".to_string(),
                },
            ],
            outputs: vec![UINodeOutput {
                name: "output".to_string(),
                r#type: "any".to_string(),
                description: "Selected output value".to_string(),
            }],
            config_schema: serde_json::json!({}),
        },
    );

    // File Input Node
    nodes.insert(
        "file_input".to_string(),
        UINodeType {
            id: "file_input".to_string(),
            name: "File Input".to_string(),
            category: "I/O".to_string(),
            description: "Read data from a file".to_string(),
            inputs: vec![],
            outputs: vec![UINodeOutput {
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

    // File Output Node
    nodes.insert(
        "file_output".to_string(),
        UINodeType {
            id: "file_output".to_string(),
            name: "File Output".to_string(),
            category: "I/O".to_string(),
            description: "Write data to a file".to_string(),
            inputs: vec![UINodeInput {
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

/// Start the API server
pub async fn start_server(state: AppState, host: &str, port: u16) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    info!("Starting API server on {}", addr);

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AgentError::Network(format!("Failed to bind to {}: {}", addr, e)))?;

    info!("API server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| AgentError::Network(format!("Server error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_request_serialization() {
        let req = ProcessRequest {
            message: "Hello".to_string(),
            max_steps: Some(5),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_health_response() {
        let response = HealthResponse {
            status: "ok".to_string(),
            version: "0.2.0".to_string(),
        };

        assert_eq!(response.status, "ok");
    }
}
