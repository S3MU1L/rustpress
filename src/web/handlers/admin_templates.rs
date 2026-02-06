use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};
use uuid::Uuid;

use rustpress::db;
use rustpress::models::ContentKind;

use crate::web::forms::{
    AdminTemplateCreateForm, AdminTemplatePreviewForm,
    AdminTemplateUpdateForm,
};
use crate::web::helpers::{
    apply_site_template, get_is_admin, iframe_srcdoc, is_htmx,
    is_unique_violation, render, render_not_found, require_user,
};

use crate::web::state::AppState;
use crate::web::templates::{
    AdminTemplateEditTemplate, AdminTemplateNewTemplate,
    AdminTemplatesListTemplate,
};

async fn fetch_content_items(
    pool: &sqlx::PgPool,
    uid: Uuid,
) -> Vec<rustpress::models::ContentItem> {
    let mut items =
        db::list_content_for_user(pool, ContentKind::Page, true, uid)
            .await
            .unwrap_or_default();
    items.extend(
        db::list_content_for_user(pool, ContentKind::Post, true, uid)
            .await
            .unwrap_or_default(),
    );
    items
}

fn sample_data()
-> (&'static str, &'static str, &'static str, &'static str) {
    (
        "Sample Page Title",
        "<p>Sample content with <strong>bold</strong> and <a href=\"#\">links</a>. This is placeholder text to preview your template layout.</p>",
        "sample-page",
        "page",
    )
}

#[get("/admin/templates")]
pub async fn admin_templates_list(
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
    render(AdminTemplatesListTemplate {
        templates,
        is_admin,
    })
}

#[get("/admin/templates/new")]
pub async fn admin_template_new(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    let content_items = fetch_content_items(&state.pool, uid).await;
    let starter_html = "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\"/>\n    <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"/>\n    <title>{{title}}</title>\n    <link rel=\"stylesheet\" href=\"/static/app.css\"/>\n  </head>\n  <body>\n    <header class=\"topbar\">\n      <div class=\"container\">\n        <a class=\"brand\" href=\"/\">RustPress</a>\n        <nav class=\"nav\"><a href=\"/admin\">Admin</a></nav>\n      </div>\n    </header>\n    <main class=\"container\">\n      <article class=\"card\">\n        <h1>{{title}}</h1>\n        <div class=\"prose\">{{content}}</div>\n      </article>\n    </main>\n  </body>\n</html>\n".to_string();
    render(AdminTemplateNewTemplate {
        starter_html,
        content_items,
        is_admin,
    })
}

#[post("/admin/templates")]
pub async fn admin_template_create(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<AdminTemplateCreateForm>,
) -> impl Responder {
    let owner_user_id = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    // Validate form before processing
    if let Err(e) = form.validate() {
        return HttpResponse::BadRequest()
            .content_type("text/plain; charset=utf-8")
            .body(e.to_string());
    }

    let data = rustpress::models::SiteTemplateCreate {
        owner_user_id,
        name: form.name.trim().to_string(),
        description: form.description.clone().unwrap_or_default(),
        html: form.html.clone(),
    };

    let created = match db::create_site_template(&state.pool, &data)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            if is_unique_violation(&e) {
                return HttpResponse::Conflict()
                    .content_type("text/plain; charset=utf-8")
                    .body(
                        "Template name already exists".to_string(),
                    );
            }
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Create failed: {e}"));
        }
    };

    if is_htmx(&req) {
        HttpResponse::Ok()
            .insert_header((
                "HX-Redirect",
                format!("/admin/templates/{}", created.id),
            ))
            .finish()
    } else {
        HttpResponse::SeeOther()
            .insert_header((
                "Location",
                format!("/admin/templates/{}", created.id),
            ))
            .finish()
    }
}

#[post("/admin/templates/preview")]
pub async fn admin_template_preview(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<AdminTemplatePreviewForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let (title, content, slug, kind) = match form
        .preview_content_id
        .as_ref()
        .filter(|s| !s.is_empty())
    {
        Some(id_str) => {
            if let Ok(content_id) = id_str.parse::<Uuid>() {
                match db::get_content_by_id(&state.pool, content_id)
                    .await
                {
                    Ok(Some(item)) => {
                        let can_view = db::can_view_content(
                            &state.pool,
                            &item,
                            uid,
                        )
                        .await
                        .unwrap_or(false);
                        if can_view {
                            (
                                item.title.clone(),
                                item.content.clone(),
                                item.slug.clone(),
                                item.kind.as_str().to_string(),
                            )
                        } else {
                            let s = sample_data();
                            (
                                s.0.to_string(),
                                s.1.to_string(),
                                s.2.to_string(),
                                s.3.to_string(),
                            )
                        }
                    }
                    _ => {
                        let s = sample_data();
                        (
                            s.0.to_string(),
                            s.1.to_string(),
                            s.2.to_string(),
                            s.3.to_string(),
                        )
                    }
                }
            } else {
                let s = sample_data();
                (
                    s.0.to_string(),
                    s.1.to_string(),
                    s.2.to_string(),
                    s.3.to_string(),
                )
            }
        }
        None => {
            let s = sample_data();
            (
                s.0.to_string(),
                s.1.to_string(),
                s.2.to_string(),
                s.3.to_string(),
            )
        }
    };

    let html = apply_site_template(
        &form.html, &title, &content, &slug, &kind,
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(iframe_srcdoc(&html))
}

#[get("/admin/templates/{id}")]
pub async fn admin_template_edit(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let template =
        match db::get_site_template_by_id(&state.pool, id).await {
            Ok(Some(t)) => t,
            Ok(None) => return render_not_found(&req),
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(e.to_string());
            }
        };

    if !template.is_builtin && template.owner_user_id != Some(uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }
    let is_admin = get_is_admin(&req);
    let content_items = fetch_content_items(&state.pool, uid).await;
    render(AdminTemplateEditTemplate {
        template,
        content_items,
        is_admin,
    })
}

