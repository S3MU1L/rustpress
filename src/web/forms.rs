use rustpress::models::RoleName;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
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

#[derive(Deserialize)]
pub struct AccountEmailForm {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct SitesQuery {
    pub q: Option<String>,
}

#[derive(Deserialize)]
pub struct SiteCreateForm {
    pub name: String,
    pub slug: String,
    pub default_template: Option<String>,
}

#[derive(Deserialize)]
pub struct SiteUpdateForm {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub default_template: Option<String>,
    pub homepage_type: Option<String>,
    pub homepage_page_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ThemesQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub site_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ApplyThemeForm {
    pub template: String,
}

#[derive(Deserialize)]
pub struct AdminCreateForm {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub template: Option<String>,
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

#[derive(Deserialize)]
pub struct AdminTemplateUpdateForm {
    pub name: Option<String>,
    pub description: Option<String>,
    pub html: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminCreateUserForm {
    pub email: String,
    pub password: String,
    pub role: RoleName,
}

#[derive(Deserialize)]
pub struct AdminUpdateUserForm {
    pub email: String,
    pub role: RoleName,
    pub new_password: Option<String>,
}
