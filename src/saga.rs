//! Saga pattern implementation for distributed transactions and workflow compensation
//!
//! A saga is a sequence of local transactions where each transaction has a compensating transaction.
//! If any transaction fails, the saga executes compensation transactions in reverse order.
//!
//! This implementation provides:
//! - Forward execution of saga steps
//! - Automatic compensation on failure
//! - State persistence for long-running sagas
//! - Saga orchestration and coordination

use crate::error::{AgentError, Result};
use crate::workflow::{WorkflowContext, WorkflowDecision, WorkflowStep};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Saga execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SagaResult {
    /// Saga completed successfully
    Completed(serde_json::Value),

    /// Saga failed and was compensated
    Compensated {
        error: String,
        compensated_steps: Vec<String>,
    },

    /// Saga failed and compensation also failed
    CompensationFailed {
        error: String,
        compensation_error: String,
        failed_at_step: String,
    },
}

/// Saga step state for tracking execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagaStepState {
    /// Step has not been executed yet
    Pending,

    /// Step is currently executing
    Executing,

    /// Step executed successfully
    Completed,

    /// Step failed during execution
    Failed(String),

    /// Step is being compensated
    Compensating,

    /// Step was successfully compensated
    Compensated,

    /// Compensation failed
    CompensationFailed(String),
}

/// A single step in a saga with its compensation
pub struct SagaStep {
    /// Step identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Forward action to execute
    pub action: Box<dyn Fn(&mut WorkflowContext) -> Result<serde_json::Value> + Send + Sync>,

    /// Compensation action (rollback)
    pub compensation:
        Box<dyn Fn(&mut WorkflowContext, &serde_json::Value) -> Result<()> + Send + Sync>,

    /// Whether this step can be retried on failure
    pub retryable: bool,

    /// Maximum retry attempts
    pub max_retries: usize,
}

impl SagaStep {
    pub fn new<F, C>(id: &str, name: &str, action: F, compensation: C) -> Self
    where
        F: Fn(&mut WorkflowContext) -> Result<serde_json::Value> + Send + Sync + 'static,
        C: Fn(&mut WorkflowContext, &serde_json::Value) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            action: Box::new(action),
            compensation: Box::new(compensation),
            retryable: true,
            max_retries: 3,
        }
    }

    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn non_retryable(mut self) -> Self {
        self.retryable = false;
        self.max_retries = 0;
        self
    }
}

/// Saga execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaContext {
    /// Saga identifier
    pub id: Uuid,

    /// Saga name
    pub name: String,

    /// When the saga started
    pub started_at: DateTime<Utc>,

    /// When the saga ended (if completed)
    pub ended_at: Option<DateTime<Utc>>,

    /// State of each step
    pub step_states: HashMap<String, SagaStepState>,

    /// Results from completed steps
    pub step_results: HashMap<String, serde_json::Value>,

    /// Retry count for each step
    pub retry_counts: HashMap<String, usize>,

    /// Workflow context
    pub workflow_context: WorkflowContext,
}

impl SagaContext {
    pub fn new(name: String, workflow_context: WorkflowContext) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            started_at: Utc::now(),
            ended_at: None,
            step_states: HashMap::new(),
            step_results: HashMap::new(),
            retry_counts: HashMap::new(),
            workflow_context,
        }
    }

    pub fn is_step_completed(&self, step_id: &str) -> bool {
        matches!(
            self.step_states.get(step_id),
            Some(SagaStepState::Completed)
        )
    }

    pub fn mark_step_executing(&mut self, step_id: &str) {
        self.step_states
            .insert(step_id.to_string(), SagaStepState::Executing);
    }

    pub fn mark_step_completed(&mut self, step_id: &str, result: serde_json::Value) {
        self.step_states
            .insert(step_id.to_string(), SagaStepState::Completed);
        self.step_results.insert(step_id.to_string(), result);
    }

    pub fn mark_step_failed(&mut self, step_id: &str, error: String) {
        self.step_states
            .insert(step_id.to_string(), SagaStepState::Failed(error));
    }

    pub fn mark_step_compensating(&mut self, step_id: &str) {
        self.step_states
            .insert(step_id.to_string(), SagaStepState::Compensating);
    }

    pub fn mark_step_compensated(&mut self, step_id: &str) {
        self.step_states
            .insert(step_id.to_string(), SagaStepState::Compensated);
    }

    pub fn mark_step_compensation_failed(&mut self, step_id: &str, error: String) {
        self.step_states.insert(
            step_id.to_string(),
            SagaStepState::CompensationFailed(error),
        );
    }

    pub fn increment_retry(&mut self, step_id: &str) -> usize {
        let count = self.retry_counts.entry(step_id.to_string()).or_insert(0);
        *count += 1;
        *count
    }
}

