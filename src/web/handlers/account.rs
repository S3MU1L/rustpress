use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;

use rustpress::models::User;
use rustpress::services::PasswordManager;

use crate::web::forms::{AccountEmailForm, ChangePasswordForm};
use crate::web::helpers::{load_user, render, require_user};
use crate::web::state::AppState;
use crate::web::templates::{MeAccountTemplate, MeSecurityTemplate};

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
            let user = load_user(&state.pool, uid).await.unwrap_or_else(|_| User {
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(me_account)
        .service(me_account_update_email)
        .service(me_security)
        .service(me_security_change_password)
        .service(me_security_mark_email_verified);
}
