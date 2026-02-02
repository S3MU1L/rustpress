use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::{ContentKind, ContentStatus};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentItem {
    pub id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub kind: ContentKind,
    pub status: ContentStatus,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub template: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCreate {
    pub owner_user_id: Option<Uuid>,
    pub kind: ContentKind,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentUpdate {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub template: Option<String>,
    pub status: Option<ContentStatus>,
}
