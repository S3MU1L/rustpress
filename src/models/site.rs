use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::HomepageType;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Site {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub default_template: String,
    pub homepage_type: HomepageType,
    pub homepage_page_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub edited_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

impl Site {
    pub fn validate_homepage(&self) -> Result<(), String> {
        HomepageType::validate(self.homepage_type, self.homepage_page_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteCreate {
    pub owner_user_id: Uuid,
    pub name: String,
    pub slug: String,
    pub default_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteUpdate {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub status: Option<String>,
    pub default_template: Option<String>,
    pub homepage_type: Option<HomepageType>,
    pub homepage_page_id: Option<Option<Uuid>>,
}

impl SiteUpdate {
    pub fn validate_homepage(&self) -> Result<(), String> {
        if let Some(typ) = self.homepage_type {
            HomepageType::validate(typ, self.homepage_page_id.flatten())
        } else {
            Ok(())
        }
    }
}
