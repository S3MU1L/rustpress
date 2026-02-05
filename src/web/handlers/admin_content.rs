use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use uuid::Uuid;

use rustpress::db;
use rustpress::models::{ContentCreate, ContentKind, ContentStatus, ContentUpdate};

use crate::web::forms::{
    AdminCreateForm, AdminLiveForm, AdminNewPreviewForm, AdminUpdateForm, SitesQuery,
};
use crate::web::helpers::{
    apply_site_template, escape_html, get_is_admin, iframe_srcdoc, is_htmx, is_unique_violation,
    normalize_builtin_template_html, render, require_user,
};
use crate::web::state::AppState;
use crate::web::templates::{
    AdminDashboardTemplate, AdminEditTemplate, AdminNewTemplate, AdminPagesListTemplate,
    AdminPostsListTemplate,
};

#[get("/admin")]
pub async fn admin_dashboard(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    let posts = db::list_content_for_user(&state.pool, ContentKind::Post, true, uid)
        .await
        .unwrap_or_default();
    let pages = db::list_content_for_user(&state.pool, ContentKind::Page, true, uid)
        .await
        .unwrap_or_default();

    render(AdminDashboardTemplate { posts, pages, is_admin })
}

#[get("/admin/posts")]
pub async fn admin_posts_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SitesQuery>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    let q = query.q.clone().unwrap_or_default();
    let posts = db::list_content_for_user(&state.pool, ContentKind::Post, true, uid)
        .await
        .unwrap_or_default();

    let posts: Vec<_> = posts
        .into_iter()
        .filter(|c| q.is_empty() || c.title.to_lowercase().contains(&q.to_lowercase()))
        .collect();

    render(AdminPostsListTemplate { posts, query: q, is_admin })
}

#[get("/admin/pages")]
pub async fn admin_pages_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<SitesQuery>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    let q = query.q.clone().unwrap_or_default();
    let pages = db::list_content_for_user(&state.pool, ContentKind::Page, true, uid)
        .await
        .unwrap_or_default();

    let pages: Vec<_> = pages
        .into_iter()
        .filter(|c| q.is_empty() || c.title.to_lowercase().contains(&q.to_lowercase()))
        .collect();

    render(AdminPagesListTemplate { pages, query: q, is_admin })
}

#[get("/admin/{kind:posts|pages}/new")]
pub async fn admin_new(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    let kind = path.into_inner();

    let templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();

    render(AdminNewTemplate {
        kind,
        default_template: "default".to_string(),
        templates,
        is_admin,
    })
}

#[post("/admin/{kind:posts|pages}")]
pub async fn admin_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    form: web::Form<AdminCreateForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let kind = match path.into_inner().as_str() {
        "posts" => ContentKind::Post,
        "pages" => ContentKind::Page,
        _ => return HttpResponse::NotFound().body("Unknown kind"),
    };

    let data = ContentCreate {
        owner_user_id: Some(uid),
        kind,
        title: form.title.trim().to_string(),
        slug: form.slug.trim().to_string(),
        content: form.content.to_string(),
        template: form
            .template
            .clone()
            .unwrap_or_else(|| "default".to_string()),
    };

    let created = match db::create_content(&state.pool, &data).await {
        Ok(item) => item,
        Err(e) => {
            if is_unique_violation(&e) {
                return HttpResponse::Conflict()
                    .content_type("text/plain; charset=utf-8")
                    .body("Slug already exists for this content type".to_string());
            }

            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Create failed: {e}"));
        }
    };

    // Seed history immediately so undo/redo works from the moment the item exists.
    if let Err(e) = db::ensure_initial_revision(&state.pool, created.id, Some(uid)).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    if is_htmx(&req) {
        HttpResponse::Ok()
            .insert_header(("HX-Redirect", format!("/admin/edit/{}", created.id)))
            .finish()
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/admin/edit/{}", created.id)))
            .finish()
    }
}

#[get("/admin/edit/{id}")]
pub async fn admin_edit(
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

    // Ensure legacy items have a baseline revision.
    if let Err(e) = db::ensure_initial_revision(&state.pool, item.id, Some(uid)).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    let is_admin = get_is_admin(&req);
    let templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();
    render(AdminEditTemplate { item, templates, is_admin })
}

