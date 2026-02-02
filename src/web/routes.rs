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
    AdminDashboardTemplate, AdminEditTemplate, AdminLoginTemplate, AdminNewTemplate,
    AdminRegisterTemplate, AdminTemplateEditTemplate, AdminTemplateNewTemplate,
    AdminTemplatesListTemplate, MeAccountTemplate, MeSecurityTemplate, PublicContentTemplate,
    PublicIndexTemplate, SiteEditTemplate, SiteNewTemplate, SitesListTemplate, ThemesTemplate,
};

const _AUTH_COOKIE: &str = "rp_uid";

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

fn require_user(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    match current_user_id(req) {
        Some(uid) => Ok(uid),
        None => {
            if is_htmx(req) {
                Err(HttpResponse::Unauthorized()
                    .insert_header(("HX-Redirect", "/login"))
                    .finish())
            } else {
                Err(HttpResponse::SeeOther()
                    .insert_header(("Location", "/login"))
                    .finish())
            }
        }
    }
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct AuthQuery {
    pub error: Option<String>,
}

#[get("/login")]
pub async fn login_form(query: web::Query<AuthQuery>) -> impl Responder {
    render(AdminLoginTemplate {
        error: query.error.clone(),
    })
}

#[derive(serde::Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub password: String,
}

#[get("/register")]
pub async fn register_form(query: web::Query<AuthQuery>) -> impl Responder {
    render(AdminRegisterTemplate {
        error: query.error.clone(),
    })
}

