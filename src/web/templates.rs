use askama::Template;

use rustpress::models::ContentItem;
use rustpress::models::SiteTemplate;

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
#[template(path = "admin/layout.html")]
pub struct AdminLayoutTemplate {
    pub title: String,
    pub body: String,
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
#[template(path = "admin/template_new.html")]
pub struct AdminTemplateNewTemplate {
    pub starter_html: String,
}

#[derive(Template)]
#[template(path = "admin/template_edit.html")]
pub struct AdminTemplateEditTemplate {
    pub template: SiteTemplate,
}
