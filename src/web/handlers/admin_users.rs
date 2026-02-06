use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};
use sqlx::PgPool;
use uuid::Uuid;

use rustpress::db;
use rustpress::models::User;
use rustpress::services::PasswordManager;

use super::super::forms::{AdminCreateUserForm, AdminUpdateUserForm};
use super::super::helpers::{
    get_is_admin, load_user, render, require_user,
};
use super::super::security::{
    PasswordValidator, generic_error_message, validate_email,
};
use super::super::state::AppState;
use super::super::templates::{
    AdminUserEditTemplate, AdminUserNewTemplate,
    AdminUsersListTemplate,
};

async fn render_edit(
    pool: &PgPool,
    user: User,
    is_admin: bool,
    error: Option<String>,
    success: Option<String>,
) -> HttpResponse {
    let target_roles = db::get_user_role_names(pool, user.id)
        .await
        .unwrap_or_default();
    render(AdminUserEditTemplate {
        target_user: user,
        target_roles,
        is_admin,
        error,
        success,
    })
}

async fn render_list(
    pool: &PgPool,
    current_user_id: Uuid,
    is_admin: bool,
    error: Option<String>,
    success: Option<String>,
) -> HttpResponse {
    let users = db::list_all_users_with_roles(pool)
        .await
        .unwrap_or_default();
    render(AdminUsersListTemplate {
        users,
        current_user_id,
        is_admin,
        error,
        success,
    })
}

#[get("/admin/users")]
pub async fn users_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };
    render_list(&state.pool, uid, get_is_admin(&req), None, None)
        .await
}

#[get("/admin/users/new")]
pub async fn users_new(req: HttpRequest) -> impl Responder {
    if let Err(resp) = require_user(&req) {
        return resp;
    }
    render(AdminUserNewTemplate {
        is_admin: get_is_admin(&req),
        error: None,
    })
}

#[post("/admin/users")]
pub async fn users_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<AdminCreateUserForm>,
) -> impl Responder {
    if let Err(resp) = require_user(&req) {
        return resp;
    }

    let is_admin = get_is_admin(&req);

    // Validate form before password hashing
    if let Err(e) = form.validate() {
        return render(AdminUserNewTemplate {
            is_admin,
            error: Some(e.to_string()),
        });
    }

    let email = form.email.trim().to_string();

    let hash = match PasswordManager::hash_password(&form.password) {
        Ok(h) => h,
        Err(e) => {
            log::error!("Password hashing error: {}", e);
            return render(AdminUserNewTemplate {
                is_admin,
                error: Some(generic_error_message("user creation")),
            });
        }
    };

    let user = match db::create_user(&state.pool, &email, &hash).await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return render(AdminUserNewTemplate {
                is_admin,
                error: Some(
                    "A user with this email already exists".into(),
                ),
            });
        }
        Err(e) => {
            log::error!("Database error creating user: {}", e);
            return render(AdminUserNewTemplate {
                is_admin,
                error: Some(generic_error_message("user creation")),
            });
        }
    };

    if let Err(e) =
        db::set_user_role(&state.pool, user.id, form.role).await
    {
        log::error!("Failed to set user role: {}", e);
    }

    HttpResponse::SeeOther()
        .insert_header(("Location", "/admin/users"))
        .finish()
}

#[get("/admin/users/{id}/edit")]
pub async fn users_edit(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(resp) = require_user(&req) {
        return resp;
    }
    let target_user =
        match load_user(&state.pool, path.into_inner()).await {
            Ok(u) => u,
            Err(resp) => return resp,
        };
    render_edit(
        &state.pool,
        target_user,
        get_is_admin(&req),
        None,
        None,
    )
    .await
}

#[post("/admin/users/{id}/edit")]
pub async fn users_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AdminUpdateUserForm>,
) -> impl Responder {
    if let Err(resp) = require_user(&req) {
        return resp;
    }

    let is_admin = get_is_admin(&req);
    let target_id = path.into_inner();
    let target_user = match load_user(&state.pool, target_id).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let email = form.email.trim().to_string();
    if !validate_email(&email) {
        return render_edit(
            &state.pool,
            target_user,
            is_admin,
            Some("Invalid email address".into()),
            None,
        )
        .await;
    }

    if let Err(e) =
        db::update_user_email(&state.pool, target_id, &email).await
    {
        log::error!("Failed to update user email: {}", e);
        return render_edit(
            &state.pool,
            target_user,
            is_admin,
            Some(generic_error_message("email update")),
            None,
        )
        .await;
    }

    let _ =
        db::set_user_role(&state.pool, target_id, form.role).await;

    // Optional password change
    if let Some(pw) = &form.new_password {
        let pw = pw.trim();
        if !pw.is_empty() {
            if let Err(msg) = PasswordValidator::validate(pw) {
                let user = load_user(&state.pool, target_id)
                    .await
                    .unwrap_or(target_user);
                return render_edit(
                    &state.pool,
                    user,
                    is_admin,
                    Some(msg),
                    None,
                )
                .await;
            }
            match PasswordManager::hash_password(pw) {
                Ok(hash) => {
                    if let Err(e) = db::update_user_password(
                        &state.pool,
                        target_id,
                        &hash,
                    )
                    .await
                    {
                        log::error!("Password update failed: {}", e);
                        let user = load_user(&state.pool, target_id)
                            .await
                            .unwrap_or(target_user);
                        return render_edit(
                            &state.pool,
                            user,
                            is_admin,
                            Some(generic_error_message(
                                "password update",
                            )),
                            None,
                        )
                        .await;
                    }
                }
                Err(e) => {
                    log::error!("Password hashing error: {}", e);
                    let user = load_user(&state.pool, target_id)
                        .await
                        .unwrap_or(target_user);
                    return render_edit(
                        &state.pool,
                        user,
                        is_admin,
                        Some(generic_error_message(
                            "password hashing",
                        )),
                        None,
                    )
                    .await;
                }
            }
        }
    }

    let updated_user = load_user(&state.pool, target_id)
        .await
        .unwrap_or(target_user);
    render_edit(
        &state.pool,
        updated_user,
        is_admin,
        None,
        Some("User updated".into()),
    )
    .await
}

#[post("/admin/users/{id}/delete")]
pub async fn users_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    let target_id = path.into_inner();

    if target_id == uid {
        return render_list(
            &state.pool,
            uid,
            is_admin,
            Some("You cannot delete your own account".into()),
            None,
        )
        .await;
    }

    let target_roles =
        db::get_user_role_names(&state.pool, target_id)
            .await
            .unwrap_or_default();
    if target_roles.contains(&"admin".to_string())
        && db::count_admins(&state.pool).await.unwrap_or(0) <= 1
    {
        return render_list(
            &state.pool,
            uid,
            is_admin,
            Some("Cannot delete the last admin".into()),
            None,
        )
        .await;
    }

    if let Err(e) = db::soft_delete_user(&state.pool, target_id).await
    {
        log::error!("Failed to delete user: {}", e);
        return render_list(
            &state.pool,
            uid,
            is_admin,
            Some(generic_error_message("user deletion")),
            None,
        )
        .await;
    }

    render_list(
        &state.pool,
        uid,
        is_admin,
        None,
        Some("User deleted".into()),
    )
    .await
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(users_list)
        .service(users_new)
        .service(users_create)
        .service(users_edit)
        .service(users_update)
        .service(users_delete);
}
