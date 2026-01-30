use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use actix_web::cookie::{Cookie, SameSite};
use askama::Template;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use rustpress::db;
use rustpress::models::{ContentCreate, ContentKind, ContentStatus, ContentUpdate, User};
use rustpress::services::PasswordManager;
use crate::web::templates::{
    AdminDashboardTemplate, AdminEditTemplate, AdminNewTemplate, AdminTemplateEditTemplate,
    AdminTemplateNewTemplate, AdminTemplatesListTemplate, AdminLoginTemplate, AdminRegisterTemplate,
    PublicContentTemplate, PublicIndexTemplate,
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

fn current_user_id(req: &HttpRequest) -> Option<Uuid> {
    // MVP auth/session.
    // Priority: cookie -> request header -> env var.
    let cookie_val = req
        .cookie("rp_uid")
        .map(|c| c.value().trim().to_string())
        .filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(&s).ok());

    if cookie_val.is_some() {
        return cookie_val;
    }

    let header_val = req
        .headers()
        .get("X-Rustpress-User-Id")
        .or_else(|| req.headers().get("X-User-Id"))
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(s).ok());

    header_val.or_else(|| {
        std::env::var("RUSTPRESS_USER_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .and_then(|s| Uuid::parse_str(&s).ok())
    })
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[get("/login")]
pub async fn login_form() -> impl Responder {
    render(AdminLoginTemplate { error: None })
}

#[derive(serde::Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub password: String,
}

#[get("/register")]
pub async fn register_form() -> impl Responder {
    render(AdminRegisterTemplate { error: None })
}

#[post("/register")]
pub async fn register_submit(
    state: web::Data<AppState>,
    form: web::Form<RegisterForm>,
) -> impl Responder {
    let email = form.email.trim().to_string();
    let password = form.password.to_string();

    if email.is_empty() || password.len() < 4 {
        return render(AdminRegisterTemplate {
            error: Some("Email required and password must be at least 4 characters".to_string()),
        });
    }

    let password_hash = match PasswordManager::hash_password(&password) {
        Ok(h) => h,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Password hashing error: {e}"));
        }
    };

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        ON CONFLICT (email) DO NOTHING
        RETURNING *
        "#,
    )
    .bind(&email)
    .bind(&password_hash)
    .fetch_optional(&state.pool)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            return render(AdminRegisterTemplate {
                error: Some("Email already exists".to_string()),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Database error: {e}"));
        }
    };

    let cookie = Cookie::build("rp_uid", user.id.to_string())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish();

    HttpResponse::SeeOther()
        .cookie(cookie)
        .insert_header(("Location", "/admin"))
        .finish()
}

#[post("/login")]
pub async fn login_submit(
    state: web::Data<AppState>,
    form: web::Form<LoginForm>,
) -> impl Responder {
    let email = form.email.trim().to_string();
    let password = form.password.to_string();

    if email.is_empty() || password.is_empty() {
        return HttpResponse::BadRequest().body("Missing email/password");
    }

    let user = sqlx::query_as::<_, User>(
        r#"SELECT * FROM users WHERE email = $1"#,
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            return render(AdminLoginTemplate {
                error: Some("Invalid credentials".to_string()),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Database error: {e}"));
        }
    };

    let ok = match PasswordManager::verify_password(&password, &user.password_hash) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Password verification error: {e}"));
        }
    };

    if !ok {
        return render(AdminLoginTemplate {
            error: Some("Invalid credentials".to_string()),
        });
    }

    let cookie = Cookie::build("rp_uid", user.id.to_string())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish();

    HttpResponse::SeeOther()
        .cookie(cookie)
        .insert_header(("Location", "/admin"))
        .finish()
}

