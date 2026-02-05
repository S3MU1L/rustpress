use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, web,
};
use uuid::Uuid;

use rustpress::db;

use crate::web::forms::{
    AdminTemplateCreateForm, AdminTemplateUpdateForm,
};
use crate::web::helpers::{
    get_is_admin, is_htmx, is_unique_violation, render,
    render_not_found, require_user,
};
use crate::web::state::AppState;
use crate::web::templates::{
    AdminTemplateEditTemplate, AdminTemplateNewTemplate,
    AdminTemplatesListTemplate,
};

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
    _state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = require_user(&req) {
        return resp;
    }

    let is_admin = get_is_admin(&req);
    let starter_html = "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\"/>\n    <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"/>\n    <title>{{title}}</title>\n    <link rel=\"stylesheet\" href=\"/static/app.css\"/>\n  </head>\n  <body>\n    <header class=\"topbar\">\n      <div class=\"container\">\n        <a class=\"brand\" href=\"/\">RustPress</a>\n        <nav class=\"nav\"><a href=\"/admin\">Admin</a></nav>\n      </div>\n    </header>\n    <main class=\"container\">\n      <article class=\"card\">\n        <h1>{{title}}</h1>\n        <div class=\"prose\">{{content}}</div>\n      </article>\n    </main>\n  </body>\n</html>\n".to_string();
    render(AdminTemplateNewTemplate {
        starter_html,
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

    if !template.is_builtin {
        if template.owner_user_id != Some(uid) {
            return HttpResponse::Forbidden().body("Forbidden");
        }
    }
    let is_admin = get_is_admin(&req);
    render(AdminTemplateEditTemplate { template, is_admin })
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
        render(AdminTemplateEditTemplate {
            template: updated,
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
        .service(admin_template_edit)
        .service(admin_template_update)
        .service(admin_template_duplicate);
}
