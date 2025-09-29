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
