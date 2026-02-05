use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserWithRoles {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub roles: String, // comma-separated role names from SQL
}

pub async fn user_is_admin(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
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

pub async fn assign_role(
    pool: &PgPool,
    user_id: Uuid,
    role_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id)
        SELECT $1, r.id FROM roles r WHERE r.name = $2
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(role_name)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_role(
    pool: &PgPool,
    user_id: Uuid,
    role_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM user_roles
        WHERE user_id = $1
          AND role_id = (SELECT id FROM roles WHERE name = $2)
        "#,
    )
    .bind(user_id)
    .bind(role_name)
    .execute(pool)
    .await?;

    Ok(())
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
