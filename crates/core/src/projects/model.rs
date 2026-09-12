//! Project row and paginated list payload.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Database project row.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Project {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Owner-scoped page of projects.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProjectPage {
    pub items: Vec<Project>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
