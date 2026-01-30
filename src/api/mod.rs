use leptos::prelude::*;

#[cfg(feature = "ssr")]
use actix_web::cookie::Cookie;
#[cfg(feature = "ssr")]
use actix_web::http::header::{HeaderValue, SET_COOKIE};
#[cfg(feature = "ssr")]
use actix_web::HttpRequest;
#[cfg(feature = "ssr")]
use leptos_actix::extract;
#[cfg(feature = "ssr")]
use leptos_actix::ResponseOptions;
#[cfg(feature = "ssr")]
use sqlx::PgPool;
#[cfg(feature = "ssr")]
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::services::PasswordManager;
use crate::types::*;

const AUTH_COOKIE: &str = "rp_uid";

#[cfg(feature = "ssr")]
async fn pool() -> Result<PgPool, ServerFnError> {
    let pool = extract::<actix_web::web::Data<PgPool>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(pool.get_ref().clone())
}

#[cfg(feature = "ssr")]
fn set_auth_cookie(uid: Uuid) -> Result<(), ServerFnError> {
    let resp = use_context::<ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("Missing ResponseOptions"))?;

    // We set a cookie header manually so this works for both SSR and client calls.
    let cookie = Cookie::build(AUTH_COOKIE, uid.to_string())
        .path("/")
        .http_only(true)
        .same_site(actix_web::cookie::SameSite::Lax)
        .finish();
    let hv = HeaderValue::from_str(&cookie.to_string())
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    resp.append_header(SET_COOKIE, hv);
    Ok(())
}

#[cfg(feature = "ssr")]
async fn current_user_id() -> Result<Option<Uuid>, ServerFnError> {
    let req = extract::<HttpRequest>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if let Some(c) = req.cookie(AUTH_COOKIE) {
        if let Ok(uid) = Uuid::parse_str(c.value()) {
            return Ok(Some(uid));
        }
    }

    Ok(None)
}

#[cfg(feature = "ssr")]
async fn require_user_id() -> Result<Uuid, ServerFnError> {
    current_user_id()
        .await?
        .ok_or_else(|| ServerFnError::new("Not logged in"))
}

#[cfg(feature = "ssr")]
fn content_to_public(c: crate::models::ContentItem) -> ContentPublic {
    ContentPublic {
        id: c.id.to_string(),
        owner_user_id: c.owner_user_id.map(|u| u.to_string()),
        kind: c.kind,
        status: c.status,
        title: c.title,
        slug: c.slug,
        content: c.content,
        template: c.template,
        created_at: c.created_at,
        edited_at: c.edited_at,
        published_at: c.published_at,
    }
}

#[cfg(feature = "ssr")]
fn template_to_summary(t: crate::models::SiteTemplate) -> TemplateSummary {
    TemplateSummary {
        id: t.id.to_string(),
        owner_user_id: t.owner_user_id.map(|u| u.to_string()),
        name: t.name,
        description: t.description,
        is_builtin: t.is_builtin,
        created_at: t.created_at,
        edited_at: t.edited_at,
    }
}

#[cfg(feature = "ssr")]
fn template_to_detail(t: crate::models::SiteTemplate) -> TemplateDetail {
    TemplateDetail {
        id: t.id.to_string(),
        owner_user_id: t.owner_user_id.map(|u| u.to_string()),
        name: t.name,
        description: t.description,
        html: t.html,
        is_builtin: t.is_builtin,
        created_at: t.created_at,
        edited_at: t.edited_at,
    }
}

/// Login server function - authenticates user credentials
#[server(Login, "/api")]
pub async fn login(email: String, password: String) -> Result<LoginResponse, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (email, password);
        return Err(ServerFnError::new("Login is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let email = email.trim().to_string();
        if email.is_empty() || password.is_empty() {
            return Err(ServerFnError::new("Invalid credentials"));
        }

        let pool = pool().await?;
        let database = crate::db::Database::from_pool(pool);
        let user = database
            .get_user(&crate::models::UserIden::Email(email.clone()), false)
            .await
            .map_err(|_| ServerFnError::new("Invalid credentials"))?;

        let ok = PasswordManager::verify_password(password.as_str(), &user.password_hash)
            .map_err(|_| ServerFnError::new("Invalid credentials"))?;
        if !ok {
            return Err(ServerFnError::new("Invalid credentials"));
        }

        set_auth_cookie(user.id)?;

        Ok(LoginResponse {
            user: UserPublic {
                id: user.id.to_string(),
                email: user.email.clone(),
                username: user.email.split('@').next().map(|s| s.to_string()),
            },
            // MVP token: same as user id. Frontend should rely on cookie.
            token: user.id.to_string(),
        })
    }
}

