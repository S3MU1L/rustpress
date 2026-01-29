use chrono::{DateTime, Utc};
use field_names::FieldNames;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use std::fmt;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct UserLogin {
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdate {
    pub id: Uuid,
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

#[derive(Debug)]
pub struct UserCreate {
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserDelete {
    pub id: Uuid,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum UserIden {
    Id(Uuid),
    Email(String),
}

#[derive(Debug, FieldNames, Default)]
#[field_names(vis = "pub")]
pub struct UserQuery {
    pub id: Option<Uuid>,
    pub email: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    //------------------------------------
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub sort_by: Option<Vec<Option<bool>>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserWithOldState {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub old_edited_at: DateTime<Utc>,
}

fn check_email_purged(email: &str) -> bool {
    email.starts_with("anon-")
        && email.ends_with("@deleted.local")
        && email.len() > "anon-@deleted.local".len()
}

impl User {
    pub fn has_email_purged(&self) -> bool {
        check_email_purged(&self.email)
    }
}

impl UserWithOldState {
    pub fn has_email_purged(&self) -> bool {
        check_email_purged(&self.email)
    }
}

impl From<UserWithOldState> for User {
    fn from(
        UserWithOldState {
            id,
            email,
            password_hash,
            created_at,
            edited_at,
            deleted_at,
            ..
        }: UserWithOldState,
    ) -> Self {
        User {
            id,
            email,
            password_hash,
            created_at,
            edited_at,
            deleted_at,
        }
    }
}

impl UserUpdate {
    pub fn is_empty(&self) -> bool {
        self.email.is_none() && self.password_hash.is_none()
    }
}

impl UserQuery {
    pub fn fields() -> &'static [&'static str] {
        &Self::FIELDS
    }

    pub fn is_empty(&self) -> bool {
        matches!(
            self,
            Self {
                id: None,
                email: None,
                created_at: None,
                edited_at: None,
                deleted_at: None,
                ..
            }
        )
    }
}

impl fmt::Display for UserIden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserIden::Id(id) => write!(f, "ID {}", id),
            UserIden::Email(email) => write!(f, "email '{}'", email),
        }
    }
}

impl From<Uuid> for UserIden {
    fn from(id: Uuid) -> Self {
        UserIden::Id(id)
    }
}

impl From<String> for UserIden {
    fn from(email: String) -> Self {
        UserIden::Email(email)
    }
}

impl From<&String> for UserIden {
    fn from(email: &String) -> Self {
        UserIden::Email(email.clone())
    }
}

impl From<&str> for UserIden {
    fn from(email: &str) -> Self {
        UserIden::Email(email.to_string())
    }
}
