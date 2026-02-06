use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use rustpress::db;
use rustpress::models::ContentItem;

use crate::web::helpers::{
    is_htmx, render, render_not_found, require_user,
};
use crate::web::state::AppState;
use crate::web::templates::AdminHistoryPartialTemplate;

/// Auth + load + `can_view_content` gate.
async fn load_viewable(
    pool: &PgPool,
    req: &HttpRequest,
    id: Uuid,
) -> Result<(Uuid, ContentItem), HttpResponse> {
    let uid = require_user(req)?;
    let item = db::get_content_by_id(pool, id)
        .await
        .map_err(|e| {
            HttpResponse::InternalServerError().body(e.to_string())
        })?
        .ok_or_else(|| render_not_found(req))?;

    let ok = db::can_view_content(pool, &item, uid).await.map_err(
        |e| HttpResponse::InternalServerError().body(e.to_string()),
    )?;
    if !ok {
        return Err(HttpResponse::Forbidden().body("Forbidden"));
    }
    Ok((uid, item))
}

/// Auth + load + `can_edit_content` gate.
async fn load_editable(
    pool: &PgPool,
    req: &HttpRequest,
    id: Uuid,
) -> Result<(Uuid, ContentItem), HttpResponse> {
    let uid = require_user(req)?;
    let item = db::get_content_by_id(pool, id)
        .await
        .map_err(|e| {
            HttpResponse::InternalServerError().body(e.to_string())
        })?
        .ok_or_else(|| render_not_found(req))?;

    let ok = db::can_edit_content(pool, &item, uid).await.map_err(
        |e| HttpResponse::InternalServerError().body(e.to_string()),
    )?;
    if !ok {
        return Err(HttpResponse::Forbidden().body("Forbidden"));
    }
    Ok((uid, item))
}

fn internal_server_error(e: impl std::fmt::Display) -> HttpResponse {
    HttpResponse::InternalServerError().body(e.to_string())
}

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
    let id = path.into_inner();
    let (uid, _item) =
        match load_viewable(&state.pool, &req, id).await {
            Ok(v) => v,
            Err(r) => return r,
        };

    if let Err(e) =
        db::ensure_initial_revision(&state.pool, id, Some(uid)).await
    {
        return internal_server_error(e);
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    match db::list_revisions(&state.pool, id, limit).await {
        Ok(revs) => HttpResponse::Ok().json(revs),
        Err(e) => internal_server_error(e),
    }
}

#[get("/admin/content/{id}/revisions/{rev}")]
pub async fn admin_get_revision(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(Uuid, i32)>,
) -> impl Responder {
    let (id, rev) = path.into_inner();
    if let Err(r) = load_viewable(&state.pool, &req, id).await {
        return r;
    }

    match db::get_revision(&state.pool, id, rev).await {
        Ok(Some(revision)) => HttpResponse::Ok().json(revision),
        Ok(None) => render_not_found(&req),
        Err(e) => internal_server_error(e),
    }
}

#[get("/admin/content/{id}/history")]
pub async fn admin_history_panel(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let (uid, item) = match load_viewable(&state.pool, &req, id).await
    {
        Ok(v) => v,
        Err(r) => return r,
    };

    if let Err(e) =
        db::ensure_initial_revision(&state.pool, id, Some(uid)).await
    {
        return internal_server_error(e);
    }

    let revisions =
        match db::list_revisions(&state.pool, id, 50).await {
            Ok(revs) => revs,
            Err(e) => return internal_server_error(e),
        };

    let user_ids: Vec<Uuid> = revisions
        .iter()
        .filter_map(|r| r.created_by_user_id)
        .collect();

    let authors =
        match db::get_user_email_map(&state.pool, &user_ids).await {
            Ok(map) => map,
            Err(e) => return internal_server_error(e),
        };

    render(AdminHistoryPartialTemplate {
        revisions,
        authors,
        current_rev: item.current_rev,
        content_item_id: id,
    })
}

#[post("/admin/content/{id}/revisions/{rev}/restore")]
pub async fn admin_restore_revision(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(Uuid, i32)>,
) -> impl Responder {
    let (id, rev) = path.into_inner();
    if let Err(r) = load_editable(&state.pool, &req, id).await {
        return r;
    }

    match db::restore_revision(&state.pool, id, rev).await {
        Ok(Some(restored)) => {
            if is_htmx(&req) {
                HttpResponse::Ok()
                    .insert_header((
                        "HX-Redirect",
                        format!("/admin/edit/{id}"),
                    ))
                    .finish()
            } else {
                HttpResponse::Ok().json(restored)
            }
        }
        Ok(None) => render_not_found(&req),
        Err(e) => internal_server_error(e),
    }
}

#[post("/admin/content/{id}/undo")]
pub async fn admin_undo(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let (uid, _item) =
        match load_editable(&state.pool, &req, id).await {
            Ok(v) => v,
            Err(r) => return r,
        };

    if let Err(e) =
        db::ensure_initial_revision(&state.pool, id, Some(uid)).await
    {
        return internal_server_error(e);
    }

    match db::undo(&state.pool, id).await {
        Ok(Some(item)) => HttpResponse::Ok().json(item),
        Ok(None) => render_not_found(&req),
        Err(e) => internal_server_error(e),
    }
}

#[post("/admin/content/{id}/redo")]
pub async fn admin_redo(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let (uid, _item) =
        match load_editable(&state.pool, &req, id).await {
            Ok(v) => v,
            Err(r) => return r,
        };

    if let Err(e) =
        db::ensure_initial_revision(&state.pool, id, Some(uid)).await
    {
        return internal_server_error(e);
    }

    match db::redo(&state.pool, id).await {
        Ok(Some(item)) => HttpResponse::Ok().json(item),
        Ok(None) => render_not_found(&req),
        Err(e) => internal_server_error(e),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(admin_list_revisions)
        .service(admin_get_revision)
        .service(admin_history_panel)
        .service(admin_restore_revision)
        .service(admin_undo)
        .service(admin_redo);
}
