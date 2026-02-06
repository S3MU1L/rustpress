use rustpress::models::RoleName;
use serde::Deserialize;

use crate::web::security::validate_slug;

pub const MIN_PASSWORD_LENGTH: usize = 12;
pub const MAX_PASSWORD_LENGTH: usize = 128; // Prevent DoS via huge passwords
pub const MAX_EMAIL_LENGTH: usize = 255;
pub const MAX_CONTENT_LENGTH: usize = 10_000_000; // 10MB
pub const MAX_TEMPLATE_LENGTH: usize = 1_000_000; // 1MB

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

impl LoginForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.email.trim().is_empty() {
            return Err("Email is required");
        }
        if self.email.len() > MAX_EMAIL_LENGTH {
            return Err("Email too long");
        }
        if self.password.is_empty() {
            return Err("Password is required");
        }
        if self.password.len() > MAX_PASSWORD_LENGTH {
            return Err("Password too long");
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct AuthQuery {
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub password: String,
}

impl RegisterForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        let email = self.email.trim();
        if email.is_empty() {
            return Err("Email is required");
        }
        if email.len() > MAX_EMAIL_LENGTH {
            return Err("Email must not exceed 255 characters");
        }
        if !email.contains('@') || !email.contains('.') {
            return Err("Invalid email format");
        }

        if self.password.len() < MIN_PASSWORD_LENGTH {
            return Err("Password must be at least 12 characters");
        }
        if self.password.len() > MAX_PASSWORD_LENGTH {
            return Err("Password must not exceed 128 characters");
        }

        // Check password complexity
        let has_uppercase =
            self.password.chars().any(|c| c.is_uppercase());
        let has_lowercase =
            self.password.chars().any(|c| c.is_lowercase());
        let has_digit = self.password.chars().any(|c| c.is_numeric());
        let has_special =
            self.password.chars().any(|c| !c.is_alphanumeric());

        if !has_uppercase
            || !has_lowercase
            || !has_digit
            || !has_special
        {
            return Err(
                "Password must contain uppercase, lowercase, digit, and special character",
            );
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct AccountEmailForm {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub new_password: String,
}

impl ChangePasswordForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.current_password.len() > MAX_PASSWORD_LENGTH {
            return Err("Current password too long");
        }
        if self.new_password.len() < MIN_PASSWORD_LENGTH {
            return Err(
                "New password must be at least 12 characters",
            );
        }
        if self.new_password.len() > MAX_PASSWORD_LENGTH {
            return Err(
                "New password must not exceed 128 characters",
            );
        }

        // Check new password complexity
        let has_uppercase =
            self.new_password.chars().any(|c| c.is_uppercase());
        let has_lowercase =
            self.new_password.chars().any(|c| c.is_lowercase());
        let has_digit =
            self.new_password.chars().any(|c| c.is_ascii_digit());
        let has_special =
            self.new_password.chars().any(|c| !c.is_alphanumeric());

        if !has_uppercase
            || !has_lowercase
            || !has_digit
            || !has_special
        {
            return Err(
                "New password must contain uppercase, lowercase, digit, and special character",
            );
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Deserialize)]
pub struct ThemesQuery {
    pub q: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminCreateForm {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub template: Option<String>,
}

impl AdminCreateForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.title.trim().is_empty() {
            return Err("Title is required");
        }
        if self.title.len() > 500 {
            return Err("Title must not exceed 500 characters");
        }
        if self.slug.trim().is_empty() {
            return Err("Slug is required");
        }
        // Validate slug format and length
        let slug = self.slug.trim();
        if !validate_slug(slug, Some(200)) {
            return Err(
                "Slug must be lowercase alphanumeric with hyphens/underscores only and not exceed 200 characters",
            );
        }

        if self.content.len() > MAX_CONTENT_LENGTH {
            return Err("Content must not exceed 10MB");
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct AdminUpdateForm {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub template: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminLiveForm {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub template: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminNewPreviewForm {
    pub kind: String,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub template: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminTemplateCreateForm {
    pub name: String,
    pub description: Option<String>,
    pub html: String,
}

impl AdminTemplateCreateForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.trim().is_empty() {
            return Err("Template name is required");
        }
        if self.name.len() > 200 {
            return Err(
                "Template name must not exceed 200 characters",
            );
        }
        if self.html.len() > MAX_TEMPLATE_LENGTH {
            return Err("Template HTML must not exceed 1MB");
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct AdminTemplateUpdateForm {
    pub name: Option<String>,
    pub description: Option<String>,
    pub html: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminTemplatePreviewForm {
    pub html: String,
    pub preview_content_id: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminCreateUserForm {
    pub email: String,
    pub password: String,
    pub role: RoleName,
}

impl AdminCreateUserForm {
    pub fn validate(&self) -> Result<(), &'static str> {
        let email = self.email.trim();
        if email.is_empty() {
            return Err("Email is required");
        }
        if email.len() > MAX_EMAIL_LENGTH {
            return Err("Email must not exceed 255 characters");
        }
        if !email.contains('@') || !email.contains('.') {
            return Err("Invalid email format");
        }

        if self.password.len() < MIN_PASSWORD_LENGTH {
            return Err("Password must be at least 12 characters");
        }
        if self.password.len() > MAX_PASSWORD_LENGTH {
            return Err("Password must not exceed 128 characters");
        }

        // Check password complexity
        let has_uppercase =
            self.password.chars().any(|c| c.is_uppercase());
        let has_lowercase =
            self.password.chars().any(|c| c.is_lowercase());
        let has_digit =
            self.password.chars().any(|c| c.is_ascii_digit());
        let has_special =
            self.password.chars().any(|c| !c.is_alphanumeric());

        if !has_uppercase
            || !has_lowercase
            || !has_digit
            || !has_special
        {
            return Err(
                "Password must contain uppercase, lowercase, digit, and special character",
            );
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct AdminUpdateUserForm {
    pub email: String,
    pub role: RoleName,
    pub new_password: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteAccountForm {
    pub password: String,
}
