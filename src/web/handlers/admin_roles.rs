use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use serde::Serialize;
use uuid::Uuid;

use crate::web::helpers::require_user;
use crate::web::state::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RoleRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserRoleRow {
    pub name: String,
}

#[get("/admin/roles")]
pub async fn admin_roles_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let _uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let roles = sqlx::query_as::<_, RoleRow>(
        r#"
        SELECT id, name, description
        FROM roles
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&state.pool)
    .await;

    match roles {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[get("/admin/me/roles")]
pub async fn admin_my_roles(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let roles = sqlx::query_as::<_, UserRoleRow>(
        r#"
        SELECT r.name
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id
        WHERE ur.user_id = $1
        ORDER BY r.name ASC
        "#,
    )
    .bind(uid)
    .fetch_all(&state.pool)
    .await;

    match roles {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(admin_roles_list).service(admin_my_roles);
}
