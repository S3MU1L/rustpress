use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    sqlx::Type,
)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum HomepageType {
    #[default]
    Posts,
    Page,
}

impl HomepageType {
    pub fn validate(
        typ: Self,
        page_id: Option<Uuid>,
    ) -> Result<(), String> {
        match (typ, page_id) {
            (Self::Posts, Some(_)) => Err("homepage_page_id must be NULL when homepage_type=posts".into()),
            (Self::Page, None) => Err("homepage_page_id is required when homepage_type=page".into()),
            _ => Ok(()),
        }
    }
}

impl std::fmt::Display for HomepageType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Posts => write!(f, "posts"),
            Self::Page => write!(f, "page"),
        }
    }
}

impl std::str::FromStr for HomepageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "posts" => Ok(Self::Posts),
            "page" => Ok(Self::Page),
            _ => Err(format!("invalid homepage type: {}", s)),
        }
    }
}
