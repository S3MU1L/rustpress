use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Site, SiteCreate, SiteUpdate};

pub async fn list_sites_for_user(
    pool: &PgPool,
    owner_user_id: Uuid,
    query: Option<&str>,
) -> Result<Vec<Site>, sqlx::Error> {
    let q = query.map(str::trim).filter(|s| !s.is_empty());

    if let Some(q) = q {
        let pattern = format!("%{}%", q);
        sqlx::query_as::<_, Site>(
            r#"
            SELECT *
            FROM sites
            WHERE owner_user_id = $1
              AND (name ILIKE $2 OR slug ILIKE $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_user_id)
        .bind(pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Site>(
            r#"
            SELECT *
            FROM sites
            WHERE owner_user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_user_id)
        .fetch_all(pool)
        .await
    }
}

pub async fn get_site_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Site>, sqlx::Error> {
    sqlx::query_as::<_, Site>(
        r#"
        SELECT *
        FROM sites
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create_site(pool: &PgPool, data: &SiteCreate) -> Result<Site, sqlx::Error> {
    sqlx::query_as::<_, Site>(
        r#"
        INSERT INTO sites (owner_user_id, name, slug, status, default_template)
        VALUES ($1, $2, $3, 'draft', $4)
        RETURNING *
        "#,
    )
    .bind(data.owner_user_id)
    .bind(&data.name)
    .bind(&data.slug)
    .bind(&data.default_template)
    .fetch_one(pool)
    .await
}

pub async fn update_site(pool: &PgPool, id: Uuid, data: &SiteUpdate) -> Result<Option<Site>, sqlx::Error> {
    sqlx::query_as::<_, Site>(
        r#"
        UPDATE sites
        SET
            name = COALESCE($1, name),
            slug = COALESCE($2, slug),
            status = COALESCE($3, status),
            default_template = COALESCE($4, default_template),
            edited_at = now()
        WHERE id = $5
        RETURNING *
        "#,
    )
    .bind(data.name.as_deref())
    .bind(data.slug.as_deref())
    .bind(data.status.as_deref())
    .bind(data.default_template.as_deref())
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn publish_site(pool: &PgPool, id: Uuid) -> Result<Option<Site>, sqlx::Error> {
    let now = Utc::now();
    sqlx::query_as::<_, Site>(
        r#"
        UPDATE sites
        SET
            status = 'published',
            published_at = COALESCE(published_at, $1),
            edited_at = now()
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(now)
    .bind(id)
    .fetch_optional(pool)
    .await
}