#[post("/register")]
pub async fn register_submit(
    state: web::Data<AppState>,
    form: web::Form<RegisterForm>,
) -> impl Responder {
    let email = form.email.trim().to_string();
    let password = form.password.to_string();

    if email.is_empty() || password.len() < 4 {
        return HttpResponse::SeeOther()
            .insert_header(("Location", "/register?error=missing"))
            .finish();
    }

    let password_hash = match PasswordManager::hash_password(&password) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Password hashing error: {e}");
            return HttpResponse::SeeOther()
                .insert_header(("Location", "/register?error=internal"))
                .finish();
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
            return HttpResponse::SeeOther()
                .insert_header(("Location", "/register?error=exists"))
                .finish();
        }
        Err(e) => {
            eprintln!("Database error: {e}");
            return HttpResponse::SeeOther()
                .insert_header(("Location", "/register?error=db"))
                .finish();
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
        return HttpResponse::SeeOther()
            .insert_header(("Location", "/login?error=missing"))
            .finish();
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
            return HttpResponse::SeeOther()
                .insert_header(("Location", "/login?error=invalid"))
                .finish();
        }
        Err(e) => {
            eprintln!("Database error: {e}");
            return HttpResponse::SeeOther()
                .insert_header(("Location", "/login?error=db"))
                .finish();
        }
    };

    let ok = match PasswordManager::verify_password(&password, &user.password_hash) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Password verification error: {e}");
            return HttpResponse::SeeOther()
                .insert_header(("Location", "/login?error=internal"))
                .finish();
        }
    };

    if !ok {
        return HttpResponse::SeeOther()
            .insert_header(("Location", "/login?error=invalid"))
            .finish();
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

async fn load_user(pool: &PgPool, uid: Uuid) -> Result<User, HttpResponse> {
    let user = sqlx::query_as::<_, User>(
        r#"SELECT * FROM users WHERE id = $1"#,
    )
    .bind(uid)
    .fetch_optional(pool)
    .await;

    match user {
        Ok(Some(u)) => Ok(u),
        Ok(None) => Err(HttpResponse::Unauthorized().body("User not found")),
        Err(e) => Err(HttpResponse::InternalServerError().body(format!("Database error: {e}"))),
    }
}

#[get("/me/account")]
pub async fn me_account(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let user = match load_user(&state.pool, uid).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    render(MeAccountTemplate {
        user,
        error: None,
        success: None,
    })
}

#[derive(serde::Deserialize)]
pub struct AccountEmailForm {
    pub email: String,
}

#[post("/me/account/email")]
pub async fn me_account_update_email(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<AccountEmailForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let new_email = form.email.trim().to_string();
    if new_email.is_empty() {
        let user = match load_user(&state.pool, uid).await {
            Ok(u) => u,
            Err(resp) => return resp,
        };
        return render(MeAccountTemplate {
            user,
            error: Some("Email is required".to_string()),
            success: None,
        });
    }

    let updated = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET email = $1, edited_at = now()
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(&new_email)
    .bind(uid)
    .fetch_one(&state.pool)
    .await;

    match updated {
        Ok(user) => render(MeAccountTemplate {
            user,
            error: None,
            success: Some("Email updated".to_string()),
        }),
        Err(e) => {
            let user = load_user(&state.pool, uid)
                .await
                .unwrap_or_else(|_| User {
                    id: uid,
                    email: new_email,
                    password_hash: "".into(),
                    email_verified_at: None,
                    created_at: Utc::now(),
                    edited_at: Utc::now(),
                    deleted_at: None,
                });
            render(MeAccountTemplate {
                user,
                error: Some(format!("Update failed: {e}")),
                success: None,
            })
        }
    }
}

#[get("/me/security")]
pub async fn me_security(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let user = match load_user(&state.pool, uid).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    render(MeSecurityTemplate {
        password_set: !user.password_hash.trim().is_empty(),
        email_verified: user.email_verified_at.is_some(),
        error: None,
        success: None,
    })
}

#[derive(serde::Deserialize)]
pub struct ChangePasswordForm {
    pub current_password: String,
    pub new_password: String,
}

#[post("/me/security/password")]
pub async fn me_security_change_password(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<ChangePasswordForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let user = match load_user(&state.pool, uid).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    if form.new_password.trim().len() < 4 {
        return render(MeSecurityTemplate {
            password_set: !user.password_hash.trim().is_empty(),
            email_verified: user.email_verified_at.is_some(),
            error: Some("New password must be at least 4 characters".to_string()),
            success: None,
        });
    }

    let ok = match PasswordManager::verify_password(&form.current_password, &user.password_hash) {
        Ok(v) => v,
        Err(e) => {
            return render(MeSecurityTemplate {
                password_set: !user.password_hash.trim().is_empty(),
                email_verified: user.email_verified_at.is_some(),
                error: Some(format!("Password verification error: {e}")),
                success: None,
            });
        }
    };

    if !ok {
        return render(MeSecurityTemplate {
            password_set: !user.password_hash.trim().is_empty(),
            email_verified: user.email_verified_at.is_some(),
            error: Some("Current password is incorrect".to_string()),
            success: None,
        });
    }

    let new_hash = match PasswordManager::hash_password(&form.new_password) {
        Ok(h) => h,
        Err(e) => {
            return render(MeSecurityTemplate {
                password_set: !user.password_hash.trim().is_empty(),
                email_verified: user.email_verified_at.is_some(),
                error: Some(format!("Password hashing error: {e}")),
                success: None,
            });
        }
    };

    let updated = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET password_hash = $1, edited_at = now()
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(new_hash)
    .bind(uid)
    .fetch_one(&state.pool)
    .await;

    match updated {
        Ok(user) => render(MeSecurityTemplate {
            password_set: !user.password_hash.trim().is_empty(),
            email_verified: user.email_verified_at.is_some(),
            error: None,
            success: Some("Password updated".to_string()),
        }),
        Err(e) => render(MeSecurityTemplate {
            password_set: !user.password_hash.trim().is_empty(),
            email_verified: user.email_verified_at.is_some(),
            error: Some(format!("Update failed: {e}")),
            success: None,
        }),
    }
}

#[post("/me/security/verify-email")]
pub async fn me_security_mark_email_verified(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let updated = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET email_verified_at = COALESCE(email_verified_at, now()), edited_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(uid)
    .fetch_one(&state.pool)
    .await;

    match updated {
        Ok(user) => render(MeSecurityTemplate {
            password_set: !user.password_hash.trim().is_empty(),
            email_verified: user.email_verified_at.is_some(),
            error: None,
            success: Some("Email marked as verified (dev)".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {e}")),
    }
}

#[derive(serde::Deserialize)]
pub struct SitesQuery {
    pub q: Option<String>,
}

#[get("/sites")]
pub async fn sites_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SitesQuery>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let q = query.q.clone().unwrap_or_default();
    let sites = db::list_sites_for_user(&state.pool, uid, query.q.as_deref())
        .await
        .unwrap_or_default();

    render(SitesListTemplate { sites, query: q })
}

#[get("/sites/new")]
pub async fn sites_new(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();

    render(SiteNewTemplate {
        templates,
        default_template: "default".to_string(),
        error: None,
    })
}

#[derive(serde::Deserialize)]
pub struct SiteCreateForm {
    pub name: String,
    pub slug: String,
    pub default_template: Option<String>,
}

#[post("/sites")]
pub async fn sites_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<SiteCreateForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let name = form.name.trim().to_string();
    let slug = form.slug.trim().to_string();
    if name.is_empty() || slug.is_empty() {
        let templates = db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default();
        return render(SiteNewTemplate {
            templates,
            default_template: form
                .default_template
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            error: Some("Name and slug are required".to_string()),
        });
    }

    let default_template = form
        .default_template
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());

    let created = db::create_site(
        &state.pool,
        &rustpress::models::SiteCreate {
            owner_user_id: uid,
            name,
            slug,
            default_template,
        },
    )
    .await;

    match created {
        Ok(site) => {
            if is_htmx(&req) {
                HttpResponse::Ok()
                    .insert_header(("HX-Redirect", format!("/sites/{}", site.id)))
                    .finish()
            } else {
                HttpResponse::SeeOther()
                    .insert_header(("Location", format!("/sites/{}", site.id)))
                    .finish()
            }
        }
        Err(e) => {
            let templates = db::list_site_templates_for_user(&state.pool, uid)
                .await
                .unwrap_or_default();
            render(SiteNewTemplate {
                templates,
                default_template: form
                    .default_template
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
                error: Some(format!("Create failed: {e}")),
            })
        }
    }
}

#[get("/sites/{id}")]
pub async fn sites_edit(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };
    let id = path.into_inner();

    let site = match db::get_site_by_id(&state.pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if site.owner_user_id != uid {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();

    render(SiteEditTemplate {
        site,
        templates,
        error: None,
        success: None,
    })
}

#[derive(serde::Deserialize)]
pub struct SiteUpdateForm {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub default_template: Option<String>,
}

#[post("/sites/{id}")]
pub async fn sites_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<SiteUpdateForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };
    let id = path.into_inner();

    let existing = match db::get_site_by_id(&state.pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if existing.owner_user_id != uid {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let update = rustpress::models::SiteUpdate {
        name: form.name.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        slug: form.slug.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        status: None,
        default_template: form
            .default_template
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
    };

    let updated = match db::update_site(&state.pool, id, &update).await {
        Ok(Some(s)) => s,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            let templates = db::list_site_templates_for_user(&state.pool, uid)
                .await
                .unwrap_or_default();
            return render(SiteEditTemplate {
                site: existing,
                templates,
                error: Some(format!("Update failed: {e}")),
                success: None,
            });
        }
    };

    let templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();

    render(SiteEditTemplate {
        site: updated,
        templates,
        error: None,
        success: Some("Saved".to_string()),
    })
}

#[post("/sites/{id}/publish")]
pub async fn sites_publish(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };
    let id = path.into_inner();

    let existing = match db::get_site_by_id(&state.pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    if existing.owner_user_id != uid {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let published = match db::publish_site(&state.pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::BadRequest().body(format!("Publish failed: {e}")),
    };

    render(SiteEditTemplate {
        site: published,
        templates: db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default(),
        error: None,
        success: Some("Site published".to_string()),
    })
}

#[derive(serde::Deserialize)]
pub struct ThemesQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub site_id: Option<Uuid>,
}

#[get("/themes")]
pub async fn themes_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ThemesQuery>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let q = query.q.clone().unwrap_or_default();
    let category = query.category.clone().unwrap_or_else(|| "all".to_string());
    let selected_site_id = query.site_id;

    let mut templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();

    let sites = db::list_sites_for_user(&state.pool, uid, None)
        .await
        .unwrap_or_default();

    if !q.trim().is_empty() {
        let needle = q.trim().to_lowercase();
        templates.retain(|t| {
            t.name.to_lowercase().contains(&needle)
                || t.description.to_lowercase().contains(&needle)
        });
    }

    match category.as_str() {
        "builtin" => templates.retain(|t| t.is_builtin),
        "custom" => templates.retain(|t| !t.is_builtin),
        _ => {}
    }

    render(ThemesTemplate {
        templates,
        sites,
        selected_site_id,
        query: q,
        category,
    })
}

