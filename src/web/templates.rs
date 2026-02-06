use std::collections::HashMap;

use askama::Template;
use uuid::Uuid;

use rustpress::db::UserWithRoles;
use rustpress::models::{
    ContentItem, ContentItemRevisionMeta, Site, SiteTemplate, User,
};

#[derive(Template)]
#[template(path = "public/index.html")]
pub struct PublicIndexTemplate {
    pub posts: Vec<ContentItem>,
}

#[derive(Template)]
#[template(path = "public/content.html")]
pub struct PublicContentTemplate {
    pub html: String,
}

#[derive(Template)]
#[template(path = "public/fallback.html")]
pub struct PublicFallbackTemplate<'a> {
    pub title: &'a str,
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct AdminDashboardTemplate {
    pub posts: Vec<ContentItem>,
    pub pages: Vec<ContentItem>,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/posts_list.html")]
pub struct AdminPostsListTemplate {
    pub posts: Vec<ContentItem>,
    pub authors: HashMap<Uuid, String>,
    pub query: String,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/pages_list.html")]
pub struct AdminPagesListTemplate {
    pub pages: Vec<ContentItem>,
    pub authors: HashMap<Uuid, String>,
    pub query: String,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/edit.html")]
pub struct AdminEditTemplate {
    pub item: ContentItem,
    pub author: String,
    pub templates: Vec<SiteTemplate>,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/new.html")]
pub struct AdminNewTemplate {
    pub kind: String,
    pub default_template: String,
    pub templates: Vec<SiteTemplate>,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/templates_list.html")]
pub struct AdminTemplatesListTemplate {
    pub templates: Vec<SiteTemplate>,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/login.html")]
pub struct AdminLoginTemplate {
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/register.html")]
pub struct AdminRegisterTemplate {
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/template_new.html")]
pub struct AdminTemplateNewTemplate {
    pub starter_html: String,
    pub content_items: Vec<ContentItem>,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/template_edit.html")]
pub struct AdminTemplateEditTemplate {
    pub template: SiteTemplate,
    pub content_items: Vec<ContentItem>,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/account.html")]
pub struct MeAccountTemplate {
    pub user: User,
    pub error: Option<String>,
    pub success: Option<String>,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/security.html")]
pub struct MeSecurityTemplate {
    pub password_set: bool,
    pub email_verified: bool,
    pub error: Option<String>,
    pub success: Option<String>,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/themes.html")]
pub struct ThemesTemplate {
    pub templates: Vec<SiteTemplate>,
    pub query: String,
    pub category: String,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/configuration.html")]
pub struct ConfigurationTemplate {
    pub site: Option<Site>,
    pub pages: Vec<ContentItem>,
    pub error: Option<String>,
    pub success: Option<String>,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "admin/users_list.html")]
pub struct AdminUsersListTemplate {
    pub users: Vec<UserWithRoles>,
    pub current_user_id: Uuid,
    pub is_admin: bool,
    pub error: Option<String>,
    pub success: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/user_new.html")]
pub struct AdminUserNewTemplate {
    pub is_admin: bool,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/user_edit.html")]
pub struct AdminUserEditTemplate {
    pub target_user: User,
    pub target_roles: Vec<String>,
    pub is_admin: bool,
    pub error: Option<String>,
    pub success: Option<String>,
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "401.html")]
pub struct UnauthorizedTemplate {
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "partials/history_panel.html")]
pub struct AdminHistoryPartialTemplate {
    pub revisions: Vec<ContentItemRevisionMeta>,
    pub authors: HashMap<Uuid, String>,
    pub current_rev: i32,
    pub content_item_id: Uuid,
}
