use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::search::SearchService;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_secret: String,
    pub search_service: SearchService,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub category: String,
    pub summary: String,
    pub content: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// {addr}/category/id/title
#[derive(Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub category: String,
    pub summary: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostResponse {
    pub title: String,
    pub tags: Vec<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PostCategory {
    #[serde(rename = "article")]
    Article,
    #[serde(rename = "note")]
    Note,
    #[serde(rename = "think")]
    Think,
    #[serde(rename = "pictures")]
    Pictures,
    #[serde(rename = "talk")]
    Talk,
}

impl PostCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PostCategory::Article => "article",
            PostCategory::Note => "note",
            PostCategory::Pictures => "pictures",
            PostCategory::Talk => "talk",
            PostCategory::Think => "think",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "article" => Some(PostCategory::Article),
            "note" => Some(PostCategory::Note),
            "pictures" => Some(PostCategory::Pictures),
            "talk" => Some(PostCategory::Talk),
            "think" => Some(PostCategory::Think),
            _ => None,
        }
    }
}