#[post("/logout")]
pub async fn logout(req: HttpRequest) -> impl Responder {
    let mut cookie = Cookie::build("rp_uid", "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish();
    cookie.make_removal();

    if is_htmx(&req) {
        HttpResponse::Ok()
            .cookie(cookie)
            .insert_header(("HX-Redirect", "/login"))
            .finish()
    } else {
        HttpResponse::SeeOther()
            .cookie(cookie)
            .insert_header(("Location", "/login"))
            .finish()
    }
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

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn iframe_srcdoc(html: &str) -> String {
    // `srcdoc` is an attribute; escape enough to keep it valid.
    // Browsers will decode entities inside attributes.
    format!(
        r#"<iframe class=\"preview-iframe\" sandbox=\"allow-same-origin\" referrerpolicy=\"no-referrer\" srcdoc=\"{}\"></iframe>"#,
        escape_html(html)
    )
}

fn apply_site_template(template_html: &str, title: &str, content_html: &str, slug: &str, kind: &str) -> String {
    // Very small placeholder system.
    // - title/slug/kind are escaped
    // - content is inserted as-is (admin-authored)
    let title_e = escape_html(title);
    let slug_e = escape_html(slug);
    let kind_e = escape_html(kind);

    let replacements: [(&str, &str); 8] = [
        ("{{title}}", title_e.as_str()),
        ("{{ title }}", title_e.as_str()),
        ("{{slug}}", slug_e.as_str()),
        ("{{ slug }}", slug_e.as_str()),
        ("{{kind}}", kind_e.as_str()),
        ("{{ kind }}", kind_e.as_str()),
        ("{{content}}", content_html),
        ("{{ content }}", content_html),
    ];

    let mut out = template_html.to_string();
    for (needle, replacement) in replacements {
        out = out.replace(needle, replacement);
    }
    out
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
        Some(item) => {
            let mut tpl = match item.owner_user_id {
                Some(owner_id) => db::get_site_template_by_name_for_user(
                    &state.pool,
                    owner_id,
                    &item.template,
                )
                .await
                .ok()
                .flatten(),
                None => db::get_site_template_by_name(&state.pool, &item.template)
                    .await
                    .ok()
                    .flatten(),
            };
            if tpl.is_none() {
                tpl = match item.owner_user_id {
                    Some(owner_id) => db::get_site_template_by_name_for_user(
                        &state.pool,
                        owner_id,
                        "default",
                    )
                    .await
                    .ok()
                    .flatten(),
                    None => db::get_site_template_by_name(&state.pool, "default")
                        .await
                        .ok()
                        .flatten(),
                };
            }

            let html = match tpl {
                Some(tpl) => apply_site_template(
                    &tpl.html,
                    &item.title,
                    &item.content,
                    &item.slug,
                    &item.kind,
                ),
                None => apply_site_template(
                    "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{title}}</title></head><body><h1>{{title}}</h1>{{content}}</body></html>",
                    &item.title,
                    &item.content,
                    &item.slug,
                    &item.kind,
                ),
            };

            render(PublicContentTemplate { html })
        }
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
        Some(item) => {
            let mut tpl = match item.owner_user_id {
                Some(owner_id) => db::get_site_template_by_name_for_user(
                    &state.pool,
                    owner_id,
                    &item.template,
                )
                .await
                .ok()
                .flatten(),
                None => db::get_site_template_by_name(&state.pool, &item.template)
                    .await
                    .ok()
                    .flatten(),
            };
            if tpl.is_none() {
                tpl = match item.owner_user_id {
                    Some(owner_id) => db::get_site_template_by_name_for_user(
                        &state.pool,
                        owner_id,
                        "default",
                    )
                    .await
                    .ok()
                    .flatten(),
                    None => db::get_site_template_by_name(&state.pool, "default")
                        .await
                        .ok()
                        .flatten(),
                };
            }

            let html = match tpl {
                Some(tpl) => apply_site_template(
                    &tpl.html,
                    &item.title,
                    &item.content,
                    &item.slug,
                    &item.kind,
                ),
                None => apply_site_template(
                    "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{title}}</title></head><body><h1>{{title}}</h1>{{content}}</body></html>",
                    &item.title,
                    &item.content,
                    &item.slug,
                    &item.kind,
                ),
            };

            render(PublicContentTemplate { html })
        }
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

#[get("/admin/{kind:posts|pages}/new")]
pub async fn admin_new(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let kind = path.into_inner();

    let templates = match current_user_id(&req) {
        Some(uid) => db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default(),
        None => db::list_site_templates(&state.pool)
            .await
            .unwrap_or_default(),
    };

    render(AdminNewTemplate {
        kind,
        default_template: "default".to_string(),
        templates,
    })
}

#[derive(serde::Deserialize)]
pub struct AdminCreateForm {
    pub title: String,
    pub slug: String,
    pub content: String,
    pub template: Option<String>,
}

#[post("/admin/{kind:posts|pages}")]
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
        owner_user_id: current_user_id(&req),
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
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let templates = match item.owner_user_id.or_else(|| current_user_id(&req)) {
        Some(uid) => db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default(),
        None => db::list_site_templates(&state.pool)
            .await
            .unwrap_or_default(),
    };
    render(AdminEditTemplate { item, templates })
}

#[get("/admin/templates")]
pub async fn admin_templates_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let templates = match current_user_id(&req) {
        Some(uid) => db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default(),
        None => db::list_site_templates(&state.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|t| t.owner_user_id.is_none())
            .collect(),
    };
    render(AdminTemplatesListTemplate { templates })
}

#[get("/admin/templates/new")]
pub async fn admin_template_new() -> impl Responder {
    let starter_html = "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\"/>\n    <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"/>\n    <title>{{title}}</title>\n    <link rel=\"stylesheet\" href=\"/static/app.css\"/>\n  </head>\n  <body>\n    <header class=\"topbar\">\n      <div class=\"container\">\n        <a class=\"brand\" href=\"/\">RustPress</a>\n        <nav class=\"nav\"><a href=\"/admin\">Admin</a></nav>\n      </div>\n    </header>\n    <main class=\"container\">\n      <article class=\"card\">\n        <h1>{{title}}</h1>\n        <div class=\"prose\">{{content}}</div>\n      </article>\n    </main>\n  </body>\n</html>\n".to_string();
    render(AdminTemplateNewTemplate { starter_html })
}

#[derive(serde::Deserialize)]
pub struct AdminTemplateCreateForm {
    pub name: String,
    pub description: Option<String>,
    pub html: String,
}

#[post("/admin/templates")]
pub async fn admin_template_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<AdminTemplateCreateForm>,
) -> impl Responder {
    let owner_user_id = match current_user_id(&req) {
        Some(uid) => uid,
        None => {
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(
                    "Missing user context. Set header X-Rustpress-User-Id or env RUSTPRESS_USER_ID (UUID).",
                );
        }
    };

    let data = rustpress::models::SiteTemplateCreate {
        owner_user_id,
        name: form.name.trim().to_string(),
        description: form.description.clone().unwrap_or_default(),
        html: form.html.clone(),
    };

    let created = match db::create_site_template(&state.pool, &data).await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Create failed: {e}"));
        }
    };

    if is_htmx(&req) {
        HttpResponse::Ok()
            .insert_header(("HX-Redirect", format!("/admin/templates/{}", created.id)))
            .finish()
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/admin/templates/{}", created.id)))
            .finish()
    }
}

