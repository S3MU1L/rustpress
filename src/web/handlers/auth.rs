use actix_web::cookie::{Cookie, SameSite};
use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};

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
    form: web::Form<LoginForm>,
) -> impl Responder {
    let email = form.email.trim().to_string();
    let password = form.password.to_string();

    if email.is_empty() || password.is_empty() {
        return HttpResponse::SeeOther()
            .insert_header(("Location", "/admin/login?error=missing"))
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
                .insert_header((
                    "Location",
                    "/admin/login?error=invalid",
                ))
                .finish();
        }
        Err(e) => {
            eprintln!("Database error: {e}");
            return HttpResponse::SeeOther()
                .insert_header(("Location", "/admin/login?error=db"))
                .finish();
        }
    };

    let ok = match PasswordManager::verify_password(
        &password,
        &user.password_hash,
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Password verification error: {e}");
            return HttpResponse::SeeOther()
                .insert_header((
                    "Location",
                    "/admin/login?error=internal",
                ))
                .finish();
        }
    };

    if !ok {
        return HttpResponse::SeeOther()
            .insert_header(("Location", "/admin/login?error=invalid"))
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
    form: web::Form<RegisterForm>,
) -> impl Responder {
    let email = form.email.trim().to_string();
    let password = form.password.to_string();

    if email.is_empty() || password.len() < 4 {
        return HttpResponse::SeeOther()
            .insert_header((
                "Location",
                "/admin/register?error=missing",
            ))
            .finish();
    }

    let password_hash =
        match PasswordManager::hash_password(&password) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Password hashing error: {e}");
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
                eprintln!("Database error: {e}");
                return HttpResponse::SeeOther()
                    .insert_header((
                        "Location",
                        "/admin/register?error=db",
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
        eprintln!("Failed to set user role: {e}");
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
                eprintln!("Failed to publish default site: {e}");
            }
        }
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

#[post("/admin/logout")]
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
