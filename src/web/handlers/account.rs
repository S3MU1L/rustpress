use actix_web::cookie::{Cookie, SameSite};
use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};
use chrono::Utc;

use rustpress::db;
use rustpress::models::User;
use rustpress::services::PasswordManager;

use crate::web::forms::{
    AccountEmailForm, ChangePasswordForm, DeleteAccountForm,
};
use crate::web::helpers::{
    get_is_admin, is_htmx, is_unique_violation, load_user, render,
    require_user,
};
use crate::web::security::{
    PasswordValidator, generic_error_message, validate_email,
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

    if !validate_email(&new_email) {
        let user = match load_user(&state.pool, uid).await {
            Ok(u) => u,
            Err(resp) => return resp,
        };
        return render_account(
            user,
            is_admin,
            Some("Invalid email address".into()),
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
                generic_error_message("email update")
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

    // Validate before any processing
    if let Err(e) = form.validate() {
        let user = match load_user(&state.pool, uid).await {
            Ok(u) => u,
            Err(resp) => return resp,
        };
        let is_admin = get_is_admin(&req);
        return render_account(
            user,
            is_admin,
            Some(e.to_string()),
            None,
        );
    }

    let user = match load_user(&state.pool, uid).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);

    let ok = match PasswordManager::verify_password(
        &form.current_password,
        &user.password_hash,
    ) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Password verification error: {}", e);
            return render_account(
                user,
                is_admin,
                Some(generic_error_message("password verification")),
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
                log::error!("Password hashing error: {}", e);
                return render_account(
                    user,
                    is_admin,
                    Some(generic_error_message("password hashing")),
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
        Err(e) => {
            log::error!("Password update failed: {}", e);
            render_account(
                user,
                is_admin,
                Some(generic_error_message("password update")),
                None,
            )
        }
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

    // Validate new password strength
    if let Err(msg) = PasswordValidator::validate(&form.new_password)
    {
        return render(MeSecurityTemplate {
            password_set: !user.password_hash.trim().is_empty(),
            email_verified: user.email_verified_at.is_some(),
            error: Some(msg),
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
            log::error!("Password verification error: {}", e);
            return render(MeSecurityTemplate {
                password_set: !user.password_hash.trim().is_empty(),
                email_verified: user.email_verified_at.is_some(),
                error: Some(generic_error_message(
                    "password verification",
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
                log::error!("Password hashing error: {}", e);
                return render(MeSecurityTemplate {
                    password_set: !user
                        .password_hash
                        .trim()
                        .is_empty(),
                    email_verified: user.email_verified_at.is_some(),
                    error: Some(generic_error_message(
                        "password hashing",
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
        Err(e) => {
            log::error!("Password update failed: {}", e);
            render(MeSecurityTemplate {
                password_set: !user.password_hash.trim().is_empty(),
                email_verified: user.email_verified_at.is_some(),
                error: Some(generic_error_message("password update")),
                success: None,
                is_admin,
            })
        }
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

#[post("/admin/me/account/delete")]
pub async fn me_account_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<DeleteAccountForm>,
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

    let ok = match PasswordManager::verify_password(
        &form.password,
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
            Some("Password is incorrect".into()),
            None,
        );
    }

    if is_admin
        && db::count_admins(&state.pool).await.unwrap_or(0) <= 1
    {
        return render_account(
            user,
            is_admin,
            Some("Cannot delete the last admin account".into()),
            None,
        );
    }

    if let Err(e) = db::soft_delete_user(&state.pool, uid).await {
        return render_account(
            user,
            is_admin,
            Some(format!("Failed to delete account: {e}")),
            None,
        );
    }

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
    cfg.service(me_account)
        .service(me_account_update_email)
        .service(me_account_change_password)
        .service(me_account_delete)
        .service(me_security)
        .service(me_security_change_password)
        .service(me_security_mark_email_verified);
}
