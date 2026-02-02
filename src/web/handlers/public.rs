use actix_web::{HttpResponse, Responder, get, web};
use sqlx::PgPool;

use rustpress::db;
use rustpress::models::{ContentItem, ContentKind, HomepageType};

use crate::web::helpers::{
    apply_site_template, normalize_builtin_template_html, render,
};
use crate::web::state::AppState;
use crate::web::templates::{
    PublicContentTemplate, PublicFallbackTemplate,
    PublicIndexTemplate,
};

async fn render_content(
    pool: &PgPool,
    item: &ContentItem,
) -> HttpResponse {
    match get_template_for_item(pool, item).await {
        Some(tpl) => {
            let tpl_html = if tpl.is_builtin {
                normalize_builtin_template_html(&tpl.html)
            } else {
                std::borrow::Cow::Borrowed(tpl.html.as_str())
            };
            let html = apply_site_template(
                tpl_html.as_ref(),
                &item.title,
                &item.content,
                &item.slug,
                item.kind.as_str(),
            );
            render(PublicContentTemplate { html })
        }
        None => render(PublicFallbackTemplate {
            title: &item.title,
            content: &item.content,
        }),
    }
}

async fn get_template_for_item(
    pool: &PgPool,
    item: &ContentItem,
) -> Option<rustpress::models::SiteTemplate> {
    let tpl = match item.owner_user_id {
        Some(owner_id) => db::get_site_template_by_name_for_user(
            pool,
            owner_id,
            &item.template,
        )
        .await
        .ok()
        .flatten(),
        None => db::get_site_template_by_name(pool, &item.template)
            .await
            .ok()
            .flatten(),
    };

    if tpl.is_some() {
        return tpl;
    }

    match item.owner_user_id {
        Some(owner_id) => db::get_site_template_by_name_for_user(
            pool, owner_id, "default",
        )
        .await
        .ok()
        .flatten(),
        None => db::get_site_template_by_name(pool, "default")
            .await
            .ok()
            .flatten(),
    }
}

async fn render_posts_index(pool: &PgPool) -> HttpResponse {
    let posts = db::list_content(pool, ContentKind::Post, false)
        .await
        .unwrap_or_default();
    render(PublicIndexTemplate { posts })
}

#[get("/")]
pub async fn home_page(state: web::Data<AppState>) -> impl Responder {
    if let Ok(Some(site)) = db::get_default_site(&state.pool).await {
        match site.homepage_type {
            HomepageType::Posts => {
                return render_posts_index(&state.pool).await;
            }
            HomepageType::Page => {
                if let Some(page_id) = site.homepage_page_id
                    && let Ok(Some(page)) =
                        db::get_content_by_id(&state.pool, page_id)
                            .await
                {
                    return render_content(&state.pool, &page).await;
                }
            }
        }
    }

    render_posts_index(&state.pool).await
}

#[get("/blog")]
pub async fn blog_index(
    state: web::Data<AppState>,
) -> impl Responder {
    render_posts_index(&state.pool).await
}

#[get("/blog/{slug}")]
pub async fn blog_post(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let slug = path.into_inner();

    match db::get_published_by_slug(
        &state.pool,
        ContentKind::Post,
        &slug,
    )
    .await
    .ok()
    .flatten()
    {
        Some(item) => render_content(&state.pool, &item).await,
        None => HttpResponse::NotFound().body("Not found"),
    }
}

#[get("/{path:.*}")]
pub async fn page_page(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let slug = path.into_inner();

    if slug.is_empty()
        || slug == "admin"
        || slug.starts_with("admin/")
        || slug == "blog"
        || slug.starts_with("blog/")
    {
        return HttpResponse::NotFound().body("Not found");
    }

    match db::get_published_by_slug(
        &state.pool,
        ContentKind::Page,
        &slug,
    )
    .await
    .ok()
    .flatten()
    {
        Some(item) => render_content(&state.pool, &item).await,
        None => HttpResponse::NotFound().body("Not found"),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(home_page)
        .service(blog_index)
        .service(blog_post);
}
