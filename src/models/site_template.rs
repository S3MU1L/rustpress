use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SiteTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub html: String,
    pub is_builtin: bool,
    pub created_at: DateTime<Utc>,
    pub edited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteTemplateCreate {
    pub name: String,
    pub description: String,
    pub html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteTemplateUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub html: Option<String>,
}