#[post("/admin/templates/{id}")]
pub async fn admin_template_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<AdminTemplateUpdateForm>,
) -> impl Responder {
    let id = path.into_inner();

    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let existing =
        match db::get_site_template_by_id(&state.pool, id).await {
            Ok(Some(t)) => t,
            Ok(None) => return render_not_found(&req),
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(e.to_string());
            }
        };

    if existing.is_builtin {
        return HttpResponse::Forbidden()
            .body("Built-in templates are read-only");
    }

    if existing.owner_user_id != Some(uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }
    let update = rustpress::models::SiteTemplateUpdate {
        name: form.name.as_ref().map(|s| s.trim().to_string()),
        description: form.description.clone(),
        html: form.html.clone(),
    };

    let updated = match db::update_site_template(
        &state.pool,
        id,
        &update,
    )
    .await
    {
        Ok(Some(t)) => t,
        Ok(None) => return render_not_found(&req),
        Err(e) => {
            if is_unique_violation(&e) {
                return HttpResponse::Conflict()
                    .content_type("text/plain; charset=utf-8")
                    .body(
                        "Template name already exists".to_string(),
                    );
            }
            return HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(format!("Update failed: {e}"));
        }
    };

    if is_htmx(&req) {
        let is_admin = get_is_admin(&req);
        let content_items =
            fetch_content_items(&state.pool, uid).await;
        render(AdminTemplateEditTemplate {
            template: updated,
            content_items,
            is_admin,
        })
    } else {
        HttpResponse::SeeOther()
            .insert_header((
                "Location",
                format!("/admin/templates/{}", id),
            ))
            .finish()
    }
}

#[post("/admin/templates/{id}/delete")]
pub async fn admin_template_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let template =
        match db::get_site_template_by_id(&state.pool, id).await {
            Ok(Some(t)) => t,
            Ok(None) => return render_not_found(&req),
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(e.to_string());
            }
        };

    if template.is_builtin {
        return HttpResponse::Forbidden()
            .body("Built-in templates cannot be deleted");
    }

    if template.owner_user_id != Some(uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    match db::delete_site_template(&state.pool, id).await {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::BadRequest().body(
                "Delete failed: template not found or is built-in",
            );
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(e.to_string());
        }
    }

    if is_htmx(&req) {
        HttpResponse::Ok()
            .insert_header(("HX-Redirect", "/admin/templates"))
            .finish()
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", "/admin/templates"))
            .finish()
    }
}

#[post("/admin/templates/{id}/duplicate")]
pub async fn admin_template_duplicate(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let id = path.into_inner();
    let template =
        match db::get_site_template_by_id(&state.pool, id).await {
            Ok(Some(t)) => t,
            Ok(None) => return render_not_found(&req),
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(e.to_string());
            }
        };

    if !template.is_builtin && template.owner_user_id != Some(uid) {
        return HttpResponse::Forbidden().body("Forbidden");
    }

    // Try a few times to avoid rare name collisions.
    let mut last_err: Option<sqlx::Error> = None;
    for _ in 0..3 {
        let suffix = Uuid::new_v4().to_string();
        let short = &suffix[..8];
        let name = format!("{}-copy-{}", template.name, short);

        let data = rustpress::models::SiteTemplateCreate {
            owner_user_id: uid,
            name,
            description: template.description.clone(),
            html: template.html.clone(),
        };

        match db::create_site_template(&state.pool, &data).await {
            Ok(created) => {
                if is_htmx(&req) {
                    return HttpResponse::Ok()
                        .insert_header((
                            "HX-Redirect",
                            format!(
                                "/admin/templates/{}",
                                created.id
                            ),
                        ))
                        .finish();
                }
                return HttpResponse::SeeOther()
                    .insert_header((
                        "Location",
                        format!("/admin/templates/{}", created.id),
                    ))
                    .finish();
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    let msg = last_err
        .as_ref()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "Duplicate failed".to_string());
    HttpResponse::BadRequest().body(msg)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(admin_templates_list)
        .service(admin_template_new)
        .service(admin_template_create)
        .service(admin_template_preview)
        .service(admin_template_edit)
        .service(admin_template_update)
        .service(admin_template_delete)
        .service(admin_template_duplicate);
}
