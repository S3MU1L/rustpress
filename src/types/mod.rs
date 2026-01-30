use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Public user information (safe to expose to frontend)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserPublic {
    pub id: String,
    pub email: String,
    pub username: Option<String>,
}

/// Login request payload
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Successful login response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserPublic,
    pub token: String,
}

/// Registration request payload
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub username: Option<String>,
}

/// Successful registration response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user: UserPublic,
    pub message: String,
}

/// Authentication error details
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthError {
    pub message: String,
    pub code: String,
}

/// Content item summary/detail for frontend consumption
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContentPublic {
    pub id: String,
    pub owner_user_id: Option<String>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TemplateSummary {
    pub id: String,
    pub owner_user_id: Option<String>,
    pub name: String,
    pub description: String,
    pub is_builtin: bool,
    pub created_at: DateTime<Utc>,
    pub edited_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TemplateDetail {
    pub id: String,
    pub owner_user_id: Option<String>,
    pub name: String,
    pub description: String,
    pub html: String,
    pub is_builtin: bool,
    pub created_at: DateTime<Utc>,
    pub edited_at: DateTime<Utc>,
}
