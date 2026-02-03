use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::ContentStatus;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentItemRevision {
    pub id: Uuid,
    pub content_item_id: Uuid,
    pub rev: i32,
    pub created_by_user_id: Option<Uuid>,

    pub title: String,
    pub slug: String,
    pub content: String,
    pub template: String,
    pub status: ContentStatus,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentItemRevisionMeta {
    pub rev: i32,
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub title: String,
    pub status: ContentStatus,
}
