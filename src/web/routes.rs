use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use askama::Template;
use sqlx::PgPool;
use uuid::Uuid;

use rustpress::db;
use rustpress::models::{ContentCreate, ContentKind, ContentStatus, ContentUpdate};
use crate::web::templates::{
    AdminDashboardTemplate, AdminEditTemplate, AdminNewTemplate, PublicContentTemplate,
    PublicIndexTemplate,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

fn is_htmx(req: &HttpRequest) -> bool {
    req.headers()
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| s.eq_ignore_ascii_case("true"))
}

fn render<T: Template>(t: T) -> HttpResponse {
    match t.render() {
        Ok(body) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body),
        Err(e) => HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Template error: {e}")),
    }
}

#[get("/")]
pub async fn public_index(state: web::Data<AppState>) -> impl Responder {
    let posts = db::list_content(&state.pool, ContentKind::Post, false)
        .await
        .unwrap_or_default();

    render(PublicIndexTemplate { posts })
}

#[get("/post/{slug}")]
pub async fn public_post(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let slug = path.into_inner();
    let maybe = db::get_published_by_slug(&state.pool, ContentKind::Post, &slug)
        .await
        .ok()
        .flatten();

    match maybe {
        Some(item) => render(PublicContentTemplate { item }),
        None => HttpResponse::NotFound().body("Not found"),
    }
}

#[get("/p/{slug}")]
pub async fn public_page(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let slug = path.into_inner();
    let maybe = db::get_published_by_slug(&state.pool, ContentKind::Page, &slug)
        .await
        .ok()
        .flatten();

    match maybe {
        Some(item) => render(PublicContentTemplate { item }),
        None => HttpResponse::NotFound().body("Not found"),
    }
}

#[get("/admin")]
pub async fn admin_dashboard(state: web::Data<AppState>) -> impl Responder {
    let posts = db::list_content(&state.pool, ContentKind::Post, true)
        .await
        .unwrap_or_default();
    let pages = db::list_content(&state.pool, ContentKind::Page, true)
        .await
        .unwrap_or_default();

    render(AdminDashboardTemplate { posts, pages })
}

#[get("/admin/{kind}/new")]
pub async fn admin_new(path: web::Path<String>) -> impl Responder {
    let kind = path.into_inner();
    if kind != "posts" && kind != "pages" {
        return HttpResponse::NotFound().body("Unknown kind");
    }

    render(AdminNewTemplate {
        kind,
        default_template: "default".to_string(),
    })
}

#[derive(serde::Deserialize)]
pub struct AdminCreateForm {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub template: Option<String>,
}

#[post("/admin/{kind}")]
pub async fn admin_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    form: web::Form<AdminCreateForm>,
) -> impl Responder {
    let kind = match path.into_inner().as_str() {
        "posts" => ContentKind::Post,
        "pages" => ContentKind::Page,
        _ => return HttpResponse::NotFound().body("Unknown kind"),
    };

    let data = ContentCreate {
        kind,
        title: form.title.trim().to_string(),
        slug: form.slug.trim().to_string(),
        content: form.content.to_string(),
        template: form
            .template
            .clone()
            .unwrap_or_else(|| "default".to_string()),
    };

    let created = match db::create_content(&state.pool, &data).await {
        Ok(item) => item,
        Err(e) => {
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Create failed: {e}"));
        }
    };

    if is_htmx(&req) {
        HttpResponse::Ok()
            .insert_header(("HX-Redirect", format!("/admin/edit/{}", created.id)))
            .finish()
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/admin/edit/{}", created.id)))
            .finish()
    }
}

#[get("/admin/edit/{id}")]
pub async fn admin_edit(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    render(AdminEditTemplate { item })
}

#[derive(serde::Deserialize)]
pub struct AdminUpdateForm {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub template: Option<String>,
    pub status: Option<String>,
}

#[post("/admin/edit/{id}")]
pub async fn admin_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AdminUpdateForm>,
) -> impl Responder {
    let id = path.into_inner();

    let status = match form.status.as_deref().map(|s| s.trim()) {
        Some("draft") => Some(ContentStatus::Draft),
        Some("published") => Some(ContentStatus::Published),
        Some("") | None => None,
        Some(_) => {
            return HttpResponse::BadRequest().body("Invalid status");
        }
    };

    let update = ContentUpdate {
        title: form.title.as_ref().map(|s| s.trim().to_string()),
        slug: form.slug.as_ref().map(|s| s.trim().to_string()),
        content: form.content.clone(),
        template: form.template.as_ref().map(|s| s.trim().to_string()),
        status,
    };

    let updated = match db::update_content(&state.pool, id, &update).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Update failed: {e}"));
        }
    };

    if is_htmx(&req) {
        render(AdminEditTemplate { item: updated })
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/admin/edit/{}", id)))
            .finish()
    }
}

#[post("/admin/publish/{id}")]
pub async fn admin_publish(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    let published = match db::publish_content(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Publish failed: {e}"));
        }
    };

    if is_htmx(&req) {
        render(AdminEditTemplate { item: published })
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/admin/edit/{}", id)))
            .finish()
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(public_index)
        .service(public_post)
        .service(public_page)
        .service(admin_dashboard)
        .service(admin_new)
        .service(admin_create)
        .service(admin_edit)
        .service(admin_update)
        .service(admin_publish);
}
