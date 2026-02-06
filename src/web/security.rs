use actix_web::{HttpRequest, cookie::Cookie};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// CSRF token management
#[allow(dead_code)]
pub struct CsrfToken;

#[allow(dead_code)]
impl CsrfToken {
    /// Generate a new CSRF token
    pub fn generate() -> String {
        Uuid::new_v4().to_string()
    }

    /// Validate CSRF token from request
    pub fn validate(req: &HttpRequest, form_token: &str) -> bool {
        if let Some(session_token) = req.cookie("csrf_token") {
            let session_value = session_token.value();
            !session_value.is_empty() && session_value == form_token
        } else {
            false
        }
    }

    /// Create a CSRF token cookie
    pub fn create_cookie(token: &str) -> Cookie<'static> {
        Cookie::build("csrf_token", token.to_string())
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(actix_web::cookie::SameSite::Strict)
            .max_age(actix_web::cookie::time::Duration::hours(24))
            .finish()
    }
}

/// Simple in-memory rate limiter
pub struct RateLimiter {
    requests: Mutex<HashMap<String, Vec<SystemTime>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
        }
    }

    /// Check if request should be allowed
    /// Returns true if allowed, false if rate limit exceeded
    pub fn check_rate_limit(
        &self,
        key: &str,
        max_requests: usize,
        window: Duration,
    ) -> bool {
        let now = SystemTime::now();
        let mut requests = self
            .requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let entry =
            requests.entry(key.to_string()).or_default();

        // Remove old requests outside the window
        entry.retain(|&time| {
            now.duration_since(time).unwrap_or(Duration::from_secs(0))
                < window
        });

        if entry.len() >= max_requests {
            return false;
        }

        entry.push(now);

        // Cleanup: remove empty entries to prevent unbounded memory growth
        requests.retain(|_, times| !times.is_empty());

        true
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Password validation
pub struct PasswordValidator;

impl PasswordValidator {
    const MIN_LENGTH: usize = 12;

    pub fn validate(password: &str) -> Result<(), String> {
        if password.len() < Self::MIN_LENGTH {
            return Err(format!(
                "Password must be at least {} characters",
                Self::MIN_LENGTH
            ));
        }

        let has_uppercase =
            password.chars().any(|c| c.is_uppercase());
        let has_lowercase =
            password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special =
            password.chars().any(|c| !c.is_alphanumeric());

        let mut requirements_met = 0;
        if has_uppercase {
            requirements_met += 1;
        }
        if has_lowercase {
            requirements_met += 1;
        }
        if has_digit {
            requirements_met += 1;
        }
        if has_special {
            requirements_met += 1;
        }

        if requirements_met < 3 {
            return Err(
                "Password must contain at least 3 of: uppercase, lowercase, digit, special character"
                    .to_string(),
            );
        }

        Ok(())
    }
}

/// Email validation
pub fn validate_email(email: &str) -> bool {
    let email = email.trim();

    if email.is_empty() || email.len() > 254 {
        return false;
    }

    // Basic email validation
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() || local.len() > 64 || domain.is_empty() {
        return false;
    }

    // Domain must have at least one dot
    if !domain.contains('.') {
        return false;
    }

    true
}

/// Slug validation
pub fn validate_slug(slug: &str) -> bool {
    if slug.is_empty() || slug.len() > 255 {
        return false;
    }

    // Slug should only contain lowercase alphanumeric, hyphens, and underscores
    slug.chars().all(|c| {
        c.is_ascii_lowercase()
            || c.is_ascii_digit()
            || c == '-'
            || c == '_'
    })
}

/// Generic error message for security
pub fn generic_error_message(context: &str) -> String {
    format!(
        "An error occurred while processing your {}. Please try again.",
        context
    )
}
