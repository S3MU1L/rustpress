use actix_web::{get, web, HttpResponse, Responder};

use rustpress::db;
use rustpress::models::ContentKind;

use crate::web::helpers::{apply_site_template, normalize_builtin_template_html, render};
use crate::web::state::AppState;
use crate::web::templates::{PublicContentTemplate, PublicIndexTemplate};

#[get("/")]
pub async fn public_index(state: web::Data<AppState>) -> impl Responder {
    let posts = db::list_content(&state.pool, ContentKind::Post, false)
        .await
        .unwrap_or_default();

    render(PublicIndexTemplate { posts })
}

#[get("/post/{slug}")]
pub async fn public_post(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let slug = path.into_inner();
    let maybe = db::get_published_by_slug(&state.pool, ContentKind::Post, &slug)
        .await
        .ok()
        .flatten();

    match maybe {
        Some(item) => {
            let mut tpl = match item.owner_user_id {
                Some(owner_id) => {
                    db::get_site_template_by_name_for_user(&state.pool, owner_id, &item.template)
                        .await
                        .ok()
                        .flatten()
                }
                None => db::get_site_template_by_name(&state.pool, &item.template)
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
                        &item.kind,
                    )
                }
                None => apply_site_template(
                    "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{title}}</title></head><body><h1>{{title}}</h1>{{content}}</body></html>",
                    &item.title,
                    &item.content,
                    &item.slug,
                    &item.kind,
                ),
            };

            render(PublicContentTemplate { html })
        }
        None => HttpResponse::NotFound().body("Not found"),
    }
}

#[get("/p/{slug}")]
pub async fn public_page(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let slug = path.into_inner();
    let maybe = db::get_published_by_slug(&state.pool, ContentKind::Page, &slug)
        .await
        .ok()
        .flatten();

    match maybe {
        Some(item) => {
            let mut tpl = match item.owner_user_id {
                Some(owner_id) => {
                    db::get_site_template_by_name_for_user(&state.pool, owner_id, &item.template)
                        .await
                        .ok()
                        .flatten()
                }
                None => db::get_site_template_by_name(&state.pool, &item.template)
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
                        &item.kind,
                    )
                }
                None => apply_site_template(
                    "<!doctype html><html><head><meta charset=\"utf-8\"><title>{{title}}</title></head><body><h1>{{title}}</h1>{{content}}</body></html>",
                    &item.title,
                    &item.content,
                    &item.slug,
                    &item.kind,
                ),
            };

            render(PublicContentTemplate { html })
        }
        None => HttpResponse::NotFound().body("Not found"),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(public_index)
        .service(public_post)
        .service(public_page);
}
