use actix_web::cookie::{Cookie, SameSite};
use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};
use std::time::Duration;

use rustpress::db;
use rustpress::models::{RoleName, SiteCreate, User};
use rustpress::services::PasswordManager;

use crate::web::forms::{AuthQuery, LoginForm, RegisterForm};
use crate::web::helpers::{is_htmx, render};
use crate::web::state::AppState;
use crate::web::templates::{
    AdminLoginTemplate, AdminRegisterTemplate,
};

#[get("/admin/login")]
pub async fn login_form(
    query: web::Query<AuthQuery>,
) -> impl Responder {
    let error = query.error.as_deref().map(|code| match code {
        "missing" => "Email and password are required".to_string(),
        "invalid" => "Invalid email or password".to_string(),
        "db" => "Database error. Please try again.".to_string(),
        "internal" => "An internal error occurred. Please try again."
            .to_string(),
        other => other.to_string(),
    });

    render(AdminLoginTemplate { error })
}

#[post("/admin/login")]
pub async fn login_submit(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<LoginForm>,
) -> impl Responder {
    // Rate limiting
    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    if !state.rate_limiter.check_rate_limit(
        &format!("login:{}", client_ip),
        5,                        // 5 attempts
        Duration::from_secs(300), // per 5 minutes
    ) {
        return HttpResponse::TooManyRequests()
            .insert_header((
                "Location",
                "/admin/login?error=rate_limit",
            ))
            .finish();
    }

    let email = form.email.trim().to_string();
    let password = form.password.to_string();

    // Fetch user
    let user = sqlx::query_as::<_, User>(
        r#"SELECT * FROM users WHERE email = $1"#,
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await;

    // Constant-time response: always verify password even if user doesn't exist
    let (user_exists, stored_hash) = match user {
        Ok(Some(u)) => (true, u.password_hash.clone()),
        Ok(None) => {
            // Use a dummy hash with same parameters as PasswordManager
            // to prevent timing side-channels
            let dummy_hash = PasswordManager::hash_password("dummy_password_for_timing")
                .unwrap_or_else(|e| {
                    log::error!("Failed to generate dummy hash: {}", e);
                    // Fallback to hardcoded hash
                    "$argon2id$v=19$m=65536,t=3,p=4$dW5rbm93bl9zYWx0X2R1bW15$E2LvWPx3FxvDaJxEMpLLBfWbLkPXfYHrF8z9CGCX3eI".to_string()
                });
            (false, dummy_hash)
        }
        Err(e) => {
            log::error!("Database error during login: {}", e);
            return HttpResponse::SeeOther()
                .insert_header((
                    "Location",
                    "/admin/login?error=internal",
                ))
                .finish();
        }
    };

    // Always perform password verification
    let password_valid =
        PasswordManager::verify_password(&password, &stored_hash)
            .unwrap_or(false);

    // Only succeed if both user exists and password is valid
    if !user_exists || !password_valid {
        return HttpResponse::SeeOther()
            .insert_header(("Location", "/admin/login?error=invalid"))
            .finish();
    }

    // Re-fetch user (we know it exists now)
    let user = sqlx::query_as::<_, User>(
        r#"SELECT * FROM users WHERE email = $1"#,
    )
    .bind(&email)
    .fetch_one(&state.pool)
    .await
    .expect("User should exist");

    let cookie = Cookie::build("rp_uid", user.id.to_string())
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::days(7))
        .finish();

    HttpResponse::SeeOther()
        .cookie(cookie)
        .insert_header(("Location", "/admin"))
        .finish()
}

#[get("/admin/register")]
pub async fn register_form(
    query: web::Query<AuthQuery>,
) -> impl Responder {
    let error = query.error.as_deref().map(|code| match code {
        "missing" => {
            "Email and password (min 4 characters) are required"
                .to_string()
        }
        "exists" => {
            "An account with this email already exists".to_string()
        }
        "rate_limit" => {
            "Too many registration attempts. Please try again later."
                .to_string()
        }
        "db" => "Database error. Please try again.".to_string(),
        "internal" => "An internal error occurred. Please try again."
            .to_string(),
        other => other.to_string(),
    });

    render(AdminRegisterTemplate { error })
}

#[post("/admin/register")]
pub async fn register_submit(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<RegisterForm>,
) -> impl Responder {
    // Validate form first (before password hashing)
    if let Err(e) = form.validate() {
        return HttpResponse::SeeOther()
            .insert_header((
                "Location",
                format!(
                    "/admin/register?error={}",
                    urlencoding::encode(e)
                ),
            ))
            .finish();
    }

    // Rate limiting
    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();

    if !state.rate_limiter.check_rate_limit(
        &format!("register:{}", client_ip),
        3,                         // 3 attempts
        Duration::from_secs(3600), // per hour
    ) {
        return HttpResponse::TooManyRequests()
            .insert_header((
                "Location",
                "/admin/register?error=rate_limit",
            ))
            .finish();
    }

    let email = form.email.trim().to_string();
    let password = form.password.to_string();

    // Hash password (validation already done by form.validate())
    let password_hash =
        match PasswordManager::hash_password(&password) {
            Ok(h) => h,
            Err(e) => {
                log::error!("Password hashing error: {}", e);
                return HttpResponse::SeeOther()
                    .insert_header((
                        "Location",
                        "/admin/register?error=internal",
                    ))
                    .finish();
            }
        };

    let user =
        match db::create_user(&state.pool, &email, &password_hash)
            .await
        {
            Ok(Some(u)) => u,
            Ok(None) => {
                return HttpResponse::SeeOther()
                    .insert_header((
                        "Location",
                        "/admin/register?error=exists",
                    ))
                    .finish();
            }
            Err(e) => {
                log::error!(
                    "Database error during registration: {}",
                    e
                );
                return HttpResponse::SeeOther()
                    .insert_header((
                        "Location",
                        "/admin/register?error=internal",
                    ))
                    .finish();
            }
        };

    // Set role: first user gets admin, subsequent users get editor.
    let user_count = db::count_users(&state.pool).await.unwrap_or(1);
    let role = if user_count <= 1 {
        RoleName::Admin
    } else {
        RoleName::Editor
    };
    if let Err(e) =
        db::set_user_role(&state.pool, user.id, role).await
    {
        log::error!("Failed to set user role: {}", e);
    }

    // Create default site if none exists
    if db::get_default_site(&state.pool)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        let site_data = SiteCreate {
            owner_user_id: user.id,
            name: "RustPress".to_string(),
            slug: "default".to_string(),
            default_template: "default".to_string(),
        };

        if let Ok(site) =
            db::create_site(&state.pool, &site_data).await
        {
            if let Err(e) =
                db::publish_site(&state.pool, site.id, user.id).await
            {
                log::error!("Failed to publish default site: {}", e);
            }
        }
    }

    let cookie = Cookie::build("rp_uid", user.id.to_string())
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::days(7))
        .finish();

    HttpResponse::SeeOther()
        .cookie(cookie)
        .insert_header(("Location", "/admin"))
        .finish()
}

#[post("/admin/logout")]
pub async fn logout(req: HttpRequest) -> impl Responder {
    let mut cookie = Cookie::build("rp_uid", "")
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .finish();
    cookie.make_removal();

    if is_htmx(&req) {
        HttpResponse::Ok()
            .cookie(cookie)
            .insert_header(("HX-Redirect", "/admin/login"))
            .finish()
    } else {
        HttpResponse::SeeOther()
            .cookie(cookie)
            .insert_header(("Location", "/admin/login"))
            .finish()
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login_form)
        .service(login_submit)
        .service(register_form)
        .service(register_submit)
        .service(logout);
}
