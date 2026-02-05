use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

use rustpress::db;
use rustpress::models::{ContentKind, HomepageType};

use crate::web::helpers::{get_is_admin, render, require_user};
use crate::web::state::AppState;
use crate::web::templates::ConfigurationTemplate;

#[derive(Deserialize)]
pub struct ConfigurationForm {
    pub homepage_type: Option<String>,
    pub homepage_page_id: Option<String>,
}

#[get("/admin/configuration")]
pub async fn configuration_page(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let _uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);
    let site = db::get_default_site(&state.pool).await.ok().flatten();
    let pages = db::list_content(&state.pool, ContentKind::Page, false)
        .await
        .unwrap_or_default();

    render(ConfigurationTemplate {
        site,
        pages,
        error: None,
        success: None,
        is_admin,
    })
}

#[post("/admin/configuration")]
pub async fn configuration_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<ConfigurationForm>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let is_admin = get_is_admin(&req);

    let site = match db::get_default_site(&state.pool).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            let pages = db::list_content(&state.pool, ContentKind::Page, false)
                .await
                .unwrap_or_default();
            return render(ConfigurationTemplate {
                site: None,
                pages,
                error: Some("No site configured".to_string()),
                success: None,
                is_admin,
            });
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

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
        name: None,
        slug: None,
        status: None,
        default_template: None,
        homepage_type,
        homepage_page_id,
    };

    let pages = db::list_content(&state.pool, ContentKind::Page, false)
        .await
        .unwrap_or_default();

    match db::update_site(&state.pool, site.id, uid, &update).await {
        Ok(Some(updated)) => {
            render(ConfigurationTemplate {
                site: Some(updated),
                pages,
                error: None,
                success: Some("Configuration saved".to_string()),
                is_admin,
            })
        }
        Ok(None) => {
            render(ConfigurationTemplate {
                site: Some(site),
                pages,
                error: Some("Update failed - site not found or no permission".to_string()),
                success: None,
                is_admin,
            })
        }
        Err(e) => {
            render(ConfigurationTemplate {
                site: Some(site),
                pages,
                error: Some(format!("Update failed: {e}")),
                success: None,
                is_admin,
            })
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(configuration_page)
        .service(configuration_update);
}
