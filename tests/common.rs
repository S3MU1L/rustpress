use chrono::{DateTime, Utc};
use uuid::Uuid;

use rustpress::models::*;

const SQL_TIME_FMT: &str = "%Y-%m-%d %H:%M:%S%#z";

pub fn parse_time(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_str(s, SQL_TIME_FMT)
        .expect("Invalid time format in test helper")
        .with_timezone(&Utc)
}

pub fn get_seed_user_0() -> User {
    User {
        id: Uuid::parse_str("00000000-0000-0000-0000-000000000000")
            .unwrap(),
        email: "user0@test.com".to_string(),
        password_hash: "password0".to_string(),
        created_at: parse_time("2026-01-04 22:15:06+00"),
        edited_at: parse_time("2026-01-04 22:15:06+00"),
        deleted_at: None,
    }
}

pub fn get_seed_user_1() -> User {
    User {
        id: Uuid::parse_str("00000000-0000-0000-0000-000000000001")
            .unwrap(),
        email: "user1@test.com".to_string(),
        password_hash: "password1".to_string(),
        created_at: parse_time("2026-01-05 13:22:56+00"),
        edited_at: parse_time("2026-01-05 13:22:56+00"),
        deleted_at: None,
    }
}

pub fn get_seed_user_purged() -> User {
    User {
        id: Uuid::parse_str("00000000-0000-0000-0000-00000000000c")
            .unwrap(),
        email:
            "anon-00000000-0000-0000-0000-00000000000c@deleted.local"
                .to_string(),
        password_hash: "ANONYMIZED".to_string(),
        created_at: parse_time("2026-01-02 18:41:18+00"),
        edited_at: parse_time("2026-01-05 09:35:22+00"),
        deleted_at: Some(parse_time("2026-01-05 09:35:22+00")),
    }
}

pub fn get_seed_user_deleted() -> User {
    User {
        id: Uuid::parse_str("00000000-0000-0000-0000-00000000000a")
            .unwrap(),
        email: "userA@test.com".to_string(),
        password_hash: "passwordA".to_string(),
        created_at: parse_time("2026-01-02 18:41:18+00"),
        edited_at: parse_time("2026-01-05 09:35:22+00"),
        deleted_at: Some(parse_time("2026-01-05 09:35:22+00")),
    }
}

pub fn get_seed_user_nonexisting() -> User {
    User {
        id: Uuid::parse_str("00000000-0000-0000-0000-00000000000d")
            .unwrap(),
        email: "userD@test.com".to_string(),
        password_hash: "passwordD".to_string(),
        created_at: parse_time("2001-11-09 12:46:00+00"),
        edited_at: parse_time("2001-11-09 13:03:00+00"),
        deleted_at: None,
    }
}
