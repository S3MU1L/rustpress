use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};
use uuid::Uuid;

use rustpress::db;
use rustpress::models::HomepageType;

use crate::web::forms::{
    ApplyThemeForm, SiteCreateForm, SiteUpdateForm, SitesQuery,
};
use crate::web::helpers::{
    get_is_admin, is_htmx, render, render_not_found, require_user,
};
use crate::web::state::AppState;
use crate::web::templates::{
    SiteEditTemplate, SiteNewTemplate, SitesListTemplate,
};

#[get("/admin/sites")]
pub async fn sites_list(
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
    let sites =
        db::list_sites_for_user(&state.pool, uid, query.q.as_deref())
            .await
            .unwrap_or_default();

    render(SitesListTemplate {
        sites,
        query: q,
        is_admin,
    })
}

#[get("/admin/sites/new")]
pub async fn sites_new(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    let templates =
        db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default();

    render(SiteNewTemplate {
        templates,
        default_template: "default".to_string(),
        error: None,
        is_admin,
    })
}

#[post("/admin/sites")]
pub async fn sites_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<SiteCreateForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);

    // Validate form before processing
    if let Err(e) = form.validate() {
        let templates =
            db::list_site_templates_for_user(&state.pool, uid)
                .await
                .unwrap_or_default();
        return render(SiteNewTemplate {
            templates,
            default_template: form
                .default_template
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            error: Some(e.to_string()),
            is_admin,
        });
    }

    let name = form.name.trim().to_string();
    let slug = form.slug.trim().to_string();

    let default_template = form
        .default_template
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());

    let created = db::create_site(
        &state.pool,
        &rustpress::models::SiteCreate {
            owner_user_id: uid,
            name,
            slug,
            default_template,
        },
    )
    .await;

    match created {
        Ok(site) => {
            if is_htmx(&req) {
                HttpResponse::Ok()
                    .insert_header((
                        "HX-Redirect",
                        format!("/admin/sites/{}", site.id),
                    ))
                    .finish()
            } else {
                HttpResponse::SeeOther()
                    .insert_header((
                        "Location",
                        format!("/admin/sites/{}", site.id),
                    ))
                    .finish()
            }
        }
        Err(e) => {
            let templates =
                db::list_site_templates_for_user(&state.pool, uid)
                    .await
                    .unwrap_or_default();
            render(SiteNewTemplate {
                templates,
                default_template: form
                    .default_template
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
                error: Some(format!("Create failed: {e}")),
                is_admin,
            })
        }
    }
}

#[get("/admin/sites/{id}")]
pub async fn sites_edit(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };
    let id = path.into_inner();

    let site = match db::get_site_by_id(&state.pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(e.to_string());
        }
    };

    if site.owner_user_id != uid {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let is_admin = get_is_admin(&req);
    let templates =
        db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default();

    render(SiteEditTemplate {
        site,
        templates,
        error: None,
        success: None,
        is_admin,
    })
}

#[post("/admin/sites/{id}")]
pub async fn sites_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<SiteUpdateForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };
    let id = path.into_inner();

    let existing = match db::get_site_by_id(&state.pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(e.to_string());
        }
    };

    if existing.owner_user_id != uid {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let homepage_type: Option<HomepageType> = form
        .homepage_type
        .as_ref()
        .and_then(|s| s.trim().parse().ok());

    let homepage_page_id: Option<Option<Uuid>> = match homepage_type {
        Some(HomepageType::Posts) => Some(None),
        Some(HomepageType::Page) => form
            .homepage_page_id
            .as_ref()
            .and_then(|s| s.trim().parse::<Uuid>().ok())
            .map(Some),
        None => None,
    };

    let update = rustpress::models::SiteUpdate {
        name: form
            .name
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        slug: form
            .slug
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        status: None,
        default_template: form
            .default_template
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        homepage_type,
        homepage_page_id,
    };

    let is_admin = get_is_admin(&req);

    if let Err(e) = update.validate_homepage() {
        let templates =
            db::list_site_templates_for_user(&state.pool, uid)
                .await
                .unwrap_or_default();
        return render(SiteEditTemplate {
            site: existing,
            templates,
            error: Some(e),
            success: None,
            is_admin,
        });
    }

    let updated = match db::update_site(&state.pool, id, uid, &update)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            let templates =
                db::list_site_templates_for_user(&state.pool, uid)
                    .await
                    .unwrap_or_default();
            return render(SiteEditTemplate {
                site: existing,
                templates,
                error: Some(format!("Update failed: {e}")),
                success: None,
                is_admin,
            });
        }
    };

    let templates =
        db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default();

    render(SiteEditTemplate {
        site: updated,
        templates,
        error: None,
        success: Some("Saved".to_string()),
        is_admin,
    })
}

#[post("/admin/sites/{id}/publish")]
pub async fn sites_publish(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };
    let id = path.into_inner();

    let existing = match db::get_site_by_id(&state.pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(e.to_string());
        }
    };
    if existing.owner_user_id != uid {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let published = match db::publish_site(&state.pool, id, uid).await
    {
        Ok(Some(s)) => s,
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            return HttpResponse::BadRequest()
                .body(format!("Publish failed: {e}"));
        }
    };

    let is_admin = get_is_admin(&req);
    render(SiteEditTemplate {
        site: published,
        templates: db::list_site_templates_for_user(&state.pool, uid)
            .await
            .unwrap_or_default(),
        error: None,
        success: Some("Site published".to_string()),
        is_admin,
    })
}

#[post("/admin/sites/{id}/theme")]
pub async fn sites_apply_theme(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<ApplyThemeForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let existing = match db::get_site_by_id(&state.pool, id).await {
        Ok(Some(s)) => s,
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(e.to_string());
        }
    };

    if existing.owner_user_id != uid {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    let template_name = form.template.trim();
    if template_name.is_empty() {
        return HttpResponse::BadRequest()
            .body("Template is required");
    }

    // Only allow selecting templates visible to this user.
    let allowed = db::get_site_template_by_name_for_user(
        &state.pool,
        uid,
        template_name,
    )
    .await
    .ok()
    .flatten();
    if allowed.is_none() {
        return HttpResponse::BadRequest().body("Unknown template");
    }

    let update = rustpress::models::SiteUpdate {
        name: None,
        slug: None,
        status: None,
        default_template: Some(template_name.to_string()),
        homepage_type: None,
        homepage_page_id: None,
    };

    let updated =
        match db::update_site(&state.pool, id, uid, &update).await {
            Ok(Some(s)) => s,
            Ok(None) => return render_not_found(&req),
            Err(e) => {
                return HttpResponse::BadRequest()
                    .body(format!("Update failed: {e}"));
            }
        };

    if is_htmx(&req) {
        HttpResponse::Ok()
            .insert_header((
                "HX-Redirect",
                format!("/admin/sites/{}", updated.id),
            ))
            .finish()
    } else {
        HttpResponse::SeeOther()
            .insert_header((
                "Location",
                format!("/admin/sites/{}", updated.id),
            ))
            .finish()
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(sites_list)
        .service(sites_new)
        .service(sites_create)
        .service(sites_edit)
        .service(sites_update)
        .service(sites_publish)
        .service(sites_apply_theme);
}
