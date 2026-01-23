mod common;

#[cfg(test)]
pub mod user_tests {
    use chrono::{DateTime, TimeZone, Utc};
    use uuid::Uuid;

    use super::common::*;

    use rustpress::common::*;
    use rustpress::models::*;

    #[test]
    fn test_user_has_email_purged_success() {
        let user = get_seed_user_purged();
        assert!(user.has_email_purged());
    }

    #[test]
    fn test_user_has_email_purged_fails_on_active_email() {
        let user = get_seed_user_0();
        assert!(!user.has_email_purged());
    }

    #[test]
    fn test_user_has_email_purged_fails_on_deleted_email() {
        let user = get_seed_user_deleted();
        assert!(!user.has_email_purged());
    }

    #[test]
    fn test_user_from_user_with_old_state_success() {}

    #[test]
    fn test_user_update_is_empty_success() {
        let user_update = UserUpdate {
            id: Uuid::new_v4(),
            email: None,
            password_hash: None,
        };

        assert!(user_update.is_empty());
    }

    #[test]
    fn test_user_update_is_empty_fails_on_nonempty_email() {
        let user_update = UserUpdate {
            id: Uuid::new_v4(),
            email: Some("user@test.com".into()),
            password_hash: None,
        };

        assert!(!user_update.is_empty());
    }

    #[test]
    fn test_user_update_is_empty_fails_on_nonempty_password_hash() {
        let user_update = UserUpdate {
            id: Uuid::new_v4(),
            email: None,
            password_hash: Some("password".into()),
        };

        assert!(!user_update.is_empty());
    }

    #[test]
    fn test_user_query_is_empty_success_on_default() {
        let user_query = UserQuery::default();
        assert!(user_query.is_empty());
    }

    #[test]
    fn test_user_query_is_empty_success_on_non_required() {
        let user_query = UserQuery {
            offset: Some(100),
            ..Default::default()
        };

        assert!(
            user_query.is_empty(),
            "Parameter \'offset\' is only controls pagination, thus non required"
        );

        let user_query = UserQuery {
            limit: Some(100),
            ..Default::default()
        };

        assert!(
            user_query.is_empty(),
            "Parameter \'limit\' is only controls pagination, thus non required"
        );

        let user_query = UserQuery {
            sort_by: Some(vec![Some(true), None, Some(false)]),
            ..Default::default()
        };

        assert!(
            user_query.is_empty(),
            "Parameter \'sort_by\' is only controls pagination, thus non required"
        );
    }

    #[test]
    fn test_user_query_is_empty_fails_on_nonempty() {
        let user = get_seed_user_deleted();

        let user_query = UserQuery {
            id: Some(user.id),
            ..Default::default()
        };

        assert!(!user_query.is_empty());

        let user_query = UserQuery {
            email: Some(user.email.clone()),
            ..Default::default()
        };

        assert!(!user_query.is_empty());

        let user_query = UserQuery {
            created_at: Some(user.created_at),
            ..Default::default()
        };

        assert!(!user_query.is_empty());

        let user_query = UserQuery {
            edited_at: Some(user.edited_at),
            ..Default::default()
        };

        assert!(!user_query.is_empty());

        let user_query = UserQuery {
            deleted_at: user.deleted_at,
            ..Default::default()
        };

        assert!(!user_query.is_empty());

        let user_query = UserQuery {
            id: Some(user.id),
            email: None,
            created_at: Some(user.created_at),
            edited_at: None,
            deleted_at: user.deleted_at,
            ..Default::default()
        };

        assert!(!user_query.is_empty());

        let user_query = UserQuery {
            id: Some(user.id),
            offset: Some(100),
            ..Default::default()
        };

        assert!(!user_query.is_empty());
    }

    #[test]
    fn test_user_query_fields_success() {
        let original_fields = UserQuery::fields().clone();

        assert_eq!(original_fields.len(), 8);

        assert!(original_fields.contains(&"id"));
        assert!(original_fields.contains(&"email"));
        assert!(original_fields.contains(&"created_at"));
        assert!(original_fields.contains(&"edited_at"));
        assert!(original_fields.contains(&"deleted_at"));
        assert!(original_fields.contains(&"offset"));
        assert!(original_fields.contains(&"limit"));
        assert!(original_fields.contains(&"sort_by"));

        assert_eq!(original_fields, UserQuery::fields());
    }

    #[test]
    fn test_user_iden_from_uuid_success() {
        let user = get_seed_user_0();
        assert_eq!(UserIden::Id(user.id), UserIden::from(user.id));
    }

    #[test]
    fn test_user_iden_from_string_success() {
        let user = get_seed_user_0();
        assert_eq!(
            UserIden::Email(user.email.clone()),
            UserIden::from(user.email)
        );
    }

    #[test]
    fn test_user_iden_from_ref_string_success() {
        let user = get_seed_user_0();
        assert_eq!(
            UserIden::Email(user.email.clone()),
            UserIden::from(&user.email)
        );

        let _ = user.email;
    }

    #[test]
    fn test_user_iden_from_str_success() {
        let user = get_seed_user_0();
        assert_eq!(
            UserIden::Email(user.email.clone()),
            UserIden::from(user.email.as_str())
        );

        let _ = user.email;
    }
}
