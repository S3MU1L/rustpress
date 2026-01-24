use askama::Template;

use rustpress::models::ContentItem;

#[derive(Template)]
#[template(path = "public/index.html")]
pub struct PublicIndexTemplate {
    pub posts: Vec<ContentItem>,
}

#[derive(Template)]
#[template(path = "public/content.html")]
pub struct PublicContentTemplate {
    pub item: ContentItem,
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
}

#[derive(Template)]
#[template(path = "admin/new.html")]
pub struct AdminNewTemplate {
    pub kind: String,
    pub default_template: String,
}
