use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use std::time::Duration;

use crate::models::{
    User, UserCreate, UserIden, UserQuery, UserUpdate,
    UserWithOldState,
};

use crate::common::{GeneralError, UserError};
use crate::log_err;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(
        database_url: &str,
    ) -> Result<Self, GeneralError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_user(
        &mut self,
        data: &UserCreate,
    ) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash) VALUES ($1, $2)
            ON CONFLICT (email) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(&data.email)
        .bind(&data.password_hash)
        .fetch_optional(&self.pool)
        .await?;

        match user {
            Some(u) => Ok(u),
            None => {
                Err(UserError::AlreadyExists((&data.email).into()))
            }
        }
    }

    pub async fn get_user(
        &self,
        data: &UserIden,
        include_deleted: bool,
    ) -> Result<User, UserError> {
        let (id, email): (Option<Uuid>, Option<String>) = match data {
            UserIden::Id(id) => (Some(*id), None),
            UserIden::Email(email) => (None, Some(email.clone())),
        };

        let result = sqlx::query_as::<_, User>(
            r#"SELECT * FROM users WHERE id = $1 OR email = $2"#,
        )
        .bind(id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(u) if include_deleted || u.deleted_at.is_none() => {
                Ok(u)
            }
            Some(_) => Err(UserError::AlreadyDeleted(data.clone())),
            None => Err(UserError::NotFound(data.clone())),
        }
    }

    pub async fn get_users(
        &self,
        data: &UserQuery,
        include_deleted: bool,
    ) -> Result<Vec<User>, UserError> {
        if data.is_empty() {
            return Err(UserError::InvalidRequest(
                "No fields provided".into(),
            ));
        }

        if data.limit.is_some_and(|limit| limit < 0) {
            return Err(UserError::InvalidRequest(
                "Pagination 'limit' is negative integer".into(),
            ));
        }

        if data.offset.is_some_and(|offset| offset < 0) {
            return Err(UserError::InvalidRequest(
                "Pagination 'offset' is negative integer".into(),
            ));
        }

        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM users WHERE ");
        let mut separated = query_builder.separated(" AND ");

        if let Some(deleted_at) = &data.deleted_at {
            separated
                .push("deleted_at = ")
                .push_bind_unseparated(deleted_at);
        } else if !include_deleted {
            separated.push("deleted_at IS NULL");
        }

        if let Some(id) = data.id {
            separated.push("id = ").push_bind_unseparated(id);
        }

        if let Some(email) = &data.email {
            separated.push("email = ").push_bind_unseparated(email);
        }

        if let Some(created_at) = &data.created_at {
            separated
                .push("created_at >= ")
                .push_bind_unseparated(created_at);
        }

        if let Some(edited_at) = &data.edited_at {
            separated
                .push("edited_at >= ")
                .push_bind_unseparated(edited_at);
        }

        if let Some(offset) = &data.offset {
            query_builder.push(" OFFSET ").push_bind(offset);
        }

        if let Some(limit) = &data.limit {
            query_builder.push(" LIMIT ").push_bind(limit);
        }

        if let Some(sort_params) = &data.sort_by {
            // Can not have order for offset, limit and sort_by
            if sort_params.len()
                > UserQuery::fields().len().saturating_sub(3)
            {
                return Err(UserError::InvalidRequest(format!(
                    "Sort parameters exceed maximum limit of {}.",
                    UserQuery::fields().len().saturating_sub(3)
                )));
            }

            let active_sorts: Vec<_> = UserQuery::fields()
                .iter()
                .zip(sort_params.iter())
                .filter_map(|(&col, &dir)| {
                    dir.map(|is_asc| (col, is_asc))
                })
                .collect();

            if active_sorts.is_empty() {
                query_builder.push(" ORDER BY created_at DESC ");
            } else {
                query_builder.push(" ORDER BY ");
                let mut separator = query_builder.separated(", ");

                for (col_name, is_asc) in active_sorts {
                    let direction =
                        if is_asc { " ASC" } else { " DESC" };
                    separator
                        .push(format!("{} {}", col_name, direction));
                }
            }
        } else {
            query_builder.push(" ORDER BY created_at DESC ");
        }

        let users = query_builder
            .build_query_as::<User>()
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    pub async fn update_user(
        &mut self,
        data: &UserUpdate,
    ) -> Result<User, UserError> {
        if data.is_empty() {
            return Err(UserError::InvalidRequest(
                "No fields provided".into(),
            ));
        }

        let result = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET
                email = CASE
                    WHEN deleted_at IS NULL THEN COALESCE($1, email)
                    ELSE email
                END,
                password_hash = CASE
                    WHEN deleted_at IS NULL THEN COALESCE($2, password_hash)
                    ELSE password_hash
                END,
                edited_at = CASE
                    WHEN deleted_at IS NULL THEN now()
                    ELSE edited_at
                END
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(data.email.as_deref())
        .bind(data.password_hash.as_deref())
        .bind(data.id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(u)) if u.deleted_at.is_some() => {
                Err(UserError::AlreadyDeleted(data.id.into()))
            }

            Ok(Some(u)) => Ok(u),

            Ok(None) => Err(UserError::NotFound(data.id.into())),

            Err(sqlx::Error::Database(e))
                if e.code().as_deref() == Some("23505") =>
            {
                let err = match &data.email {
                    Some(email) => {
                        UserError::AlreadyExists(email.into())
                    }
                    None => {
                        log_err!(self.pool, data.clone());
                        UserError::Internal
                    }
                };

                Err(err)
            }

            Err(e) => Err(UserError::Database(e)),
        }
    }

    pub async fn soft_delete_user(
        &mut self,
        data: &UserIden,
    ) -> Result<User, UserError> {
        let (id, email): (Option<Uuid>, Option<String>) = match data {
            UserIden::Id(id) => (Some(*id), None),
            UserIden::Email(email) => (None, Some(email.clone())),
        };

        let result = sqlx::query_as::<_, UserWithOldState>(
            r#"
            WITH old_state AS (
                SELECT id, edited_at FROM users
                WHERE id = $1 OR email = $2
            ),
            updated_state AS (
                UPDATE users
                SET
                    edited_at = CASE
                        WHEN deleted_at IS NULL THEN now()
                        ELSE edited_at
                    END,
                    deleted_at = CASE
                        WHEN deleted_at IS NULL THEN now()
                        ELSE deleted_at
                    END
                WHERE id = $1 OR email = $2
                RETURNING *
            )
            SELECT
                u.*,
                os.edited_at AS old_edited_at
            FROM updated_state u
            JOIN old_state os ON u.id = os.id
            "#,
        )
        .bind(id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(u) if u.edited_at != u.old_edited_at => Ok(u.into()),

            Some(_) => Err(UserError::AlreadyDeleted(data.clone())),

            None => Err(UserError::NotFound(data.clone())),
        }
    }

    pub async fn hard_delete_user(
        &mut self,
        data: &UserIden,
    ) -> Result<User, UserError> {
        let (id, email): (Option<Uuid>, Option<String>) = match data {
            UserIden::Id(id) => (Some(*id), None),
            UserIden::Email(email) => (None, Some(email.clone())),
        };

        let user = sqlx::query_as::<_, User>(
            r#"
            DELETE FROM users
            WHERE id = $1 OR email = $2
            RETURNING
                id,
                email,
                password_hash,
                created_at,
                now() as edited_at,
                deleted_at
            "#,
        )
        .bind(id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        match user {
            Some(u) => Ok(u),
            None => Err(UserError::NotFound(data.clone())),
        }
    }

    pub async fn purge_user(
        &mut self,
        data: &UserIden,
    ) -> Result<User, UserError> {
        let (id, email): (Option<Uuid>, Option<String>) = match data {
            UserIden::Id(id) => (Some(*id), None),
            UserIden::Email(email) => (None, Some(email.clone())),
        };

        let result = sqlx::query_as::<_, UserWithOldState>(
            r#"
            WITH old_state AS (
                SELECT id, edited_at FROM users
                WHERE id = $1 OR email = $2
            ),
            updated_state AS (
                UPDATE users
                SET
                    edited_at = CASE
                        WHEN email NOT LIKE 'anon-%@deleted.local' THEN now()
                        ELSE edited_at
                    END,
                    deleted_at = CASE
                        WHEN email NOT LIKE 'anon-%@deleted.local' THEN now()
                        ELSE deleted_at
                    END,
                    email = 'anon-' || id::text || '@deleted.local',
                    password_hash = 'ANONYMIZED'
                WHERE id = $1 OR email = $2
                RETURNING *
            )
            SELECT
                u.*,
                os.edited_at AS old_edited_at
            FROM updated_state u
            JOIN old_state os ON u.id = os.id
            "#,
        )
        .bind(id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(u) if u.edited_at != u.old_edited_at => Ok(u.into()),

            Some(_) => Err(UserError::AlreadyDeleted(data.clone())),

            None => Err(UserError::NotFound(data.clone())),
        }
    }

    pub async fn recover_user(
        &mut self,
        data: &UserIden,
    ) -> Result<User, UserError> {
        let (id, email): (Option<Uuid>, Option<String>) = match data {
            UserIden::Id(id) => (Some(*id), None),
            UserIden::Email(email) => (None, Some(email.clone())),
        };

        let result = sqlx::query_as::<_, UserWithOldState>(
            r#"
            WITH old_state AS (
                SELECT id, edited_at FROM users
                WHERE id = $1 OR email = $2
            ),
            updated_state AS (
                UPDATE users
                SET
                    edited_at = CASE
                        WHEN email NOT LIKE 'anon-%@deleted.local' AND
                             deleted_at IS NOT NULL
                        THEN now()
                        ELSE edited_at
                    END,
                    deleted_at = CASE
                        WHEN email NOT LIKE 'anon-%@deleted.local'
                        THEN NULL
                        ELSE deleted_at
                    END
                WHERE id = $1 OR email = $2
                RETURNING *
            )
            SELECT
                u.*,
                os.edited_at AS old_edited_at
            FROM updated_state u
            JOIN old_state os ON u.id = os.id
            "#,
        )
        .bind(id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(u) if u.edited_at != u.old_edited_at => Ok(u.into()),

            Some(u) if u.has_email_purged() => {
                Err(UserError::IrreversiblyDeleted(data.clone()))
            }

            Some(_) => Err(UserError::AlreadyExists(data.clone())),

            None => Err(UserError::NotFound(data.clone())),
        }
    }
}