#[get("/admin/templates/{id}")]
pub async fn admin_template_edit(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let template = match db::get_site_template_by_id(&state.pool, id).await {
        Ok(Some(t)) => t,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !template.is_builtin {
        let uid = match current_user_id(&req) {
            Some(uid) => uid,
            None => {
                return HttpResponse::BadRequest()
                    .content_type("text/plain; charset=utf-8")
                    .body(
                        "Missing user context. Set header X-Rustpress-User-Id or env RUSTPRESS_USER_ID (UUID).",
                    );
            }
        };

        if template.owner_user_id != Some(uid) {
            return HttpResponse::Forbidden().body("Forbidden");
        }
    }
    render(AdminTemplateEditTemplate { template })
}

#[derive(serde::Deserialize)]
pub struct AdminTemplateUpdateForm {
    pub name: Option<String>,
    pub description: Option<String>,
    pub html: Option<String>,
}

#[post("/admin/templates/{id}")]
pub async fn admin_template_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AdminTemplateUpdateForm>,
) -> impl Responder {
    let id = path.into_inner();

    let existing = match db::get_site_template_by_id(&state.pool, id).await {
        Ok(Some(t)) => t,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if existing.is_builtin {
        return HttpResponse::Forbidden().body("Built-in templates are read-only");
    }

    let uid = match current_user_id(&req) {
        Some(uid) => uid,
        None => {
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(
                    "Missing user context. Set header X-Rustpress-User-Id or env RUSTPRESS_USER_ID (UUID).",
                );
        }
    };

    if existing.owner_user_id != Some(uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }
    let update = rustpress::models::SiteTemplateUpdate {
        name: form.name.as_ref().map(|s| s.trim().to_string()),
        description: form.description.clone(),
        html: form.html.clone(),
    };

    let updated = match db::update_site_template(&state.pool, id, &update).await {
        Ok(Some(t)) => t,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Update failed: {e}"));
        }
    };

    if is_htmx(&req) {
        render(AdminTemplateEditTemplate { template: updated })
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/admin/templates/{}", id)))
            .finish()
    }
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
        let templates = match updated.owner_user_id.or_else(|| current_user_id(&req)) {
            Some(uid) => db::list_site_templates_for_user(&state.pool, uid)
                .await
                .unwrap_or_default(),
            None => db::list_site_templates(&state.pool)
                .await
                .unwrap_or_default(),
        };
        render(AdminEditTemplate {
            item: updated,
            templates,
        })
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
        let templates = match published.owner_user_id.or_else(|| current_user_id(&req)) {
            Some(uid) => db::list_site_templates_for_user(&state.pool, uid)
                .await
                .unwrap_or_default(),
            None => db::list_site_templates(&state.pool)
                .await
                .unwrap_or_default(),
        };
        render(AdminEditTemplate {
            item: published,
            templates,
        })
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/admin/edit/{}", id)))
            .finish()
    }
}

