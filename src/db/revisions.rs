use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::{
    ContentItem, ContentItemRevision, ContentItemRevisionMeta,
    ContentStatus,
};

pub async fn ensure_initial_revision(
    pool: &PgPool,
    content_item_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<i32, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let current = lock_current_rev(&mut tx, content_item_id).await?;

    let item = sqlx::query_as::<_, ContentItem>(
        r#"
        SELECT *
        FROM content_items
        WHERE id = $1
        "#,
    )
    .bind(content_item_id)
    .fetch_one(&mut *tx)
    .await?;

    // If rev=1 exists already (e.g. seeded by migrations), do nothing.
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM content_item_revisions
            WHERE content_item_id = $1 AND rev = 1
        )
        "#,
    )
    .bind(content_item_id)
    .fetch_one(&mut *tx)
    .await?;

    if !exists {
        insert_revision_snapshot(&mut tx, &item, 1, actor_user_id)
            .await?;
    }

    // Ensure pointer is sane.
    if current < 1 {
        sqlx::query(
            r#"
            UPDATE content_items
            SET current_rev = 1
            WHERE id = $1
            "#,
        )
        .bind(content_item_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(current.max(1))
}

pub async fn record_revision(
    pool: &PgPool,
    item: &ContentItem,
    actor_user_id: Option<Uuid>,
) -> Result<i32, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Lock first to prevent race conditions
    let mut current = lock_current_rev(&mut tx, item.id).await?;

    let rev1_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM content_item_revisions
            WHERE content_item_id = $1 AND rev = 1
        )
        "#,
    )
    .bind(item.id)
    .fetch_one(&mut *tx)
    .await?;

    if !rev1_exists {
        insert_revision_snapshot(&mut tx, item, 1, actor_user_id)
            .await?;
        sqlx::query(
            r#"
            UPDATE content_items
            SET current_rev = 1
            WHERE id = $1
            "#,
        )
        .bind(item.id)
        .execute(&mut *tx)
        .await?;
        current = 1;
    }

    let max_rev = max_rev(&mut tx, item.id).await?;

    if current < max_rev {
        // Truncate redo history when recording after an undo.
        sqlx::query(
            r#"
            DELETE FROM content_item_revisions
            WHERE content_item_id = $1 AND rev > $2
            "#,
        )
        .bind(item.id)
        .bind(current)
        .execute(&mut *tx)
        .await?;
    }

    let next = current.saturating_add(1);
    if next == current {
        return Err(sqlx::Error::Protocol(
            "Revision limit reached (i32::MAX)".into(),
        ));
    }

    insert_revision_snapshot(&mut tx, item, next, actor_user_id)
        .await?;

    sqlx::query(
        r#"
        UPDATE content_items
        SET current_rev = $1
        WHERE id = $2
        "#,
    )
    .bind(next)
    .bind(item.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(next)
}

