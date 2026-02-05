mod common;

#[cfg(test)]
pub mod db_tests {
    use chrono::Utc;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::common::*;

    use rustpress::common::*;
    use rustpress::db::*;
    use rustpress::models::*;

    const TIME_LIMIT: i64 = 1;

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_migration_integrity(pool: PgPool) {
        let user0 = get_seed_user_0();
        let user1 = get_seed_user_1();
        let user_purged = get_seed_user_purged();
        let user_deleted = get_seed_user_deleted();
        let user_nonexisting = get_seed_user_nonexisting();

        let fetch_user = |target_id: Uuid| {
            let pool = pool.clone();

            async move {
                sqlx::query_as!(
                    User,
                    r#"
                    SELECT id, email, password_hash, email_verified_at, created_at, edited_at, deleted_at
                    FROM users
                    WHERE id = $1
                    "#,
                    target_id
                )
                .fetch_optional(&pool)
                .await
                .expect("Failed database query")
            }
        };

        let fetched_0 = fetch_user(user0.id).await;
        assert_eq!(fetched_0, Some(user0));

        let fetched_1 = fetch_user(user1.id).await;
        assert_eq!(fetched_1, Some(user1));

        let fetched_p = fetch_user(user_purged.id).await;
        assert_eq!(fetched_p, Some(user_purged));

        let fetched_d = fetch_user(user_deleted.id).await;
        assert_eq!(fetched_d, Some(user_deleted));

        let fetched_n = fetch_user(user_nonexisting.id).await;
        assert!(fetched_n.is_none());
    }

    async fn test_database_new(pool: PgPool) {
        todo!();
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_from_pool(pool: PgPool) {
        let db = Database::from_pool(pool.clone());

        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&db.pool)
            .await
            .expect("Failed to query database");

        assert_eq!(row.0, 1);

        pool.close().await;

        let result = sqlx::query_as::<_, (i32,)>("SELECT 1")
            .fetch_one(&db.pool)
            .await;

        assert!(
            result.is_err(),
            "The struct's pool should be closed because it shares the underlying instance"
        );
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_add_user_success(pool: PgPool) {
        let mut db = Database::from_pool(pool);

        let new_user = UserCreate {
            email: "test_new@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
        };

        let start = Utc::now();
        let user = db
            .add_user(&new_user)
            .await
            .expect("Failed to add user to database");
        let end = Utc::now();

        assert_eq!(user.email, new_user.email);
        assert_eq!(user.password_hash, new_user.password_hash);
        assert_eq!(user.deleted_at, None);

        assert_eq!(
            user.created_at, user.edited_at,
            "New users should have synced timestamps"
        );

        assert!(
            user.created_at >= start && user.created_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Insert was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_add_user_fails_on_duplicate_email(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let existing_email = get_seed_user_0().email;

        let new_user = UserCreate {
            email: existing_email.clone(),
            password_hash: "hashed_password".to_string(),
        };

        let start = Utc::now();
        let result = db.add_user(&new_user).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyExists(UserIden::Email(email)))
                if email == existing_email
            ),
            "Should return AlreadyExists error with duplicated email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Insert was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_update_user_email_success(pool: PgPool) {
        let mut db = Database::from_pool(pool);

        let original_user = get_seed_user_0();

        let update_data = UserUpdate {
            id: original_user.id,
            email: Some("new_email@test.com".into()),
            password_hash: None,
        };

        let start = Utc::now();
        let updated_user = db
            .update_user(&update_data)
            .await
            .expect("Failed to update user email");
        let end = Utc::now();

        assert_eq!(updated_user.id, original_user.id);
        assert_eq!(updated_user.email, "new_email@test.com");
        assert_eq!(
            updated_user.password_hash,
            original_user.password_hash
        );
        assert_eq!(updated_user.deleted_at, None);

        assert_eq!(updated_user.created_at, original_user.created_at);

        assert_ne!(
            updated_user.created_at, updated_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            updated_user.edited_at >= start
                && updated_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Update was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_update_user_password_success(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_0();

        let update_data = UserUpdate {
            id: original_user.id,
            email: None,
            password_hash: Some("new_secret_hash".into()),
        };

        let start = Utc::now();
        let updated_user = db
            .update_user(&update_data)
            .await
            .expect("Failed to update password");
        let end = Utc::now();

        assert_eq!(updated_user.id, original_user.id);
        assert_eq!(updated_user.email, original_user.email);
        assert_eq!(updated_user.password_hash, "new_secret_hash");
        assert_eq!(updated_user.deleted_at, None);

        assert_eq!(updated_user.created_at, original_user.created_at);
        assert_ne!(
            updated_user.created_at, updated_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            updated_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            updated_user.edited_at >= start
                && updated_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Update was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_update_user_fails_on_duplicate_email(pool: PgPool) {
        let mut db = Database::from_pool(pool);

        let target_id = get_seed_user_1().id;
        let existing_email = get_seed_user_0().email;

        let start = Utc::now();
        let result = db
            .update_user(&UserUpdate {
                id: target_id,
                email: Some(existing_email.clone()),
                password_hash: None,
            })
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyExists(UserIden::Email(email)))
                if email == existing_email
            ),
            "Should return AlreadyExists error with duplicated email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Update was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_update_user_fails_when_all_fields_none(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let user = get_seed_user_0();

        let start = Utc::now();
        let result = db
            .update_user(&UserUpdate {
                id: user.id,
                email: None,
                password_hash: None,
            })
            .await;
        let end = Utc::now();

        assert!(
            matches!(result, Err(UserError::InvalidRequest(_))),
            "Should return InvalidRequest when no fields are provided"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Update was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_update_user_fails_if_deleted(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let deleted_user = get_seed_user_deleted();

        let start = Utc::now();
        let result = db
            .update_user(&UserUpdate {
                id: deleted_user.id,
                email: Some("revive@test.com".into()),
                password_hash: None,
            })
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Id(id)))
                if id == deleted_user.id
            ),
            "Should return AlreadyDeleted error with correct ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Update was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_update_user_fails_if_purged(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let deleted_user = get_seed_user_purged();

        let start = Utc::now();
        let result = db
            .update_user(&UserUpdate {
                id: deleted_user.id,
                email: Some("revive@test.com".into()),
                password_hash: None,
            })
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Id(id)))
                if id == deleted_user.id
            ),
            "Should return AlreadyDeleted error with correct ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Update was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_update_user_fails_if_not_found(pool: PgPool) {
        let mut db = Database::from_pool(pool);

