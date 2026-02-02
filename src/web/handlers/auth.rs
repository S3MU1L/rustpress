use actix_web::cookie::{Cookie, SameSite};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use rustpress::models::User;
use rustpress::services::PasswordManager;

use crate::web::forms::{AuthQuery, LoginForm, RegisterForm};
use crate::web::helpers::{is_htmx, render};
use crate::web::state::AppState;
use crate::web::templates::{AdminLoginTemplate, AdminRegisterTemplate};

#[get("/login")]
pub async fn login_form(query: web::Query<AuthQuery>) -> impl Responder {
    render(AdminLoginTemplate {
        error: query.error.clone(),
    })
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

    let user = sqlx::query_as::<_, User>(r#"SELECT * FROM users WHERE email = $1"#)
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login_form)
        .service(login_submit)
        .service(register_form)
        .service(register_submit)
        .service(logout);
}
