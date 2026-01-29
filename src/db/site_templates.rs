use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{SiteTemplate, SiteTemplateCreate, SiteTemplateUpdate};

pub async fn list_site_templates(pool: &PgPool) -> Result<Vec<SiteTemplate>, sqlx::Error> {
    sqlx::query_as::<_, SiteTemplate>(
        r#"
        SELECT *
        FROM site_templates
        ORDER BY is_builtin DESC, name ASC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_site_template_by_id(pool: &PgPool, id: Uuid) -> Result<Option<SiteTemplate>, sqlx::Error> {
    sqlx::query_as::<_, SiteTemplate>(
        r#"
        SELECT *
        FROM site_templates
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_site_template_by_name(
    pool: &PgPool,
    name: &str,
) -> Result<Option<SiteTemplate>, sqlx::Error> {
    sqlx::query_as::<_, SiteTemplate>(
        r#"
        SELECT *
        FROM site_templates
        WHERE name = $1
        "#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

pub async fn create_site_template(
    pool: &PgPool,
    data: &SiteTemplateCreate,
) -> Result<SiteTemplate, sqlx::Error> {
    sqlx::query_as::<_, SiteTemplate>(
        r#"
        INSERT INTO site_templates (name, description, html, is_builtin)
        VALUES ($1, $2, $3, false)
        RETURNING *
        "#,
    )
    .bind(&data.name)
    .bind(&data.description)
    .bind(&data.html)
    .fetch_one(pool)
    .await
}

pub async fn update_site_template(
    pool: &PgPool,
    id: Uuid,
    data: &SiteTemplateUpdate,
) -> Result<Option<SiteTemplate>, sqlx::Error> {
    sqlx::query_as::<_, SiteTemplate>(
        r#"
        UPDATE site_templates
        SET
            name = COALESCE($1, name),
            description = COALESCE($2, description),
            html = COALESCE($3, html),
            edited_at = now()
        WHERE id = $4
        RETURNING *
        "#,
    )
    .bind(data.name.as_deref())
    .bind(data.description.as_deref())
    .bind(data.html.as_deref())
    .bind(id)
    .fetch_optional(pool)
    .await
}
