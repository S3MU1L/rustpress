use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{RoleName, User};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserWithRoles {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub roles: String, // comma-separated role names from SQL
}

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        ON CONFLICT (email) DO NOTHING
        RETURNING *
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .fetch_optional(pool)
    .await
}

pub async fn count_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(r#"SELECT COUNT(*) FROM users"#)
        .fetch_one(pool)
        .await
}

pub async fn count_admins(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT ur.user_id)
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id
        WHERE r.name = 'admin'
        "#,
    )
    .fetch_one(pool)
    .await
}

pub async fn user_is_admin(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM user_roles ur
            JOIN roles r ON r.id = ur.role_id
            WHERE ur.user_id = $1 AND r.name = 'admin'
        )
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn get_user_role_names(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT r.name
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id
        WHERE ur.user_id = $1
        ORDER BY r.name ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Set the user's role. Each user has exactly one role (admin or editor).
/// This replaces any existing role.
pub async fn set_user_role(
    pool: &PgPool,
    user_id: Uuid,
    role: RoleName,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"DELETE FROM user_roles WHERE user_id = $1"#)
        .bind(user_id)
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id)
        SELECT $1, r.id FROM roles r WHERE r.name = $2
        "#,
    )
    .bind(user_id)
    .bind(role.as_str())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_user_email(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE users SET email = $1, edited_at = now() WHERE id = $2"#,
    )
    .bind(email)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_user_password(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE users SET password_hash = $1, edited_at = now() WHERE id = $2"#,
    )
    .bind(password_hash)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn soft_delete_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE users SET deleted_at = now(), edited_at = now() WHERE id = $1"#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_email_map(
    pool: &PgPool,
    ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, String>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = sqlx::query_as::<_, (Uuid, String)>(
        r#"SELECT id, email FROM users WHERE id = ANY($1)"#,
    )
    .bind(ids)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().collect())
}

pub async fn list_all_users_with_roles(
    pool: &PgPool,
) -> Result<Vec<UserWithRoles>, sqlx::Error> {
    let rows = sqlx::query_as::<_, UserWithRoles>(
        r#"
        SELECT
            u.id,
            u.email,
            u.created_at,
            u.deleted_at,
            COALESCE(string_agg(r.name, ', ' ORDER BY r.name), '') AS roles
        FROM users u
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        GROUP BY u.id, u.email, u.created_at, u.deleted_at
        ORDER BY u.created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
