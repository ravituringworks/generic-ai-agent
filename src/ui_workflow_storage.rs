//! Storage layer for UI workflows
//!
//! Persists visual workflow definitions in SQLite

use crate::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

/// Visual workflow stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoredWorkflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes_json: String,
    pub connections_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// UI Workflow Storage
pub struct UIWorkflowStorage {
    pool: SqlitePool,
}

impl UIWorkflowStorage {
    /// Create a new storage instance
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;

        // Create table if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ui_workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                nodes_json TEXT NOT NULL,
                connections_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// List all workflows
    pub async fn list(&self) -> Result<Vec<StoredWorkflow>> {
        let workflows = sqlx::query_as::<_, StoredWorkflow>(
            "SELECT * FROM ui_workflows ORDER BY updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(workflows)
    }

    /// Get a workflow by ID
    pub async fn get(&self, id: &str) -> Result<Option<StoredWorkflow>> {
        let workflow = sqlx::query_as::<_, StoredWorkflow>(
            "SELECT * FROM ui_workflows WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(workflow)
    }

    /// Create a new workflow
    pub async fn create(&self, workflow: &StoredWorkflow) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO ui_workflows (id, name, description, nodes_json, connections_json, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&workflow.id)
        .bind(&workflow.name)
        .bind(&workflow.description)
        .bind(&workflow.nodes_json)
        .bind(&workflow.connections_json)
        .bind(&workflow.created_at)
        .bind(&workflow.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update an existing workflow
    pub async fn update(&self, workflow: &StoredWorkflow) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE ui_workflows
            SET name = ?, description = ?, nodes_json = ?, connections_json = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&workflow.name)
        .bind(&workflow.description)
        .bind(&workflow.nodes_json)
        .bind(&workflow.connections_json)
        .bind(&workflow.updated_at)
        .bind(&workflow.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a workflow
    pub async fn delete(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ui_workflows WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
