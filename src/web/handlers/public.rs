use actix_web::{get, web, HttpResponse, Responder};

use rustpress::db;
use rustpress::models::ContentKind;

use crate::web::helpers::{apply_site_template, normalize_builtin_template_html, render};
use crate::web::state::AppState;
use crate::web::templates::{PublicContentTemplate, PublicIndexTemplate};

/// Home page - shows landing or redirects to blog
#[get("/")]
pub async fn home_page(state: web::Data<AppState>) -> impl Responder {
    // For now, show posts index as the home page
    let posts = db::list_content(&state.pool, ContentKind::Post, false)
        .await
        .unwrap_or_default();

    render(PublicIndexTemplate { posts })
}

/// Blog index - lists all published posts
#[get("/blog")]
pub async fn blog_index(state: web::Data<AppState>) -> impl Responder {
    let posts = db::list_content(&state.pool, ContentKind::Post, false)
        .await
        .unwrap_or_default();

    render(PublicIndexTemplate { posts })
}

/// Single blog post by slug
#[get("/blog/{slug}")]
pub async fn blog_post(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
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

/// Page resolver - catch-all for pages by slug
/// This must be registered LAST to avoid conflicts with other routes
#[get("/{path:.*}")]
pub async fn page_page(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let slug = path.into_inner();

    // Empty path is handled by home_page
    if slug.is_empty() {
        return HttpResponse::NotFound().body("Not found");
    }

    // Reserved paths - these should never match as pages
    if slug == "admin" || slug.starts_with("admin/") || slug == "blog" || slug.starts_with("blog/")
    {
        return HttpResponse::NotFound().body("Not found");
    }

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
    cfg.service(home_page)
        .service(blog_index)
        .service(blog_post);
    // NOTE: page_page must be registered separately and LAST in main.rs
    // because it's a catch-all route that would otherwise match everything
}