pub async fn list_revisions(
    pool: &PgPool,
    content_item_id: Uuid,
    limit: i64,
) -> Result<Vec<ContentItemRevisionMeta>, sqlx::Error> {
    sqlx::query_as::<_, ContentItemRevisionMeta>(
        r#"
        SELECT
            rev,
            created_by_user_id,
            created_at,
            title,
            status
        FROM content_item_revisions
        WHERE content_item_id = $1
        ORDER BY rev DESC
        LIMIT $2
        "#,
    )
    .bind(content_item_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn get_revision(
    pool: &PgPool,
    content_item_id: Uuid,
    rev_num: i32,
) -> Result<Option<ContentItemRevision>, sqlx::Error> {
    sqlx::query_as::<_, ContentItemRevision>(
        r#"
        SELECT *
        FROM content_item_revisions
        WHERE content_item_id = $1 AND rev = $2
        "#,
    )
    .bind(content_item_id)
    .bind(rev_num)
    .fetch_optional(pool)
    .await
}

pub async fn restore_revision(
    pool: &PgPool,
    content_item_id: Uuid,
    rev_num: i32,
) -> Result<Option<ContentItem>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let item =
        restore_revision_in_tx(&mut tx, content_item_id, rev_num)
            .await?;
    tx.commit().await?;
    Ok(item)
}

async fn restore_revision_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    content_item_id: Uuid,
    rev_num: i32,
) -> Result<Option<ContentItem>, sqlx::Error> {
    // Lock the content_items row first
    let _current = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT current_rev
        FROM content_items
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(content_item_id)
    .fetch_optional(&mut **tx)
    .await?;

    if _current.is_none() {
        return Ok(None);
    }

    let rev = sqlx::query_as::<
        _,
        (String, String, String, String, ContentStatus),
    >(
        r#"
        SELECT title, slug, content, template, status
        FROM content_item_revisions
        WHERE content_item_id = $1 AND rev = $2
        "#,
    )
    .bind(content_item_id)
    .bind(rev_num)
    .fetch_optional(&mut **tx)
    .await?;

    let Some((title, slug, content, template, status)) = rev else {
        return Ok(None);
    };

    // Keep published_at if present; restoring a draft shouldn't implicitly clear it.
    let item = sqlx::query_as::<_, ContentItem>(
        r#"
        UPDATE content_items
        SET
            title = $1,
            slug = $2,
            content = $3,
            template = $4,
            status = $5,
            edited_at = now(),
            current_rev = $6
        WHERE id = $7
        RETURNING *
        "#,
    )
    .bind(title)
    .bind(slug)
    .bind(content)
    .bind(template)
    .bind(status.as_str())
    .bind(rev_num)
    .bind(content_item_id)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(item)
}

pub async fn undo(
    pool: &PgPool,
    content_item_id: Uuid,
) -> Result<Option<ContentItem>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let current = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT current_rev
        FROM content_items
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(content_item_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(current) = current else {
        return Ok(None);
    };

    // Go to previous revision, or stay at 1 if already there
    let target_rev = if current <= 1 { 1 } else { current - 1 };

    let item =
        restore_revision_in_tx(&mut tx, content_item_id, target_rev)
            .await?;
    tx.commit().await?;
    Ok(item)
}

pub async fn redo(
    pool: &PgPool,
    content_item_id: Uuid,
) -> Result<Option<ContentItem>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let current = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT current_rev
        FROM content_items
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(content_item_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(current) = current else {
        return Ok(None);
    };

    // Check if next revision exists
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM content_item_revisions
            WHERE content_item_id = $1 AND rev = $2
        )
        "#,
    )
    .bind(content_item_id)
    .bind(current + 1)
    .fetch_one(&mut *tx)
    .await?;

    // Go to next revision if it exists, otherwise stay at current
    let target_rev = if exists { current + 1 } else { current };

    let item =
        restore_revision_in_tx(&mut tx, content_item_id, target_rev)
            .await?;
    tx.commit().await?;
    Ok(item)
}

async fn lock_current_rev(
    tx: &mut Transaction<'_, Postgres>,
    content_item_id: Uuid,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>(
        r#"
        SELECT current_rev
        FROM content_items
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(content_item_id)
    .fetch_one(&mut **tx)
    .await
}

async fn max_rev(
    tx: &mut Transaction<'_, Postgres>,
    content_item_id: Uuid,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COALESCE(MAX(rev), 0)
        FROM content_item_revisions
        WHERE content_item_id = $1
        "#,
    )
    .bind(content_item_id)
    .fetch_one(&mut **tx)
    .await
}

async fn insert_revision_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    item: &ContentItem,
    rev_num: i32,
    actor_user_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO content_item_revisions (
            content_item_id,
            rev,
            title,
            slug,
            content,
            template,
            status,
            created_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(item.id)
    .bind(rev_num)
    .bind(&item.title)
    .bind(&item.slug)
    .bind(&item.content)
    .bind(&item.template)
    .bind(item.status.as_str())
    .bind(actor_user_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