#[post("/admin/edit/{id}")]
pub async fn admin_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AdminUpdateForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();

    // Enforce ownership before mutating.
    let existing = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let can_edit = match db::can_edit_content(&state.pool, &existing, uid).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !can_edit {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let status = match form.status.as_deref().map(|s| s.trim()) {
        Some("draft") => Some(ContentStatus::Draft),
        Some("published") => Some(ContentStatus::Published),
        Some("") | None => None,
        Some(_) => {
            return HttpResponse::BadRequest().body("Invalid status");
        }
    };

    let update = ContentUpdate {
        title: form.title.as_ref().map(|s| s.trim().to_string()),
        slug: form.slug.as_ref().map(|s| s.trim().to_string()),
        content: form.content.clone(),
        template: form.template.as_ref().map(|s| s.trim().to_string()),
        status,
    };

    let updated = match db::update_content(&state.pool, id, &update).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            if is_unique_violation(&e) {
                return HttpResponse::Conflict()
                    .content_type("text/plain; charset=utf-8")
                    .body("Slug already exists for this content type".to_string());
            }
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Update failed: {e}"));
        }
    };

    // Record a new revision for explicit saves.
    if let Err(e) = db::ensure_initial_revision(&state.pool, id, Some(uid)).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }
    if let Err(e) = db::record_revision(&state.pool, &updated, Some(uid)).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    if is_htmx(&req) {
        let is_admin = get_is_admin(&req);
        let templates = db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default();
        render(AdminEditTemplate {
            item: updated,
            templates,
            is_admin,
        })
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/admin/edit/{}", id)))
            .finish()
    }
}

#[post("/admin/publish/{id}")]
pub async fn admin_publish(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();

    // Enforce ownership before mutating.
    let existing = match db::get_content_by_id(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let can_edit = match db::can_edit_content(&state.pool, &existing, uid).await {
        Ok(v) => v,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    if !can_edit {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let published = match db::publish_content(&state.pool, id).await {
        Ok(Some(item)) => item,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Publish failed: {e}"));
        }
    };

    if let Err(e) = db::ensure_initial_revision(&state.pool, id, Some(uid)).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }
    if let Err(e) = db::record_revision(&state.pool, &published, Some(uid)).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    if is_htmx(&req) {
        let is_admin = get_is_admin(&req);
        let templates = db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default();
        render(AdminEditTemplate {
            item: published,
            templates,
            is_admin,
        })
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", format!("/admin/edit/{}", id)))
            .finish()
    }
}

#[post("/admin/edit/{id}/autosave")]
pub async fn admin_autosave(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AdminLiveForm>,
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

    // Autosave should never implicitly publish.
    let update = ContentUpdate {
        title: form.title.as_ref().map(|s| s.trim().to_string()),
        slug: form.slug.as_ref().map(|s| s.trim().to_string()),
        content: form.content.clone(),
        template: form.template.as_ref().map(|s| s.trim().to_string()),
        status: None,
    };

    match db::update_content(&state.pool, id, &update).await {
        Ok(Some(updated)) => {
            if let Err(e) = db::ensure_initial_revision(&state.pool, id, Some(uid)).await {
                return HttpResponse::BadRequest()
                    .content_type("text/html; charset=utf-8")
                    .body(format!(
                        "<span class=\"muted\">Autosave failed: {}</span>",
                        escape_html(&e.to_string())
                    ));
            }
            if let Err(e) = db::record_revision(&state.pool, &updated, Some(uid)).await {
                return HttpResponse::BadRequest()
                    .content_type("text/html; charset=utf-8")
                    .body(format!(
                        "<span class=\"muted\">Autosave failed: {}</span>",
                        escape_html(&e.to_string())
                    ));
            }

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(format!(
                    "<span class=\"muted\">Autosaved at {}</span>",
                    Utc::now().format("%H:%M:%S")
                ))
        }
        Ok(None) => HttpResponse::NotFound().body("Not found"),
        Err(e) => HttpResponse::BadRequest()
            .content_type("text/html; charset=utf-8")
            .body(format!(
                "<span class=\"muted\">Autosave failed: {}</span>",
                escape_html(&e.to_string())
            )),
    }
}

#[post("/admin/edit/{id}/preview")]
pub async fn admin_preview(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AdminLiveForm>,
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

    let title = form
        .title
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| item.title.clone());
    let slug = form
        .slug
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| item.slug.clone());
    let content = form.content.clone().unwrap_or_else(|| item.content.clone());
    let template_name = form
        .template
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| item.template.clone());

    let mut tpl = match item.owner_user_id {
        Some(owner_id) => {
            db::get_site_template_by_name_for_user(&state.pool, owner_id, &template_name)
                .await
                .ok()
                .flatten()
        }
        None => db::get_site_template_by_name(&state.pool, &template_name)
            .await
            .ok()
            .flatten(),
    };
    if tpl.is_none() {
        tpl = match item.owner_user_id {
            Some(owner_id) => {
                db::get_site_template_by_name_for_user(&state.pool, owner_id, "default")
                    .await
                    .ok()
                    .flatten()
            }
            None => db::get_site_template_by_name(&state.pool, "default")
                .await
                .ok()
                .flatten(),
        };
    }

    let html = match tpl {
        Some(tpl) => {
            let tpl_html = if tpl.is_builtin {
                normalize_builtin_template_html(&tpl.html)
            } else {
                std::borrow::Cow::Borrowed(tpl.html.as_str())
            };
            apply_site_template(tpl_html.as_ref(), &title, &content, &slug, item.kind.as_str())
        }
        None => apply_site_template(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{title}}</title></head><body><h1>{{title}}</h1>{{content}}</body></html>",
            &title,
            &content,
            &slug,
            item.kind.as_str(),
        ),
    };

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(iframe_srcdoc(&html))
}

