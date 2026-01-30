use crate::types::*;
use leptos::prelude::*;

/// Login server function - authenticates user credentials
/// Currently returns mock data; replace with real backend calls later
#[server(Login, "/api")]
pub async fn login(email: String, password: String) -> Result<LoginResponse, ServerFnError> {
    if !email.is_empty() && password.len() >= 4 {
        Ok(LoginResponse {
            user: UserPublic {
                id: format!("usr_{}", &email.chars().take(8).collect::<String>()),
                email: email.clone(),
                username: Some(email.split('@').next().unwrap_or("user").into()),
            },
            token: "mock-jwt-token-test".into(),
        })
    } else {
        Err(ServerFnError::new("Invalid credentials"))
    }
}

/// Register server function - creates a new user account
/// Currently returns mock data; replace with real backend calls later
#[server(Register, "/api")]
pub async fn register(
    email: String,
    password: String,
    username: Option<String>,
) -> Result<RegisterResponse, ServerFnError> {
    if password.len() < 4 {
        return Err(ServerFnError::new("Password must be at least 4 characters"));
    }

    // Mock implementation - always succeeds for valid input
    Ok(RegisterResponse {
        user: UserPublic {
            id: format!("usr_{}", &email.chars().take(8).collect::<String>()),
            email: email.clone(),
            username: username.or_else(|| Some(email.split('@').next().unwrap_or("user").into())),
        },
        message: "Registration successful! Welcome to RustPress.".into(),
    })
}

/// Get current user - retrieves the logged-in user's information
/// Currently returns None (no user logged in); implement session handling later
#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<UserPublic>, ServerFnError> {
    // Mock: no user logged in by default
    // Later: check session/JWT token and return user if authenticated
    Ok(None)
}