        let target_id = get_seed_user_nonexisting().id;

        let start = Utc::now();
        let result = db
            .update_user(&UserUpdate {
                id: target_id,
                email: Some("ghost@test.com".into()),
                password_hash: None,
            })
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::NotFound(UserIden::Id(id))) if id == target_id
            ),
            "Should return NotFound for non-existent ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Update was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_soft_delete_user_success_by_id(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_0();

        let start = Utc::now();
        let deleted_user = db
            .soft_delete_user(&UserIden::Id(original_user.id))
            .await
            .expect("Failed to soft delete user from database");
        let end = Utc::now();

        assert_eq!(deleted_user.id, original_user.id);
        assert_eq!(deleted_user.email, original_user.email);
        assert_eq!(
            deleted_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(deleted_user.created_at, original_user.created_at);
        assert_ne!(
            deleted_user.created_at, deleted_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            deleted_user.deleted_at,
            Some(deleted_user.edited_at)
        );
        assert!(
            deleted_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            deleted_user.edited_at >= start
                && deleted_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Soft delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_soft_delete_user_success_by_email(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_0();

        let start = Utc::now();
        let deleted_user = db
            .soft_delete_user(&UserIden::Email(
                original_user.email.clone(),
            ))
            .await
            .expect("Failed to soft delete user from database");
        let end = Utc::now();

        assert_eq!(deleted_user.id, original_user.id);
        assert_eq!(deleted_user.email, original_user.email);
        assert_eq!(
            deleted_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(deleted_user.created_at, original_user.created_at);
        assert_ne!(
            deleted_user.created_at, deleted_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            deleted_user.deleted_at,
            Some(deleted_user.edited_at)
        );
        assert!(
            deleted_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            deleted_user.edited_at >= start
                && deleted_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Soft delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_soft_delete_user_fails_if_not_found_by_id(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_id = get_seed_user_nonexisting().id;

        let start = Utc::now();
        let result =
            db.soft_delete_user(&UserIden::Id(target_id)).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::NotFound(UserIden::Id(id)))
                if id == target_id
            ),
            "Should return NotFound for non-existent ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Soft delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_soft_delete_user_fails_if_not_found_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_email = get_seed_user_nonexisting().email;

        let start = Utc::now();
        let result = db
            .soft_delete_user(&UserIden::Email(target_email.clone()))
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::NotFound(UserIden::Email(email)))
                if email == target_email
            ),
            "Should return NotFound for non-existent email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Soft delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_soft_delete_user_fails_if_deleted_by_id(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_id = get_seed_user_deleted().id;

        let start = Utc::now();
        let result =
            db.soft_delete_user(&UserIden::Id(target_id)).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Id(id)))
                if id == target_id
            ),
            "Should return AlreadyDeleted error with correct ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Soft delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_soft_delete_user_fails_if_deleted_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_email = get_seed_user_deleted().email;

        let start = Utc::now();
        let result = db
            .soft_delete_user(&UserIden::Email(target_email.clone()))
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Email(email)))
                if email == target_email
            ),
            "Should return AlreadyDeleted error with correct email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Soft delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_soft_delete_user_fails_if_purged_by_id(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_id = get_seed_user_purged().id;

        let start = Utc::now();
        let result =
            db.soft_delete_user(&UserIden::Id(target_id)).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Id(id)))
                if id == target_id
            ),
            "Should return AlreadyDeleted error with correct ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Soft delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_soft_delete_user_fails_if_purged_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_email = get_seed_user_purged().email;

        let start = Utc::now();
        let result = db
            .soft_delete_user(&UserIden::Email(target_email.clone()))
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Email(email)))
                if email == target_email
            ),
            "Should return AlreadyDeleted error with correct email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Soft delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_hard_delete_user_success_on_active_by_id(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool.clone());
        let original_user = get_seed_user_0();

        let start = Utc::now();
        let deleted_user = db
            .hard_delete_user(&UserIden::Id(original_user.id))
            .await
            .expect("Failed to hard delete user from database");
        let end = Utc::now();

        assert_eq!(deleted_user.id, original_user.id);
        assert_eq!(deleted_user.email, original_user.email);
        assert_eq!(
            deleted_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(deleted_user.created_at, original_user.created_at);
        assert_ne!(
            deleted_user.created_at, deleted_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            deleted_user.deleted_at, None,
            "Parameter \'deleted_at\' is used only as soft delete flag"
        );

        assert!(
            deleted_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            deleted_user.edited_at >= start
                && deleted_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let result = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password_hash, email_verified_at, created_at, edited_at, deleted_at
                FROM users
                WHERE id = $1
                "#,
            deleted_user.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Failed database query");

        assert!(
            result.is_none(),
            "User should be permanently removed from database"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Hard delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_hard_delete_user_success_on_active_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool.clone());
        let original_user = get_seed_user_0();

        let start = Utc::now();
        let deleted_user = db
            .hard_delete_user(&UserIden::Email(
                original_user.email.clone(),
            ))
            .await
            .expect("Failed to hard delete user from database");
        let end = Utc::now();

        assert_eq!(deleted_user.id, original_user.id);
        assert_eq!(deleted_user.email, original_user.email);
        assert_eq!(
            deleted_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(deleted_user.created_at, original_user.created_at);
        assert_ne!(
            deleted_user.created_at, deleted_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            deleted_user.deleted_at, None,
            "Parameter \'deleted_at\' is used only as soft delete flag"
        );

        assert!(
            deleted_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            deleted_user.edited_at >= start
                && deleted_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let result = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password_hash, email_verified_at, created_at, edited_at, deleted_at
                FROM users
                WHERE id = $1
                "#,
            deleted_user.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Failed database query");

        assert!(
            result.is_none(),
            "User should be permanently removed from database"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Hard delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_hard_delete_user_success_on_deleted_by_id(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool.clone());
        let original_user = get_seed_user_deleted();

        let start = Utc::now();
        let deleted_user = db
            .hard_delete_user(&UserIden::Id(original_user.id))
            .await
            .expect("Failed to hard delete user from database");
        let end = Utc::now();

        assert_eq!(deleted_user.id, original_user.id);
        assert_eq!(deleted_user.email, original_user.email);
        assert_eq!(
            deleted_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(deleted_user.created_at, original_user.created_at);
        assert_ne!(
            deleted_user.created_at, deleted_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            deleted_user.deleted_at, original_user.deleted_at,
            "Parameter \'deleted_at\' is used only as soft delete flag"
        );

        assert!(
            deleted_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            deleted_user.edited_at >= start
                && deleted_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let result = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password_hash, email_verified_at, created_at, edited_at, deleted_at
                FROM users
                WHERE id = $1
                "#,
            deleted_user.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Failed database query");

        assert!(
            result.is_none(),
            "User should be permanently removed from database"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Hard delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_hard_delete_user_success_on_deleted_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool.clone());
        let original_user = get_seed_user_deleted();

        let start = Utc::now();
        let deleted_user = db
            .hard_delete_user(&UserIden::Email(
                original_user.email.clone(),
            ))
            .await
            .expect("Failed to hard delete user from database");
        let end = Utc::now();

        assert_eq!(deleted_user.id, original_user.id);
        assert_eq!(deleted_user.email, original_user.email);
        assert_eq!(
            deleted_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(deleted_user.created_at, original_user.created_at);
        assert_ne!(
            deleted_user.created_at, deleted_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            deleted_user.deleted_at, original_user.deleted_at,
            "Parameter \'deleted_at\' is used only as soft delete flag"
        );

        assert!(
            deleted_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            deleted_user.edited_at >= start
                && deleted_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let result = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password_hash, email_verified_at, created_at, edited_at, deleted_at
                FROM users
                WHERE id = $1
                "#,
            deleted_user.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Failed database query");

        assert!(
            result.is_none(),
            "User should be permanently removed from database"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Hard delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_hard_delete_user_success_on_purged_by_id(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool.clone());
        let original_user = get_seed_user_purged();

        let start = Utc::now();
        let deleted_user = db
            .hard_delete_user(&UserIden::Id(original_user.id))
            .await
            .expect("Failed to hard delete user from database");
        let end = Utc::now();

        assert_eq!(deleted_user.id, original_user.id);
        assert_eq!(deleted_user.email, original_user.email);
        assert_eq!(
            deleted_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(deleted_user.created_at, original_user.created_at);
        assert_ne!(
            deleted_user.created_at, deleted_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            deleted_user.deleted_at, original_user.deleted_at,
            "Parameter \'deleted_at\' is used only as soft delete flag"
        );

        assert!(
            deleted_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            deleted_user.edited_at >= start
                && deleted_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let result = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password_hash, email_verified_at, created_at, edited_at, deleted_at
                FROM users
                WHERE id = $1
                "#,
            deleted_user.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Failed database query");

        assert!(
            result.is_none(),
            "User should be permanently removed from database"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Hard delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_hard_delete_user_success_on_purged_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool.clone());
        let original_user = get_seed_user_deleted();

        let start = Utc::now();
        let deleted_user = db
            .hard_delete_user(&UserIden::Email(
                original_user.email.clone(),
            ))
            .await
            .expect("Failed to hard delete user from database");
        let end = Utc::now();

        assert_eq!(deleted_user.id, original_user.id);
        assert_eq!(deleted_user.email, original_user.email);
        assert_eq!(
            deleted_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(deleted_user.created_at, original_user.created_at);
        assert_ne!(
            deleted_user.created_at, deleted_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            deleted_user.deleted_at, original_user.deleted_at,
            "Parameter \'deleted_at\' is used only as soft delete flag"
        );

        assert!(
            deleted_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            deleted_user.edited_at >= start
                && deleted_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let result = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password_hash, email_verified_at, created_at, edited_at, deleted_at
                FROM users
                WHERE id = $1
                "#,
            deleted_user.id
        )
        .fetch_optional(&pool)
        .await
        .expect("Failed database query");

        assert!(
            result.is_none(),
            "User should be permanently removed from database"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Hard delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_hard_delete_user_fails_if_not_found_by_id(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_id = get_seed_user_nonexisting().id;

        let start = Utc::now();
        let result =
            db.hard_delete_user(&UserIden::Id(target_id)).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::NotFound(UserIden::Id(id)))
                if id == target_id
            ),
            "Should return NotFound for non-existent ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Hard delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_hard_delete_user_fails_if_not_found_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_email = get_seed_user_nonexisting().email;

        let start = Utc::now();
        let result = db
            .hard_delete_user(&UserIden::Email(target_email.clone()))
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::NotFound(UserIden::Email(email)))
                if email == target_email
            ),
            "Should return NotFound for non-existent email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Hard delete was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_purge_user_success_on_active_by_id(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_0();

        let start = Utc::now();
        let purged_user = db
            .purge_user(&UserIden::Id(original_user.id))
            .await
            .expect("Failed to purge user from database");
        let end = Utc::now();

        assert_eq!(purged_user.id, original_user.id);
        assert_eq!(purged_user.created_at, original_user.created_at);

        assert_ne!(purged_user.email, original_user.email);
        assert!(
            purged_user.has_email_purged(),
            "Parameter \'email\' should have been censored"
        );

        assert_ne!(
            purged_user.password_hash,
            original_user.password_hash
        );
        assert_eq!(
            purged_user.password_hash, "ANONYMIZED",
            "Parameter \'password_hash\' should have been censored"
        );

        assert_ne!(
            purged_user.created_at, purged_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            purged_user.deleted_at,
            Some(purged_user.edited_at)
        );
        assert!(
            purged_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            purged_user.edited_at >= start
                && purged_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Purge was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_purge_user_success_on_active_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_0();

        let start = Utc::now();
        let purged_user = db
            .purge_user(&UserIden::Email(original_user.email.clone()))
            .await
            .expect("Failed to purge user from database");
        let end = Utc::now();

        assert_eq!(purged_user.id, original_user.id);
        assert_eq!(purged_user.created_at, original_user.created_at);

        assert_ne!(purged_user.email, original_user.email);
        assert!(
            purged_user.has_email_purged(),
            "Parameter \'email\' should have been censored"
        );

        assert_ne!(
            purged_user.password_hash,
            original_user.password_hash
        );
        assert_eq!(
            purged_user.password_hash, "ANONYMIZED",
            "Parameter \'password_hash\' should have been censored"
        );

        assert_ne!(
            purged_user.created_at, purged_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            purged_user.deleted_at,
            Some(purged_user.edited_at)
        );
        assert!(
            purged_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            purged_user.edited_at >= start
                && purged_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Purge was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_purge_user_success_on_deleted_by_id(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_deleted();

        let start = Utc::now();
        let purged_user = db
            .purge_user(&UserIden::Id(original_user.id))
            .await
            .expect("Failed to purge user from database");
        let end = Utc::now();

        assert_eq!(purged_user.id, original_user.id);
        assert_eq!(purged_user.created_at, original_user.created_at);

        assert_ne!(purged_user.email, original_user.email);
        assert!(
            purged_user.has_email_purged(),
            "Parameter \'email\' should have been censored"
        );

        assert_ne!(
            purged_user.password_hash,
            original_user.password_hash
        );
        assert_eq!(
            purged_user.password_hash, "ANONYMIZED",
            "Parameter \'password_hash\' should have been censored"
        );

        assert_ne!(
            purged_user.created_at, purged_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            purged_user.deleted_at,
            Some(purged_user.edited_at)
        );
        assert!(
            purged_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            purged_user.edited_at >= start
                && purged_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Purge was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_purge_user_success_on_deleted_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_deleted();

        let start = Utc::now();
        let purged_user = db
            .purge_user(&UserIden::Email(original_user.email.clone()))
            .await
            .expect("Failed to purge user from database");
        let end = Utc::now();

        assert_eq!(purged_user.id, original_user.id);
        assert_eq!(purged_user.created_at, original_user.created_at);

        assert_ne!(purged_user.email, original_user.email);
        assert!(
            purged_user.has_email_purged(),
            "Parameter \'email\' should have been censored"
        );

        assert_ne!(
            purged_user.password_hash,
            original_user.password_hash
        );
        assert_eq!(
            purged_user.password_hash, "ANONYMIZED",
            "Parameter \'password_hash\' should have been censored"
        );

        assert_ne!(
            purged_user.created_at, purged_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(
            purged_user.deleted_at,
            Some(purged_user.edited_at)
        );
        assert!(
            purged_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            purged_user.edited_at >= start
                && purged_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Purge was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_purge_user_fails_on_purged_by_id(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_purged();

        let start = Utc::now();
        let result =
            db.purge_user(&UserIden::Id(original_user.id)).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Id(id)))
                if id == original_user.id
            ),
            "Should return AlreadyDeleted with correct ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Purge was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_purge_user_fails_on_purged_by_email(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_purged();

        let start = Utc::now();
        let result = db
            .purge_user(&UserIden::Email(original_user.email.clone()))
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Email(email)))
                if email == original_user.email
            ),
            "Should return AlreadyDeleted with correct email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Purge was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_purge_user_fails_if_not_found_by_id(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let target_id = get_seed_user_nonexisting().id;

        let start = Utc::now();
        let result = db.purge_user(&UserIden::Id(target_id)).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::NotFound(UserIden::Id(id)))
                if id == target_id
            ),
            "Should return NotFound for non-existent ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Purge was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_purge_user_fails_if_not_found_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_email = get_seed_user_nonexisting().email;

        let start = Utc::now();
        let result = db
            .purge_user(&UserIden::Email(target_email.clone()))
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::NotFound(UserIden::Email(email)))
                if email == target_email
            ),
            "Should return NotFound for non-existent email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Purge was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn recover_user_success_by_id(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_deleted();

        let start = Utc::now();
        let recovered_user = db
            .recover_user(&UserIden::Id(original_user.id))
            .await
            .expect("Failed to recover deleted user from database");
        let end = Utc::now();

        assert_eq!(recovered_user.id, original_user.id);
        assert_eq!(recovered_user.email, original_user.email);
        assert_eq!(
            recovered_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(
            recovered_user.created_at,
            original_user.created_at
        );
        assert_ne!(
            recovered_user.created_at, recovered_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(recovered_user.deleted_at, None);

        assert!(
            recovered_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            recovered_user.edited_at >= start
                && recovered_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Recovery was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn recover_user_success_by_email(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let original_user = get_seed_user_deleted();

        let start = Utc::now();
        let recovered_user = db
            .recover_user(&UserIden::Email(
                original_user.email.clone(),
            ))
            .await
            .expect("Failed to recover deleted user from database");
        let end = Utc::now();

        assert_eq!(recovered_user.id, original_user.id);
        assert_eq!(recovered_user.email, original_user.email);
        assert_eq!(
            recovered_user.password_hash,
            original_user.password_hash
        );

        assert_eq!(
            recovered_user.created_at,
            original_user.created_at
        );
        assert_ne!(
            recovered_user.created_at, recovered_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert_eq!(recovered_user.deleted_at, None);

        assert!(
            recovered_user.edited_at > original_user.edited_at,
            "Parameter \'edited_at\' should have updated"
        );

        assert!(
            recovered_user.edited_at >= start
                && recovered_user.edited_at <= end,
            "Timestamp outside test window"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Recovery was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_recover_user_fails_if_not_found_by_id(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_id = get_seed_user_nonexisting().id;

        let start = Utc::now();
        let result = db.recover_user(&UserIden::Id(target_id)).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::NotFound(UserIden::Id(id)))
                if id == target_id
            ),
            "Should return NotFound for non-existent ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Recovery was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_recover_user_fails_if_not_found_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_email = get_seed_user_nonexisting().email;

        let start = Utc::now();
        let result = db
            .recover_user(&UserIden::Email(target_email.clone()))
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::NotFound(UserIden::Email(email)))
                if email == target_email
            ),
            "Should return NotFound for non-existent email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Recovery was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_recover_user_fails_on_active_by_id(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let target_id = get_seed_user_0().id;

        let start = Utc::now();
        let result = db.recover_user(&UserIden::Id(target_id)).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyExists(UserIden::Id(id)))
                if id == target_id
            ),
            "Should return AlreadyExists for active ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Recovery was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_recover_user_fails_on_active_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_email = get_seed_user_0().email;

        let start = Utc::now();
        let result = db
            .recover_user(&UserIden::Email(target_email.clone()))
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyExists(UserIden::Email(email)))
                if email == target_email
            ),
            "Should return AlreadyExists for active email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Recovery was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_recover_user_fails_on_purged_by_id(pool: PgPool) {
        let mut db = Database::from_pool(pool);
        let target_id = get_seed_user_purged().id;

        let start = Utc::now();
        let result = db.recover_user(&UserIden::Id(target_id)).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::IrreversiblyDeleted(UserIden::Id(id)))
                if id == target_id
            ),
            "Should return IrreversiblyDeleted for purged ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Recovery was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_recover_user_fails_on_purged_by_email(
        pool: PgPool,
    ) {
        let mut db = Database::from_pool(pool);
        let target_email = get_seed_user_purged().email;

        let start = Utc::now();
        let result = db
            .recover_user(&UserIden::Email(target_email.clone()))
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::IrreversiblyDeleted(UserIden::Email(email)))
                if email == target_email
            ),
            "Should return IrreversiblyDeleted for purged email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Recovery was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_user_success_active_by_id(pool: PgPool) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let user = get_seed_user_0();

            let start = Utc::now();
            let result = db
                .get_user(&UserIden::Id(user.id), include_deleted)
                .await
                .expect("Failed to load user from database");
            let end = Utc::now();

            assert_eq!(result, user);

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get user was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_user_success_active_by_email(pool: PgPool) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let user = get_seed_user_0();

            let start = Utc::now();
            let result = db
                .get_user(
                    &UserIden::Email(user.email.clone()),
                    include_deleted,
                )
                .await
                .expect("Failed to load user from database");
            let end = Utc::now();

            assert_eq!(result, user);

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get user was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_user_success_deleted_by_id(pool: PgPool) {
        let db = Database::from_pool(pool);
        let user = get_seed_user_deleted();

        let start = Utc::now();
        let result = db
            .get_user(&UserIden::Id(user.id), true)
            .await
            .expect("Failed to load user from database");
        let end = Utc::now();

        assert_eq!(result, user);

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get user was too slow");

        let start = Utc::now();
        let result = db.get_user(&UserIden::Id(user.id), false).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Id(id)))
                if id == user.id
            ),
            "Should return AlreadyExists error with deleted ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get user was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_user_success_deleted_by_email(pool: PgPool) {
        let db = Database::from_pool(pool);
        let user = get_seed_user_deleted();

        let start = Utc::now();
        let result = db
            .get_user(&UserIden::Email(user.email.clone()), true)
            .await
            .expect("Failed to load user from database");
        let end = Utc::now();

        assert_eq!(result, user);

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get user was too slow");

        let start = Utc::now();
        let result = db
            .get_user(&UserIden::Email(user.email.clone()), false)
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Email(email)))
                if email == user.email
            ),
            "Should return AlreadyExists error with deleted email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get user was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_user_success_purged_by_id(pool: PgPool) {
        let db = Database::from_pool(pool);
        let user = get_seed_user_purged();

        let start = Utc::now();
        let result = db
            .get_user(&UserIden::Id(user.id), true)
            .await
            .expect("Failed to load user from database");
        let end = Utc::now();

        assert_eq!(result, user);

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get user was too slow");

        let start = Utc::now();
        let result = db.get_user(&UserIden::Id(user.id), false).await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Id(id)))
                if id == user.id
            ),
            "Should return AlreadyExists error with deleted ID"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get user was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_user_success_purged_by_email(pool: PgPool) {
        let db = Database::from_pool(pool);
        let user = get_seed_user_purged();

        let start = Utc::now();
        let result = db
            .get_user(&UserIden::Email(user.email.clone()), true)
            .await
            .expect("Failed to load user from database");
        let end = Utc::now();

        assert_eq!(result, user);

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get user was too slow");

        let start = Utc::now();
        let result = db
            .get_user(&UserIden::Email(user.email.clone()), false)
            .await;
        let end = Utc::now();

        assert!(
            matches!(
                result,
                Err(UserError::AlreadyDeleted(UserIden::Email(email)))
                if email == user.email
            ),
            "Should return AlreadyExists error with deleted email"
        );

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get user was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_user_fails_if_not_found_by_id(pool: PgPool) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let target_id = get_seed_user_nonexisting().id;

            let start = Utc::now();
            let result = db
                .get_user(&UserIden::Id(target_id), include_deleted)
                .await;
            let end = Utc::now();

            assert!(
                matches!(
                    result,
                    Err(UserError::NotFound(UserIden::Id(id)))
                    if id == target_id
                ),
                "Should return AlreadyExists error with duplicated email"
            );

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get user was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_user_fails_if_not_found_by_email(pool: PgPool) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let target_email = get_seed_user_nonexisting().email;

            let start = Utc::now();
            let result = db
                .get_user(
                    &UserIden::Email(target_email.clone()),
                    include_deleted,
                )
                .await;
            let end = Utc::now();

            assert!(
                matches!(
                    result,
                    Err(UserError::NotFound(UserIden::Email(email)))
                    if email == target_email
                ),
                "Should return AlreadyExists error with duplicated email"
            );

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get user was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_users_success_active_on_id(pool: PgPool) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let user = get_seed_user_0();

            let start = Utc::now();
            let users = db
                .get_users(
                    &UserQuery {
                        id: Some(user.id),
                        ..Default::default()
                    },
                    include_deleted,
                )
                .await
                .expect("Failed to get users from database");
            let end = Utc::now();

            assert_eq!(users.len(), 1);
            assert_eq!(users[0], user);

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get users was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_users_success_active_on_email(pool: PgPool) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let user = get_seed_user_0();

            let start = Utc::now();
            let users = db
                .get_users(
                    &UserQuery {
                        email: Some(user.email.clone()),
                        ..Default::default()
                    },
                    include_deleted,
                )
                .await
                .expect("Failed to get users from database");
            let end = Utc::now();

            assert_eq!(users.len(), 1);
            assert_eq!(users[0], user);

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get users was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_users_success_deleted_on_id(pool: PgPool) {
        let db = Database::from_pool(pool);
        let user = get_seed_user_deleted();

        let start = Utc::now();
        let users = db
            .get_users(
                &UserQuery {
                    id: Some(user.id),
                    ..Default::default()
                },
                true,
            )
            .await
            .expect("Failed to get users from database");
        let end = Utc::now();

        assert_eq!(users.len(), 1);
        assert_eq!(users[0], user);

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get users was too slow");

        let start = Utc::now();
        let users = db
            .get_users(
                &UserQuery {
                    id: Some(user.id),
                    ..Default::default()
                },
                false,
            )
            .await
            .expect("Failed to get users from database");
        let end = Utc::now();

        assert_eq!(users.len(), 0);

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get users was too slow");
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_users_success_deleted_on_email(pool: PgPool) {
        let db = Database::from_pool(pool);
        let user = get_seed_user_deleted();

        let start = Utc::now();
        let users = db
            .get_users(
                &UserQuery {
                    email: Some(user.email.clone()),
                    ..Default::default()
                },
                true,
            )
            .await
            .expect("Failed to get users from database");
        let end = Utc::now();

        assert_eq!(users.len(), 1);
        assert_eq!(users[0], user);

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get users was too slow");

        let start = Utc::now();
        let users = db
            .get_users(
                &UserQuery {
                    email: Some(user.email.clone()),
                    ..Default::default()
                },
                false,
            )
            .await
            .expect("Failed to get users from database");
        let end = Utc::now();

        assert_eq!(users.len(), 0);

        let duration = (end - start).num_seconds();
        assert!(duration < TIME_LIMIT, "Get users was too slow");
    }
    //
    // #[sqlx::test(migrations = "./tests/migrations")]
    // async fn test_get_users_success_on_created_at(pool: PgPool) {
    //     todo!();
    // }
    //
    // #[sqlx::test(migrations = "./tests/migrations")]
    // async fn test_get_users_success_on_edited_at(pool: PgPool) {
    //     todo!();
    // }
    //
    // #[sqlx::test(migrations = "./tests/migrations")]
    // async fn test_get_users_success_on_deleted_at(pool: PgPool) {
    //     todo!();
    // }
    //
    // #[sqlx::test(migrations = "./tests/migrations")]
    // async fn test_get_users_success_on_sort_by(pool: PgPool) {
    //     todo!();
    // }
    //
    // #[sqlx::test(migrations = "./tests/migrations")]
    // async fn test_get_users_success_mix(pool: PgPool) {
    //     // offset, limit and multiple values
    //     todo!();
    // }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_users_failes_when_all_fields_none(
        pool: PgPool,
    ) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let start = Utc::now();
            let result = db
                .get_users(&UserQuery::default(), include_deleted)
                .await;
            let end = Utc::now();

            assert!(
                matches!(
                    result,
                    Err(UserError::InvalidRequest(msg))
                    if msg.contains("fields")
                ),
                "Should return InvalidRequest error"
            );

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get users was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_users_failes_on_negative_limit(pool: PgPool) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let user = get_seed_user_0();

            let start = Utc::now();
            let result = db
                .get_users(
                    &UserQuery {
                        id: Some(user.id),
                        limit: Some(-1),
                        ..Default::default()
                    },
                    include_deleted,
                )
                .await;
            let end = Utc::now();

            assert!(
                matches!(
                    result,
                    Err(UserError::InvalidRequest(msg))
                    if msg.contains("limit")
                ),
                "Should return InvalidRequest error"
            );

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get users was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_users_failes_on_negative_offset(pool: PgPool) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let user = get_seed_user_0();

            let start = Utc::now();
            let result = db
                .get_users(
                    &UserQuery {
                        id: Some(user.id),
                        offset: Some(-1),
                        ..Default::default()
                    },
                    include_deleted,
                )
                .await;
            let end = Utc::now();

            assert!(
                matches!(
                    result,
                    Err(UserError::InvalidRequest(msg))
                    if msg.contains("offset")
                ),
                "Should return InvalidRequest error"
            );

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get users was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }

    #[sqlx::test(migrations = "./tests/migrations")]
    async fn test_get_users_failes_on_sort_by_count(pool: PgPool) {
        let db = Database::from_pool(pool);

        async fn run_test(db: &Database, include_deleted: bool) {
            let user = get_seed_user_0();

            let start = Utc::now();
            let result = db
                .get_users(
                    &UserQuery {
                        id: Some(user.id),
                        sort_by: Some(vec![None; 25]),
                        ..Default::default()
                    },
                    include_deleted,
                )
                .await;
            let end = Utc::now();

            assert!(
                matches!(
                    result,
                    Err(UserError::InvalidRequest(msg))
                    if msg.contains("maximum limit")
                ),
                "Should return InvalidRequest error"
            );

            let duration = (end - start).num_seconds();
            assert!(duration < TIME_LIMIT, "Get users was too slow");
        }

        run_test(&db, true).await;
        run_test(&db, false).await;
    }
}
