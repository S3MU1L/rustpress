use actix_web::{HttpRequest, HttpResponse, Responder, get, web};

use rustpress::db;

use crate::web::helpers::require_user;
use crate::web::state::AppState;

#[get("/admin/roles")]
pub async fn admin_roles_list(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let _uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    match db::list_roles(&state.pool).await {
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

    match db::get_user_role_names(&state.pool, uid).await {
        Ok(names) => HttpResponse::Ok().json(names),
        Err(e) => {
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(admin_roles_list).service(admin_my_roles);
}
