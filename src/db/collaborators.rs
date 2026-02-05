use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ContentCollaborator, ContentItem, ContentKind, RoleName,
};

pub async fn can_view_content(
    pool: &PgPool,
    item: &ContentItem,
    uid: Uuid,
) -> Result<bool, sqlx::Error> {
    if item.owner_user_id.is_none() {
        return Ok(true);
    }

    if item.owner_user_id == Some(uid) {
        return Ok(true);
    }

    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM content_item_collaborators
            WHERE content_item_id = $1 AND user_id = $2
        )
        "#,
    )
    .bind(item.id)
    .bind(uid)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

pub async fn can_edit_content(
    pool: &PgPool,
    item: &ContentItem,
    uid: Uuid,
) -> Result<bool, sqlx::Error> {
    if item.owner_user_id.is_none() {
        return Ok(true);
    }

    if item.owner_user_id == Some(uid) {
        return Ok(true);
    }

    let role = sqlx::query_scalar::<_, Option<RoleName>>(
        r#"
        SELECT role
        FROM content_item_collaborators
        WHERE content_item_id = $1 AND user_id = $2
        "#,
    )
    .bind(item.id)
    .bind(uid)
    .fetch_optional(pool)
    .await?
    .flatten();

    Ok(role == Some(RoleName::Editor))
}

pub async fn can_manage_collaborators(
    _pool: &PgPool,
    item: &ContentItem,
    uid: Uuid,
) -> Result<bool, sqlx::Error> {
    Ok(item.owner_user_id.is_some_and(|owner| owner == uid))
}

pub async fn list_content_for_user(
    pool: &PgPool,
    kind: ContentKind,
    include_drafts: bool,
    uid: Uuid,
) -> Result<Vec<ContentItem>, sqlx::Error> {
    if include_drafts {
        sqlx::query_as::<_, ContentItem>(
            r#"
            SELECT DISTINCT c.*
            FROM content_items c
            LEFT JOIN content_item_collaborators col
              ON col.content_item_id = c.id
             AND col.user_id = $2
            WHERE c.kind = $1
              AND (c.owner_user_id IS NULL OR c.owner_user_id = $2 OR col.user_id IS NOT NULL)
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(kind.as_str())
        .bind(uid)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, ContentItem>(
            r#"
            SELECT DISTINCT c.*
            FROM content_items c
            LEFT JOIN content_item_collaborators col
              ON col.content_item_id = c.id
             AND col.user_id = $2
            WHERE c.kind = $1
              AND c.status = 'published'
              AND (c.owner_user_id IS NULL OR c.owner_user_id = $2 OR col.user_id IS NOT NULL)
            ORDER BY c.published_at DESC NULLS LAST, c.created_at DESC
            "#,
        )
        .bind(kind.as_str())
        .bind(uid)
        .fetch_all(pool)
        .await
    }
}

pub async fn list_collaborators(
    pool: &PgPool,
    content_item_id: Uuid,
) -> Result<Vec<ContentCollaborator>, sqlx::Error> {
    sqlx::query_as::<_, ContentCollaborator>(
        r#"
        SELECT
            col.content_item_id,
            col.user_id,
            u.email,
            col.role,
            col.created_at
        FROM content_item_collaborators col
        JOIN users u ON u.id = col.user_id
        WHERE col.content_item_id = $1
        ORDER BY u.email ASC
        "#,
    )
    .bind(content_item_id)
    .fetch_all(pool)
    .await
}

pub async fn add_collaborator(
    pool: &PgPool,
    content_item_id: Uuid,
    email: &str,
    role: RoleName,
    invited_by_user_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    let user_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO content_item_collaborators (content_item_id, user_id, role, invited_by_user_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (content_item_id, user_id)
        DO UPDATE SET role = EXCLUDED.role
        "#,
    )
    .bind(content_item_id)
    .bind(user_id)
    .bind(role)
    .bind(invited_by_user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_collaborator_role(
    pool: &PgPool,
    content_item_id: Uuid,
    user_id: Uuid,
    role: RoleName,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE content_item_collaborators
        SET role = $1
        WHERE content_item_id = $2 AND user_id = $3
        "#,
    )
    .bind(role)
    .bind(content_item_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_collaborator(
    pool: &PgPool,
    content_item_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM content_item_collaborators
        WHERE content_item_id = $1 AND user_id = $2
        "#,
    )
    .bind(content_item_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