/// Register server function - creates a new user account
#[server(Register, "/api")]
pub async fn register(
    email: String,
    password: String,
    username: Option<String>,
) -> Result<RegisterResponse, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (email, password, username);
        return Err(ServerFnError::new("Register is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let email = email.trim().to_string();
        if email.is_empty() || !email.contains('@') {
            return Err(ServerFnError::new("Please enter a valid email"));
        }
        if password.len() < 6 {
            return Err(ServerFnError::new("Password must be at least 6 characters"));
        }

        let password_hash = PasswordManager::hash_password(password.as_str())
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let pool = pool().await?;
        let mut database = crate::db::Database::from_pool(pool);
        let user = database
            .add_user(&crate::models::UserCreate {
                email: email.clone(),
                password_hash,
            })
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        set_auth_cookie(user.id)?;

        Ok(RegisterResponse {
            user: UserPublic {
                id: user.id.to_string(),
                email: user.email.clone(),
                username: username.or_else(|| user.email.split('@').next().map(|s| s.to_string())),
            },
            message: "Registration successful!".into(),
        })
    }
}

/// Get current user - retrieves the logged-in user's information
#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<UserPublic>, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        return Ok(None);
    }

    #[cfg(feature = "ssr")]
    {
        let Some(uid) = current_user_id().await? else {
            return Ok(None);
        };

        let pool = pool().await?;
        let database = crate::db::Database::from_pool(pool);
        let user = database
            .get_user(&crate::models::UserIden::Id(uid), false)
            .await
            .map_err(|_| ServerFnError::new("Not logged in"))?;

        Ok(Some(UserPublic {
            id: user.id.to_string(),
            email: user.email.clone(),
            username: user.email.split('@').next().map(|s| s.to_string()),
        }))
    }
}

/// List content items (pages/posts) for the current user.
#[server(ListContent, "/api")]
pub async fn list_content(kind: String, include_drafts: bool) -> Result<Vec<ContentPublic>, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (kind, include_drafts);
        return Err(ServerFnError::new("ListContent is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let uid = require_user_id().await?;
        let kind = match kind.trim() {
            "page" | "pages" => crate::models::ContentKind::Page,
            "post" | "posts" => crate::models::ContentKind::Post,
            _ => return Err(ServerFnError::new("Invalid kind")),
        };

        let pool = pool().await?;
        let items = crate::db::list_content(&pool, kind, include_drafts)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(items
            .into_iter()
            .filter(|c| c.owner_user_id.is_none() || c.owner_user_id == Some(uid))
            .map(content_to_public)
            .collect())
    }
}

#[server(CreateContent, "/api")]
pub async fn create_content(
    kind: String,
    title: String,
    slug: String,
    content: String,
    template: Option<String>,
) -> Result<ContentPublic, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (kind, title, slug, content, template);
        return Err(ServerFnError::new("CreateContent is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let uid = require_user_id().await?;
        let kind = match kind.trim() {
            "page" | "pages" => crate::models::ContentKind::Page,
            "post" | "posts" => crate::models::ContentKind::Post,
            _ => return Err(ServerFnError::new("Invalid kind")),
        };
        if title.trim().is_empty() {
            return Err(ServerFnError::new("Title is required"));
        }
        if slug.trim().is_empty() {
            return Err(ServerFnError::new("Slug is required"));
        }

        let pool = pool().await?;
        let created = crate::db::create_content(
            &pool,
            &crate::models::ContentCreate {
                owner_user_id: Some(uid),
                kind,
                title: title.trim().to_string(),
                slug: slug.trim().to_string(),
                content,
                template: template
                    .as_deref()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "default".to_string()),
            },
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(content_to_public(created))
    }
}

