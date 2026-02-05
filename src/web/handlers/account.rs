use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};
use chrono::Utc;

use rustpress::db;
use rustpress::models::User;
use rustpress::services::PasswordManager;

use crate::web::forms::{AccountEmailForm, ChangePasswordForm};
use crate::web::helpers::{
    get_is_admin, is_unique_violation, load_user, render,
    require_user,
};
use crate::web::state::AppState;
use crate::web::templates::{MeAccountTemplate, MeSecurityTemplate};

fn render_account(
    user: User,
    is_admin: bool,
    error: Option<String>,
    success: Option<String>,
) -> HttpResponse {
    render(MeAccountTemplate {
        user,
        error,
        success,
        is_admin,
    })
}

#[get("/admin/me/account")]
pub async fn me_account(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };
    let user = match load_user(&state.pool, uid).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    render_account(user, get_is_admin(&req), None, None)
}

#[post("/admin/me/account/email")]
pub async fn me_account_update_email(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<AccountEmailForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    let new_email = form.email.trim().to_string();

    if new_email.is_empty() {
        let user = match load_user(&state.pool, uid).await {
            Ok(u) => u,
            Err(resp) => return resp,
        };
        return render_account(
            user,
            is_admin,
            Some("Email is required".into()),
            None,
        );
    }

    match db::update_user_email(&state.pool, uid, &new_email).await {
        Ok(()) => {
            let user = match load_user(&state.pool, uid).await {
                Ok(u) => u,
                Err(resp) => return resp,
            };
            render_account(
                user,
                is_admin,
                None,
                Some("Email updated".into()),
            )
        }
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
            let msg = if is_unique_violation(&e) {
                "A user with this email already exists".into()
            } else {
                format!("Update failed: {e}")
            };
            render_account(user, is_admin, Some(msg), None)
        }
    }
}

#[post("/admin/me/account/password")]
pub async fn me_account_change_password(
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

    let is_admin = get_is_admin(&req);

    if form.new_password.trim().len() < 4 {
        return render_account(
            user,
            is_admin,
            Some("New password must be at least 4 characters".into()),
            None,
        );
    }

    let ok = match PasswordManager::verify_password(
        &form.current_password,
        &user.password_hash,
    ) {
        Ok(v) => v,
        Err(e) => {
            return render_account(
                user,
                is_admin,
                Some(format!("Password verification error: {e}")),
                None,
            );
        }
    };

    if !ok {
        return render_account(
            user,
            is_admin,
            Some("Current password is incorrect".into()),
            None,
        );
    }

    let new_hash =
        match PasswordManager::hash_password(&form.new_password) {
            Ok(h) => h,
            Err(e) => {
                return render_account(
                    user,
                    is_admin,
                    Some(format!("Password hashing error: {e}")),
                    None,
                );
            }
        };

    match db::update_user_password(&state.pool, uid, &new_hash).await
    {
        Ok(()) => {
            let user =
                load_user(&state.pool, uid).await.unwrap_or(user);
            render_account(
                user,
                is_admin,
                None,
                Some("Password updated".into()),
            )
        }
        Err(e) => render_account(
            user,
            is_admin,
            Some(format!("Password update failed: {e}")),
            None,
        ),
    }
}

#[get("/admin/me/security")]
pub async fn me_security(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let user = match load_user(&state.pool, uid).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    render(MeSecurityTemplate {
        password_set: !user.password_hash.trim().is_empty(),
        email_verified: user.email_verified_at.is_some(),
        error: None,
        success: None,
        is_admin,
    })
}

#[post("/admin/me/security/password")]
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

    let is_admin = get_is_admin(&req);

    if form.new_password.trim().len() < 4 {
        return render(MeSecurityTemplate {
            password_set: !user.password_hash.trim().is_empty(),
            email_verified: user.email_verified_at.is_some(),
            error: Some(
                "New password must be at least 4 characters"
                    .to_string(),
            ),
            success: None,
            is_admin,
        });
    }

    let ok = match PasswordManager::verify_password(
        &form.current_password,
        &user.password_hash,
    ) {
        Ok(v) => v,
        Err(e) => {
            return render(MeSecurityTemplate {
                password_set: !user.password_hash.trim().is_empty(),
                email_verified: user.email_verified_at.is_some(),
                error: Some(format!(
                    "Password verification error: {e}"
                )),
                success: None,
                is_admin,
            });
        }
    };

    if !ok {
        return render(MeSecurityTemplate {
            password_set: !user.password_hash.trim().is_empty(),
            email_verified: user.email_verified_at.is_some(),
            error: Some("Current password is incorrect".to_string()),
            success: None,
            is_admin,
        });
    }

    let new_hash =
        match PasswordManager::hash_password(&form.new_password) {
            Ok(h) => h,
            Err(e) => {
                return render(MeSecurityTemplate {
                    password_set: !user
                        .password_hash
                        .trim()
                        .is_empty(),
                    email_verified: user.email_verified_at.is_some(),
                    error: Some(format!(
                        "Password hashing error: {e}"
                    )),
                    success: None,
                    is_admin,
                });
            }
        };

    match db::update_user_password(&state.pool, uid, &new_hash).await
    {
        Ok(()) => {
            let user =
                load_user(&state.pool, uid).await.unwrap_or(user);
            render(MeSecurityTemplate {
                password_set: !user.password_hash.trim().is_empty(),
                email_verified: user.email_verified_at.is_some(),
                error: None,
                success: Some("Password updated".to_string()),
                is_admin,
            })
        }
        Err(e) => render(MeSecurityTemplate {
            password_set: !user.password_hash.trim().is_empty(),
            email_verified: user.email_verified_at.is_some(),
            error: Some(format!("Update failed: {e}")),
            success: None,
            is_admin,
        }),
    }
}

#[post("/admin/me/security/verify-email")]
pub async fn me_security_mark_email_verified(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);

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
            success: Some(
                "Email marked as verified (dev)".to_string(),
            ),
            is_admin,
        }),
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Database error: {e}")),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(me_account)
        .service(me_account_update_email)
        .service(me_account_change_password)
        .service(me_security)
        .service(me_security_change_password)
        .service(me_security_mark_email_verified);
}
