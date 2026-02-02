use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
    Page,
    Post,
}

impl ContentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Post => "post",
        }
    }
}

impl Default for ContentKind {
    fn default() -> Self {
        Self::Post
    }
}

impl std::fmt::Display for ContentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq<&str> for ContentKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl std::str::FromStr for ContentKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "page" => Ok(Self::Page),
            "post" => Ok(Self::Post),
            _ => Err(format!("invalid content kind: {}", s)),
        }
    }
}
