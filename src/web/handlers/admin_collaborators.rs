use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

use rustpress::db;

use crate::web::helpers::require_user;
use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct AddCollaboratorForm {
    pub email: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct SetRoleForm {
    pub role: String,
}

fn normalize_role(role: &str) -> Option<&'static str> {
    match role.trim().to_lowercase().as_str() {
        "viewer" => Some("viewer"),
        "editor" => Some("editor"),
        _ => None,
    }
}

#[get("/admin/content/{id}/collaborators")]
pub async fn admin_list_collaborators(
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

    let can_view = match db::can_view_content(&state.pool, &item, uid).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !can_view {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    match db::list_collaborators(&state.pool, id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/admin/content/{id}/collaborators")]
pub async fn admin_add_collaborator(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AddCollaboratorForm>,
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

    let can_manage = match db::can_manage_collaborators(&state.pool, &item, uid).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !can_manage {
        return HttpResponse::Forbidden().body("Only the owner can manage collaborators");
    }

    let role = match normalize_role(&form.role) {
        Some(r) => r,
        None => return HttpResponse::BadRequest().body("Invalid role (use viewer|editor)"),
    };

    let email = form.email.trim().to_string();
    if email.is_empty() {
        return HttpResponse::BadRequest().body("Email required");
    }

    match db::add_collaborator(&state.pool, id, &email, role, Some(uid)).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/admin/content/{id}/collaborators/{user_id}")]
pub async fn admin_set_collaborator_role(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
    form: web::Form<SetRoleForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let (id, user_id) = path.into_inner();

    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let can_manage = match db::can_manage_collaborators(&state.pool, &item, uid).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !can_manage {
        return HttpResponse::Forbidden().body("Only the owner can manage collaborators");
    }

    let role = match normalize_role(&form.role) {
        Some(r) => r,
        None => return HttpResponse::BadRequest().body("Invalid role (use viewer|editor)"),
    };

    match db::set_collaborator_role(&state.pool, id, user_id, role).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/admin/content/{id}/collaborators/{user_id}/remove")]
pub async fn admin_remove_collaborator(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let (id, user_id) = path.into_inner();

    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let can_manage = match db::can_manage_collaborators(&state.pool, &item, uid).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !can_manage {
        return HttpResponse::Forbidden().body("Only the owner can manage collaborators");
    }

    match db::remove_collaborator(&state.pool, id, user_id).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(admin_list_collaborators)
        .service(admin_add_collaborator)
        .service(admin_set_collaborator_role)
        .service(admin_remove_collaborator);
}