/// Full-page preview in a new tab (does NOT publish)
#[get("/admin/content/{id}/preview")]
pub async fn admin_preview_fullpage(
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

    let template_name = &item.template;

    let mut tpl = match item.owner_user_id {
        Some(owner_id) => {
            db::get_site_template_by_name_for_user(&state.pool, owner_id, template_name)
                .await
                .ok()
                .flatten()
        }
        None => db::get_site_template_by_name(&state.pool, template_name)
            .await
            .ok()
            .flatten(),
    };
    if tpl.is_none() {
        tpl = match item.owner_user_id {
            Some(owner_id) => {
                db::get_site_template_by_name_for_user(&state.pool, owner_id, "default")
                    .await
                    .ok()
                    .flatten()
            }
            None => db::get_site_template_by_name(&state.pool, "default")
                .await
                .ok()
                .flatten(),
        };
    }

    let html = match tpl {
        Some(tpl) => {
            let tpl_html = if tpl.is_builtin {
                normalize_builtin_template_html(&tpl.html)
            } else {
                std::borrow::Cow::Borrowed(tpl.html.as_str())
            };
            apply_site_template(
                tpl_html.as_ref(),
                &item.title,
                &item.content,
                &item.slug,
                item.kind.as_str(),
            )
        }
        None => apply_site_template(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{title}}</title></head><body><h1>{{title}}</h1>{{content}}</body></html>",
            &item.title,
            &item.content,
            &item.slug,
            item.kind.as_str(),
        ),
    };

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[post("/admin/preview")]
pub async fn admin_preview_new(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<AdminNewPreviewForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let kind = match form.kind.trim() {
        "posts" => "post",
        "pages" => "page",
        other => other,
    };

    let title = form
        .title
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Untitled".to_string());
    let slug = form
        .slug
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "untitled".to_string());
    let content = form.content.clone().unwrap_or_default();
    let template_name = form
        .template
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());

    let mut tpl = db::get_site_template_by_name_for_user(&state.pool, uid, &template_name)
        .await
        .ok()
        .flatten();
    if tpl.is_none() {
        tpl = db::get_site_template_by_name_for_user(&state.pool, uid, "default")
            .await
            .ok()
            .flatten();
    }

    let html = match tpl {
        Some(tpl) => {
            let tpl_html = if tpl.is_builtin {
                normalize_builtin_template_html(&tpl.html)
            } else {
                std::borrow::Cow::Borrowed(tpl.html.as_str())
            };
            apply_site_template(tpl_html.as_ref(), &title, &content, &slug, kind)
        }
        None => apply_site_template(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{title}}</title></head><body><h1>{{title}}</h1>{{content}}</body></html>",
            &title,
            &content,
            &slug,
            kind,
        ),
    };

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(iframe_srcdoc(&html))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(admin_dashboard)
        .service(admin_posts_list)
        .service(admin_pages_list)
        .service(admin_new)
        .service(admin_create)
        .service(admin_edit)
        .service(admin_update)
        .service(admin_publish)
        .service(admin_autosave)
        .service(admin_preview)
        .service(admin_preview_fullpage)
        .service(admin_preview_new);
}