#[server(UpdateContent, "/api")]
pub async fn update_content(
    id: String,
    title: Option<String>,
    slug: Option<String>,
    content: Option<String>,
    template: Option<String>,
    status: Option<String>,
) -> Result<ContentPublic, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (id, title, slug, content, template, status);
        return Err(ServerFnError::new("UpdateContent is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let uid = require_user_id().await?;
        let id = Uuid::parse_str(id.trim()).map_err(|_| ServerFnError::new("Invalid id"))?;
        let pool = pool().await?;

        let existing = crate::db::get_content_by_id(&pool, id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .ok_or_else(|| ServerFnError::new("Not found"))?;
        if existing.owner_user_id.is_some_and(|owner| owner != uid) {
            return Err(ServerFnError::new("Forbidden"));
        }

        let status = match status.as_deref().map(|s| s.trim()) {
            Some("draft") => Some(crate::models::ContentStatus::Draft),
            Some("published") => Some(crate::models::ContentStatus::Published),
            Some("") | None => None,
            Some(_) => return Err(ServerFnError::new("Invalid status")),
        };

        let updated = crate::db::update_content(
            &pool,
            id,
            &crate::models::ContentUpdate {
                title: title.as_ref().map(|s| s.trim().to_string()),
                slug: slug.as_ref().map(|s| s.trim().to_string()),
                content,
                template: template.as_ref().map(|s| s.trim().to_string()),
                status,
            },
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;

        Ok(content_to_public(updated))
    }
}

#[server(PublishContent, "/api")]
pub async fn publish_content(id: String) -> Result<ContentPublic, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        return Err(ServerFnError::new("PublishContent is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let uid = require_user_id().await?;
        let id = Uuid::parse_str(id.trim()).map_err(|_| ServerFnError::new("Invalid id"))?;
        let pool = pool().await?;

        let existing = crate::db::get_content_by_id(&pool, id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .ok_or_else(|| ServerFnError::new("Not found"))?;
        if existing.owner_user_id.is_some_and(|owner| owner != uid) {
            return Err(ServerFnError::new("Forbidden"));
        }

        let published = crate::db::publish_content(&pool, id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .ok_or_else(|| ServerFnError::new("Not found"))?;

        Ok(content_to_public(published))
    }
}

/// List templates visible to the current user (built-in + owned).
#[server(ListTemplates, "/api")]
pub async fn list_templates() -> Result<Vec<TemplateSummary>, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        return Err(ServerFnError::new("ListTemplates is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let uid = require_user_id().await?;
        let pool = pool().await?;
        let templates = crate::db::list_site_templates_for_user(&pool, uid)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .into_iter()
            .map(template_to_summary)
            .collect();
        Ok(templates)
    }
}

#[server(GetTemplate, "/api")]
pub async fn get_template(id: String) -> Result<TemplateDetail, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        return Err(ServerFnError::new("GetTemplate is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let uid = require_user_id().await?;
        let id = Uuid::parse_str(id.trim()).map_err(|_| ServerFnError::new("Invalid id"))?;
        let pool = pool().await?;
        let t = crate::db::get_site_template_by_id(&pool, id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .ok_or_else(|| ServerFnError::new("Not found"))?;

        if !t.is_builtin && t.owner_user_id != Some(uid) {
            return Err(ServerFnError::new("Forbidden"));
        }

        Ok(template_to_detail(t))
    }
}

#[server(CreateTemplate, "/api")]
pub async fn create_template(
    name: String,
    description: Option<String>,
    html: String,
) -> Result<TemplateDetail, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (name, description, html);
        return Err(ServerFnError::new("CreateTemplate is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let uid = require_user_id().await?;
        if name.trim().is_empty() {
            return Err(ServerFnError::new("Name is required"));
        }
        if html.trim().is_empty() {
            return Err(ServerFnError::new("HTML is required"));
        }

        let pool = pool().await?;
        let created = crate::db::create_site_template(
            &pool,
            &crate::models::SiteTemplateCreate {
                owner_user_id: uid,
                name: name.trim().to_string(),
                description: description.unwrap_or_default(),
                html,
            },
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(template_to_detail(created))
    }
}

#[server(UpdateTemplate, "/api")]
pub async fn update_template(
    id: String,
    name: Option<String>,
    description: Option<String>,
    html: Option<String>,
) -> Result<TemplateDetail, ServerFnError> {
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (id, name, description, html);
        return Err(ServerFnError::new("UpdateTemplate is only available on the server"));
    }

    #[cfg(feature = "ssr")]
    {
        let uid = require_user_id().await?;
        let id = Uuid::parse_str(id.trim()).map_err(|_| ServerFnError::new("Invalid id"))?;
        let pool = pool().await?;

        let existing = crate::db::get_site_template_by_id(&pool, id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .ok_or_else(|| ServerFnError::new("Not found"))?;
        if existing.is_builtin {
            return Err(ServerFnError::new("Built-in templates are read-only"));
        }
        if existing.owner_user_id != Some(uid) {
            return Err(ServerFnError::new("Forbidden"));
        }

        let updated = crate::db::update_site_template(
            &pool,
            id,
            &crate::models::SiteTemplateUpdate {
                name: name.as_ref().map(|s| s.trim().to_string()),
                description,
                html,
            },
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("Not found"))?;

        Ok(template_to_detail(updated))
    }
}
