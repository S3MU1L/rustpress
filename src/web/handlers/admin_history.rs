use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

use rustpress::db;

use crate::web::helpers::require_user;
use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct RevisionsQuery {
    pub limit: Option<i64>,
}

#[get("/admin/content/{id}/revisions")]
pub async fn admin_list_revisions(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    query: web::Query<RevisionsQuery>,
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

    // Ensure legacy items have a baseline revision.
    if let Err(e) = db::ensure_initial_revision(&state.pool, id, Some(uid)).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    match db::list_revisions(&state.pool, id, limit).await {
        Ok(revs) => HttpResponse::Ok().json(revs),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/admin/content/{id}/revisions/{rev}/restore")]
pub async fn admin_restore_revision(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(Uuid, i32)>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let (id, rev) = path.into_inner();
    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let can_edit = match db::can_edit_content(&state.pool, &item, uid).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !can_edit {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    match db::restore_revision(&state.pool, id, rev).await {
        Ok(Some(item)) => HttpResponse::Ok().json(item),
        Ok(None) => HttpResponse::NotFound().body("Revision not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/admin/content/{id}/undo")]
pub async fn admin_undo(
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

    let can_edit = match db::can_edit_content(&state.pool, &item, uid).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !can_edit {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    if let Err(e) = db::ensure_initial_revision(&state.pool, id, Some(uid)).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    match db::undo(&state.pool, id).await {
        Ok(Some(item)) => HttpResponse::Ok().json(item),
        Ok(None) => HttpResponse::NotFound().body("Not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/admin/content/{id}/redo")]
pub async fn admin_redo(
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

    let can_edit = match db::can_edit_content(&state.pool, &item, uid).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !can_edit {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    if let Err(e) = db::ensure_initial_revision(&state.pool, id, Some(uid)).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    match db::redo(&state.pool, id).await {
        Ok(Some(item)) => HttpResponse::Ok().json(item),
        Ok(None) => HttpResponse::NotFound().body("Not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(admin_list_revisions)
        .service(admin_restore_revision)
        .service(admin_undo)
        .service(admin_redo);
}
