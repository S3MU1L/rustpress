use serde::{Deserialize, Serialize};

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
