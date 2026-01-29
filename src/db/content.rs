use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{ContentCreate, ContentItem, ContentKind, ContentStatus, ContentUpdate};

pub async fn create_content(pool: &PgPool, data: &ContentCreate) -> Result<ContentItem, sqlx::Error> {
    sqlx::query_as::<_, ContentItem>(
        r#"
        INSERT INTO content_items (owner_user_id, kind, status, title, slug, content, template)
        VALUES ($1, $2, 'draft', $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(data.owner_user_id)
    .bind(data.kind.as_str())
    .bind(&data.title)
    .bind(&data.slug)
    .bind(&data.content)
    .bind(&data.template)
    .fetch_one(pool)
    .await
}

pub async fn list_content(
    pool: &PgPool,
    kind: ContentKind,
    include_drafts: bool,
) -> Result<Vec<ContentItem>, sqlx::Error> {
    if include_drafts {
        sqlx::query_as::<_, ContentItem>(
            r#"
            SELECT *
            FROM content_items
            WHERE kind = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(kind.as_str())
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, ContentItem>(
            r#"
            SELECT *
            FROM content_items
            WHERE kind = $1 AND status = 'published'
            ORDER BY published_at DESC NULLS LAST, created_at DESC
            "#,
        )
        .bind(kind.as_str())
        .fetch_all(pool)
        .await
    }
}

pub async fn get_content_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ContentItem>, sqlx::Error> {
    sqlx::query_as::<_, ContentItem>(
        r#"
        SELECT *
        FROM content_items
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_published_by_slug(
    pool: &PgPool,
    kind: ContentKind,
    slug: &str,
) -> Result<Option<ContentItem>, sqlx::Error> {
    sqlx::query_as::<_, ContentItem>(
        r#"
        SELECT *
        FROM content_items
        WHERE kind = $1 AND slug = $2 AND status = 'published'
        "#,
    )
    .bind(kind.as_str())
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn update_content(pool: &PgPool, id: Uuid, data: &ContentUpdate) -> Result<Option<ContentItem>, sqlx::Error> {
    sqlx::query_as::<_, ContentItem>(
        r#"
        UPDATE content_items
        SET
            title = COALESCE($1, title),
            slug = COALESCE($2, slug),
            content = COALESCE($3, content),
            template = COALESCE($4, template),
            status = COALESCE($5, status),
            edited_at = now()
        WHERE id = $6
        RETURNING *
        "#,
    )
    .bind(data.title.as_deref())
    .bind(data.slug.as_deref())
    .bind(data.content.as_deref())
    .bind(data.template.as_deref())
    .bind(data.status.as_ref().map(ContentStatus::as_str))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn publish_content(pool: &PgPool, id: Uuid) -> Result<Option<ContentItem>, sqlx::Error> {
    let now = Utc::now();
    sqlx::query_as::<_, ContentItem>(
        r#"
        UPDATE content_items
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
