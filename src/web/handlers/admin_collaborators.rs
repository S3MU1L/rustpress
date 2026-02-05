use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};
use serde::Deserialize;
use uuid::Uuid;

use rustpress::db;
use rustpress::models::RoleName;

use super::super::helpers::{
    render_not_found, render_unauthorized, require_user,
};
use super::super::state::AppState;

#[derive(Deserialize)]
pub struct CollaboratorForm {
    pub email: Option<String>,
    pub role: RoleName,
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
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(e.to_string());
        }
    };

    let can_view =
        match db::can_view_content(&state.pool, &item, uid).await {
            Ok(v) => v,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(e.to_string());
            }
        };

    if !can_view {
        return render_unauthorized(&req);
    }

    match db::list_collaborators(&state.pool, id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => {
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[post("/admin/content/{id}/collaborators")]
pub async fn admin_add_collaborator(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<CollaboratorForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(e.to_string());
        }
    };

    let can_manage =
        match db::can_manage_collaborators(&state.pool, &item, uid)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(e.to_string());
            }
        };

    if !can_manage {
        return render_unauthorized(&req);
    }

    let email =
        form.email.as_deref().unwrap_or("").trim().to_string();
    if email.is_empty() {
        return HttpResponse::BadRequest().body("Email required");
    }

    match db::add_collaborator(
        &state.pool,
        id,
        &email,
        form.role,
        Some(uid),
    )
    .await
    {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/admin/content/{id}/collaborators/{user_id}")]
pub async fn admin_set_collaborator_role(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
    form: web::Form<CollaboratorForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let (id, user_id) = path.into_inner();

    let item = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(e.to_string());
        }
    };

    let can_manage =
        match db::can_manage_collaborators(&state.pool, &item, uid)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(e.to_string());
            }
        };

    if !can_manage {
        return render_unauthorized(&req);
    }

    match db::set_collaborator_role(
        &state.pool,
        id,
        user_id,
        form.role,
    )
    .await
    {
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
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(e.to_string());
        }
    };

    let can_manage =
        match db::can_manage_collaborators(&state.pool, &item, uid)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(e.to_string());
            }
        };

    if !can_manage {
        return render_unauthorized(&req);
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
