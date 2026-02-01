use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Site {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub default_template: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteCreate {
    pub owner_user_id: Uuid,
    pub name: String,
    pub slug: String,
    pub default_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteUpdate {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub status: Option<String>,
    pub default_template: Option<String>,
}