#[derive(serde::Deserialize)]
pub struct AdminLiveForm {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub template: Option<String>,
}

#[post("/admin/edit/{id}/autosave")]
pub async fn admin_autosave(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    form: web::Form<AdminLiveForm>,
) -> impl Responder {
    let id = path.into_inner();

    // Autosave should never implicitly publish.
    let update = ContentUpdate {
        title: form.title.as_ref().map(|s| s.trim().to_string()),
        slug: form.slug.as_ref().map(|s| s.trim().to_string()),
        content: form.content.clone(),
        template: form.template.as_ref().map(|s| s.trim().to_string()),
        status: None,
    };

    match db::update_content(&state.pool, id, &update).await {
        Ok(Some(_)) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(format!(
                "<span class=\"muted\">Autosaved at {}</span>",
                Utc::now().format("%H:%M:%S")
            )),
        Ok(None) => HttpResponse::NotFound().body("Not found"),
        Err(e) => HttpResponse::BadRequest()
            .content_type("text/html; charset=utf-8")
            .body(format!(
                "<span class=\"muted\">Autosave failed: {}</span>",
                escape_html(&e.to_string())
            )),
    }
}

#[post("/admin/edit/{id}/preview")]
pub async fn admin_preview(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    form: web::Form<AdminLiveForm>,
) -> impl Responder {
    let id = path.into_inner();
    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let title = form
        .title
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| item.title.clone());
    let slug = form
        .slug
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| item.slug.clone());
    let content = form.content.clone().unwrap_or_else(|| item.content.clone());
    let template_name = form
        .template
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| item.template.clone());

    let mut tpl = match item.owner_user_id {
        Some(owner_id) => db::get_site_template_by_name_for_user(
            &state.pool,
            owner_id,
            &template_name,
        )
        .await
        .ok()
        .flatten(),
        None => db::get_site_template_by_name(&state.pool, &template_name)
            .await
            .ok()
            .flatten(),
    };
    if tpl.is_none() {
        tpl = match item.owner_user_id {
            Some(owner_id) => db::get_site_template_by_name_for_user(
                &state.pool,
                owner_id,
                "default",
            )
            .await
            .ok()
            .flatten(),
            None => db::get_site_template_by_name(&state.pool, "default")
                .await
                .ok()
                .flatten(),
        };
    }

    let html = match tpl {
        Some(tpl) => apply_site_template(&tpl.html, &title, &content, &slug, &item.kind),
        None => apply_site_template(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{title}}</title></head><body><h1>{{title}}</h1>{{content}}</body></html>",
            &title,
            &content,
            &slug,
            &item.kind,
        ),
    };

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(iframe_srcdoc(&html))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login_form)
        .service(login_submit)
        .service(register_form)
        .service(register_submit)
        .service(logout)
        .service(public_index)
        .service(public_post)
        .service(public_page)
        .service(admin_dashboard)
        .service(admin_new)
        .service(admin_create)
        .service(admin_edit)
        .service(admin_update)
        .service(admin_publish)
        .service(admin_autosave)
        .service(admin_preview)
        .service(admin_templates_list)
        .service(admin_template_new)
        .service(admin_template_create)
        .service(admin_template_edit)
        .service(admin_template_update);
}
