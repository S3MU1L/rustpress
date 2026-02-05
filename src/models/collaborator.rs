use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    sqlx::Type,
)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RoleName {
    Admin,
    Editor,
}

impl RoleName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
        }
    }
}

impl std::fmt::Display for RoleName {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentCollaborator {
    pub content_item_id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub role: RoleName,
    pub created_at: DateTime<Utc>,
}
