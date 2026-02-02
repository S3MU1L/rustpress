use actix_web::{get, web, HttpRequest, Responder};

use rustpress::db;

use crate::web::forms::ThemesQuery;
use crate::web::helpers::{render, require_user};
use crate::web::state::AppState;
use crate::web::templates::ThemesTemplate;

#[get("/admin/themes")]
pub async fn themes_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ThemesQuery>,
) -> impl Responder {
    let uid = match require_user(&req) {
        Ok(uid) => uid,
        Err(resp) => return resp,
    };

    let q = query.q.clone().unwrap_or_default();
    let category = query.category.clone().unwrap_or_else(|| "all".to_string());
    let selected_site_id = query.site_id;

    let mut templates = db::list_site_templates_for_user(&state.pool, uid)
        .await
        .unwrap_or_default();

    let sites = db::list_sites_for_user(&state.pool, uid, None)
        .await
        .unwrap_or_default();

    if !q.trim().is_empty() {
        let needle = q.trim().to_lowercase();
        templates.retain(|t| {
            t.name.to_lowercase().contains(&needle)
                || t.description.to_lowercase().contains(&needle)
        });
    }

    match category.as_str() {
        "builtin" => templates.retain(|t| t.is_builtin),
        "custom" => templates.retain(|t| !t.is_builtin),
        _ => {}
    }

    render(ThemesTemplate {
        templates,
        sites,
        selected_site_id,
        query: q,
        category,
    })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(themes_list);
}
