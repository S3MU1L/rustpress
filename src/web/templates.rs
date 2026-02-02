use askama::Template;

use rustpress::models::{ContentItem, Site, SiteTemplate, User};

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
}

#[derive(Template)]
#[template(path = "admin/edit.html")]
pub struct AdminEditTemplate {
    pub item: ContentItem,
    pub templates: Vec<SiteTemplate>,
}

#[derive(Template)]
#[template(path = "admin/new.html")]
pub struct AdminNewTemplate {
    pub kind: String,
    pub default_template: String,
    pub templates: Vec<SiteTemplate>,
}

#[derive(Template)]
#[template(path = "admin/templates_list.html")]
pub struct AdminTemplatesListTemplate {
    pub templates: Vec<SiteTemplate>,
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
}

#[derive(Template)]
#[template(path = "admin/template_edit.html")]
pub struct AdminTemplateEditTemplate {
    pub template: SiteTemplate,
}

#[derive(Template)]
#[template(path = "admin/account.html")]
pub struct MeAccountTemplate {
    pub user: User,
    pub error: Option<String>,
    pub success: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/security.html")]
pub struct MeSecurityTemplate {
    pub password_set: bool,
    pub email_verified: bool,
    pub error: Option<String>,
    pub success: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/sites_list.html")]
pub struct SitesListTemplate {
    pub sites: Vec<Site>,
    pub query: String,
}

#[derive(Template)]
#[template(path = "admin/site_new.html")]
pub struct SiteNewTemplate {
    pub templates: Vec<SiteTemplate>,
    pub default_template: String,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/site_edit.html")]
pub struct SiteEditTemplate {
    pub site: Site,
    pub templates: Vec<SiteTemplate>,
    pub error: Option<String>,
    pub success: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/themes.html")]
pub struct ThemesTemplate {
    pub templates: Vec<SiteTemplate>,
    pub sites: Vec<Site>,
    pub selected_site_id: Option<uuid::Uuid>,
    pub query: String,
    pub category: String,
}

#[derive(Template)]
#[template(path = "admin/configuration.html")]
pub struct ConfigurationTemplate {
    pub site: Option<Site>,
    pub pages: Vec<ContentItem>,
    pub error: Option<String>,
    pub success: Option<String>,
}
