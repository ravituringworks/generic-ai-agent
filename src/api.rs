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
use crate::workflow::{WorkflowContext, WorkflowEngine, WorkflowSnapshot};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// The agent instance
    pub agent: Arc<RwLock<Agent>>,
    /// Workflow engine
    pub workflow_engine: Arc<WorkflowEngine>,
}

impl AppState {
    pub async fn new(config: AgentConfig) -> Result<Self> {
        let agent = AgentBuilder::new().with_config(config).build().await?;
        let workflow_engine = Arc::new(WorkflowEngine::default());

        Ok(Self {
            agent: Arc::new(RwLock::new(agent)),
            workflow_engine,
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

/// Custom error type for API responses
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
