use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
    Page,
    Post,
}

impl ContentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Post => "post",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentStatus {
    Draft,
    Published,
}

impl ContentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentItem {
    pub id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub kind: String,
    pub status: String,
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