/// Saga orchestrator - manages saga execution
pub struct SagaOrchestrator {
    steps: Vec<SagaStep>,
}

impl SagaOrchestrator {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(mut self, step: SagaStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Execute the saga
    pub async fn execute(&self, mut context: SagaContext) -> Result<SagaResult> {
        info!(
            "Starting saga '{}' with {} steps",
            context.name,
            self.steps.len()
        );

        // Execute forward steps
        for (index, step) in self.steps.iter().enumerate() {
            info!("Executing saga step {}: {}", index + 1, step.name);
            context.mark_step_executing(&step.id);

            let result = self.execute_step_with_retry(step, &mut context).await;

            match result {
                Ok(step_result) => {
                    context.mark_step_completed(&step.id, step_result);
                    info!("Saga step '{}' completed successfully", step.name);
                }
                Err(e) => {
                    error!("Saga step '{}' failed: {}", step.name, e);
                    context.mark_step_failed(&step.id, e.to_string());

                    // Compensate previous steps
                    return self.compensate(context, index).await;
                }
            }
        }

        // All steps completed successfully
        context.ended_at = Some(Utc::now());
        info!("Saga '{}' completed successfully", context.name);

        let final_result = context
            .step_results
            .values()
            .last()
            .cloned()
            .unwrap_or(serde_json::json!({}));

        Ok(SagaResult::Completed(final_result))
    }

    /// Execute a step with retry logic
    async fn execute_step_with_retry(
        &self,
        step: &SagaStep,
        context: &mut SagaContext,
    ) -> Result<serde_json::Value> {
        let mut last_error = None;

        let max_attempts = if step.retryable {
            step.max_retries + 1
        } else {
            1
        };

        for attempt in 1..=max_attempts {
            if attempt > 1 {
                debug!(
                    "Retrying saga step '{}' (attempt {}/{})",
                    step.name, attempt, max_attempts
                );
            }

            match (step.action)(&mut context.workflow_context) {
                Ok(result) => {
                    return Ok(result);
                }
                Err(e) => {
                    warn!(
                        "Saga step '{}' attempt {} failed: {}",
                        step.name, attempt, e
                    );
                    last_error = Some(e);

                    if attempt < max_attempts {
                        context.increment_retry(&step.id);
                        // Exponential backoff
                        let delay =
                            std::time::Duration::from_millis(100 * 2u64.pow((attempt - 1) as u32));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AgentError::Workflow("Step execution failed".to_string())))
    }

    /// Compensate completed steps in reverse order
    async fn compensate(
        &self,
        mut context: SagaContext,
        failed_at_index: usize,
    ) -> Result<SagaResult> {
        info!("Starting compensation from step index {}", failed_at_index);

        let mut compensated_steps = Vec::new();
        let mut compensation_error = None;

        // Compensate in reverse order
        for index in (0..failed_at_index).rev() {
            let step = &self.steps[index];

            if !context.is_step_completed(&step.id) {
                continue;
            }

            info!("Compensating step: {}", step.name);
            context.mark_step_compensating(&step.id);

            let step_result = context
                .step_results
                .get(&step.id)
                .cloned()
                .unwrap_or(serde_json::json!({}));

            match (step.compensation)(&mut context.workflow_context, &step_result) {
                Ok(_) => {
                    context.mark_step_compensated(&step.id);
                    compensated_steps.push(step.name.clone());
                    info!("Successfully compensated step: {}", step.name);
                }
                Err(e) => {
                    error!("Compensation failed for step '{}': {}", step.name, e);
                    context.mark_step_compensation_failed(&step.id, e.to_string());
                    compensation_error = Some((e.to_string(), step.name.clone()));
                    break;
                }
            }
        }

        context.ended_at = Some(Utc::now());

        if let Some((comp_err, failed_step)) = compensation_error {
            let original_error = self.steps[failed_at_index].name.clone();
            error!("Saga compensation failed at step: {}", failed_step);

            Ok(SagaResult::CompensationFailed {
                error: original_error,
                compensation_error: comp_err,
                failed_at_step: failed_step,
            })
        } else {
            let original_error = self.steps[failed_at_index].name.clone();
            info!(
                "Saga successfully compensated {} steps",
                compensated_steps.len()
            );

            Ok(SagaResult::Compensated {
                error: original_error,
                compensated_steps,
            })
        }
    }
}

impl Default for SagaOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow step that executes a saga
pub struct SagaWorkflowStep {
    orchestrator: SagaOrchestrator,
    saga_name: String,
}

impl SagaWorkflowStep {
    pub fn new(saga_name: String, orchestrator: SagaOrchestrator) -> Self {
        Self {
            orchestrator,
            saga_name,
        }
    }
}

#[async_trait]
impl WorkflowStep for SagaWorkflowStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<WorkflowDecision> {
        info!("Executing saga workflow step: {}", self.saga_name);

        let saga_context = SagaContext::new(self.saga_name.clone(), context.clone());

        match self.orchestrator.execute(saga_context).await? {
            SagaResult::Completed(result) => {
                context.metadata.insert(
                    "saga_result".to_string(),
                    serde_json::to_string(&result).unwrap_or_default(),
                );
                Ok(WorkflowDecision::Continue)
            }
            SagaResult::Compensated {
                error,
                compensated_steps,
            } => {
                warn!(
                    "Saga compensated after error: {}. Compensated {} steps.",
                    error,
                    compensated_steps.len()
                );
                Err(AgentError::Workflow(format!(
                    "Saga failed and was compensated: {}",
                    error
                )))
            }
            SagaResult::CompensationFailed {
                error,
                compensation_error,
                failed_at_step,
            } => {
                error!(
                    "Saga compensation failed at step '{}': {}",
                    failed_at_step, compensation_error
                );
                Err(AgentError::Workflow(format!(
                    "Saga failed ({}) and compensation also failed at '{}': {}",
                    error, failed_at_step, compensation_error
                )))
            }
        }
    }

    fn name(&self) -> &str {
        &self.saga_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::WorkflowContext;

    #[tokio::test]
    async fn test_saga_success() {
        let step1 = SagaStep::new(
            "step1",
            "First Step",
            |_ctx| Ok(serde_json::json!({"value": 1})),
            |_ctx, _result| Ok(()),
        );

        let step2 = SagaStep::new(
            "step2",
            "Second Step",
            |_ctx| Ok(serde_json::json!({"value": 2})),
            |_ctx, _result| Ok(()),
        );

        let orchestrator = SagaOrchestrator::new().add_step(step1).add_step(step2);

        let workflow_ctx = WorkflowContext::new(10);
        let saga_ctx = SagaContext::new("test-saga".to_string(), workflow_ctx);

        let result = orchestrator.execute(saga_ctx).await.unwrap();

        assert!(matches!(result, SagaResult::Completed(_)));
    }

    #[tokio::test]
    async fn test_saga_compensation() {
        let step1 = SagaStep::new(
            "step1",
            "First Step",
            move |_ctx| Ok(serde_json::json!({"value": 1})),
            move |_ctx, _result| Ok(()),
        );

        let step2 = SagaStep::new(
            "step2",
            "Failing Step",
            |_ctx| Err(AgentError::Workflow("Intentional failure".to_string())),
            |_ctx, _result| Ok(()),
        )
        .non_retryable();

        let orchestrator = SagaOrchestrator::new().add_step(step1).add_step(step2);

        let workflow_ctx = WorkflowContext::new(10);
        let saga_ctx = SagaContext::new("test-saga".to_string(), workflow_ctx);

        let result = orchestrator.execute(saga_ctx).await.unwrap();

        assert!(matches!(result, SagaResult::Compensated { .. }));
    }
}