#[derive(serde::Deserialize)]
pub struct ApplyThemeForm {
    pub template: String,
}

#[post("/sites/{id}/theme")]
pub async fn sites_apply_theme(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<ApplyThemeForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let existing = match db::get_site_by_id(&state.pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if existing.owner_user_id != uid {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let template_name = form.template.trim();
    if template_name.is_empty() {
        return HttpResponse::BadRequest().body("Template is required");
    }

    // Only allow selecting templates visible to this user.
    let allowed = db::get_site_template_by_name_for_user(&state.pool, uid, template_name)
        .await
        .ok()
        .flatten();
    if allowed.is_none() {
        return HttpResponse::BadRequest().body("Unknown template");
    }

    let update = rustpress::models::SiteUpdate {
        name: None,
        slug: None,
        status: None,
        default_template: Some(template_name.to_string()),
    };

    let updated = match db::update_site(&state.pool, id, &update).await {
        Ok(Some(s)) => s,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::BadRequest().body(format!("Update failed: {e}")),
    };

    if is_htmx(&req) {
        HttpResponse::Ok()
            .insert_header(("HX-Redirect", format!("/sites/{}", updated.id)))
            .finish()
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/sites/{}", updated.id)))
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

fn is_unique_violation(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().as_deref() == Some("23505"),
        _ => false,
    }
}

fn iframe_srcdoc(html: &str) -> String {
    // `srcdoc` is an attribute; escape enough to keep it valid.
    // Browsers will decode entities inside attributes.
    format!(
        r#"<iframe class="preview-iframe" sandbox="allow-same-origin" referrerpolicy="no-referrer" srcdoc="{}"></iframe>"#,
        escape_html(html)
    )
}

fn normalize_builtin_template_html(html: &str) -> std::borrow::Cow<'_, str> {
    if !(html.contains("\\n") || html.contains("\\t")) {
        return std::borrow::Cow::Borrowed(html);
    }

    // Older built-in templates were inserted via migrations using literal "\\n" sequences.
    // Only normalize built-ins (call sites guard on `is_builtin`) to avoid surprising changes
    // in user-authored templates.
    let html = html.replace("\\n", "\n").replace("\\t", "\t");
    std::borrow::Cow::Owned(html)
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
                Some(tpl) => {
                    let tpl_html = if tpl.is_builtin {
                        normalize_builtin_template_html(&tpl.html)
                    } else {
                        std::borrow::Cow::Borrowed(tpl.html.as_str())
                    };
                    apply_site_template(
                        tpl_html.as_ref(),
                        &item.title,
                        &item.content,
                        &item.slug,
                        &item.kind,
                    )
                }
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
                Some(tpl) => {
                    let tpl_html = if tpl.is_builtin {
                        normalize_builtin_template_html(&tpl.html)
                    } else {
                        std::borrow::Cow::Borrowed(tpl.html.as_str())
                    };
                    apply_site_template(
                        tpl_html.as_ref(),
                        &item.title,
                        &item.content,
                        &item.slug,
                        &item.kind,
                    )
                }
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
pub async fn admin_dashboard(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let posts = db::list_content(&state.pool, ContentKind::Post, true)
        .await
        .unwrap_or_default();
    let pages = db::list_content(&state.pool, ContentKind::Page, true)
        .await
        .unwrap_or_default();

    // MVP scoping: show only content owned by this user or legacy NULL-owned content.
    let posts = posts
        .into_iter()
        .filter(|c| c.owner_user_id.is_none() || c.owner_user_id == Some(uid))
        .collect();
    let pages = pages
        .into_iter()
        .filter(|c| c.owner_user_id.is_none() || c.owner_user_id == Some(uid))
        .collect();

    render(AdminDashboardTemplate { posts, pages })
}

#[get("/admin/{kind:posts|pages}/new")]
pub async fn admin_new(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let kind = path.into_inner();

    let templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();

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
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let kind = match path.into_inner().as_str() {
        "posts" => ContentKind::Post,
        "pages" => ContentKind::Page,
        _ => return HttpResponse::NotFound().body("Unknown kind"),
    };

    let data = ContentCreate {
        owner_user_id: Some(uid),
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
            if is_unique_violation(&e) {
                return HttpResponse::Conflict()
                    .content_type("text/plain; charset=utf-8")
                    .body("Slug already exists for this content type".to_string());
            }

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
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if item.owner_user_id.is_some_and(|owner| owner != uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();
    render(AdminEditTemplate { item, templates })
}

#[get("/admin/templates")]
pub async fn admin_templates_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();
    render(AdminTemplatesListTemplate { templates })
}

#[get("/admin/templates/new")]
pub async fn admin_template_new(req: HttpRequest) -> impl Responder {
    if let Err(resp) = require_user(&req) {
        return resp;
    }

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
    let owner_user_id = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
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
            if is_unique_violation(&e) {
                return HttpResponse::Conflict()
                    .content_type("text/plain; charset=utf-8")
                    .body("Template name already exists".to_string());
            }
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
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let template = match db::get_site_template_by_id(&state.pool, id).await {
        Ok(Some(t)) => t,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !template.is_builtin {
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

    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let existing = match db::get_site_template_by_id(&state.pool, id).await {
        Ok(Some(t)) => t,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if existing.is_builtin {
        return HttpResponse::Forbidden().body("Built-in templates are read-only");
    }

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
            if is_unique_violation(&e) {
                return HttpResponse::Conflict()
                    .content_type("text/plain; charset=utf-8")
                    .body("Template name already exists".to_string());
            }
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

#[post("/admin/templates/{id}/duplicate")]
pub async fn admin_template_duplicate(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let template = match db::get_site_template_by_id(&state.pool, id).await {
        Ok(Some(t)) => t,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !template.is_builtin && template.owner_user_id != Some(uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    // Try a few times to avoid rare name collisions.
    let mut last_err: Option<sqlx::Error> = None;
    for _ in 0..3 {
        let suffix = Uuid::new_v4().to_string();
        let short = &suffix[..8];
        let name = format!("{}-copy-{}", template.name, short);

        let data = rustpress::models::SiteTemplateCreate {
            owner_user_id: uid,
            name,
            description: template.description.clone(),
            html: template.html.clone(),
        };

        match db::create_site_template(&state.pool, &data).await {
            Ok(created) => {
                if is_htmx(&req) {
                    return HttpResponse::Ok()
                        .insert_header(("HX-Redirect", format!("/admin/templates/{}", created.id)))
                        .finish();
                }
                return HttpResponse::SeeOther()
                    .insert_header(("Location", format!("/admin/templates/{}", created.id)))
                    .finish();
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    let msg = last_err
        .as_ref()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "Duplicate failed".to_string());
    HttpResponse::BadRequest().body(msg)
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
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();

    // Enforce ownership before mutating.
    let existing = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if existing.owner_user_id.is_some_and(|owner| owner != uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }

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
            if is_unique_violation(&e) {
                return HttpResponse::Conflict()
                    .content_type("text/plain; charset=utf-8")
                    .body("Slug already exists for this content type".to_string());
            }
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Update failed: {e}"));
        }
    };

    if is_htmx(&req) {
        let templates = db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default();
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
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();

    // Enforce ownership before mutating.
    let existing = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if existing.owner_user_id.is_some_and(|owner| owner != uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }

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
        let templates = db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default();
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
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AdminLiveForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();

    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if item.owner_user_id.is_some_and(|owner| owner != uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }

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
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AdminLiveForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if item.owner_user_id.is_some_and(|owner| owner != uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }

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
        Some(tpl) => {
            let tpl_html = if tpl.is_builtin {
                normalize_builtin_template_html(&tpl.html)
            } else {
                std::borrow::Cow::Borrowed(tpl.html.as_str())
            };
            apply_site_template(tpl_html.as_ref(), &title, &content, &slug, &item.kind)
        }
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

#[derive(serde::Deserialize)]
pub struct AdminNewPreviewForm {
    pub kind: String,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub template: Option<String>,
}

#[post("/admin/preview")]
pub async fn admin_preview_new(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<AdminNewPreviewForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let kind = match form.kind.trim() {
        "posts" => "post",
        "pages" => "page",
        other => other,
    };

    let title = form
        .title
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Untitled".to_string());
    let slug = form
        .slug
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "untitled".to_string());
    let content = form.content.clone().unwrap_or_default();
    let template_name = form
        .template
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());

    let mut tpl = db::get_site_template_by_name_for_user(&state.pool, uid, &template_name)
        .await
        .ok()
        .flatten();
    if tpl.is_none() {
        tpl = db::get_site_template_by_name_for_user(&state.pool, uid, "default")
            .await
            .ok()
            .flatten();
    }

    let html = match tpl {
        Some(tpl) => {
            let tpl_html = if tpl.is_builtin {
                normalize_builtin_template_html(&tpl.html)
            } else {
                std::borrow::Cow::Borrowed(tpl.html.as_str())
            };
            apply_site_template(tpl_html.as_ref(), &title, &content, &slug, kind)
        }
        None => apply_site_template(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{title}}</title></head><body><h1>{{title}}</h1>{{content}}</body></html>",
            &title,
            &content,
            &slug,
            kind,
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
        .service(me_account)
        .service(me_account_update_email)
        .service(me_security)
        .service(me_security_change_password)
        .service(me_security_mark_email_verified)
        .service(sites_list)
        .service(sites_new)
        .service(sites_create)
        .service(sites_edit)
        .service(sites_update)
        .service(sites_publish)
        .service(themes_list)
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
        .service(admin_preview_new)
        .service(admin_templates_list)
        .service(admin_template_new)
        .service(admin_template_create)
        .service(admin_template_edit)
        .service(admin_template_update)
        .service(admin_template_duplicate)
        .service(sites_apply_theme);
}
