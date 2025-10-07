use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{
    common::{AppState, PostResponse},
    some_errors::{GetPostsError, Result},
};

#[derive(Deserialize, Clone)]
pub struct PostParams {
    category: String,
}

pub async fn get_posts(
    Query(params): Query<PostParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PostResponse>>> {
    let valid_categories = ["article", "note", "think", "pictures", "talk"];

    if !valid_categories.contains(&params.category.as_str()) {
        return Err(GetPostsError::CategoryError.into());
    }

    let pool = &state.db_pool;
    let query_results = sqlx::query_as!(
        PostResponse,
        "SELECT title, tags, content FROM articles WHERE category = $1",
        &params.category
    )
    .fetch_all(pool)
    .await?;

    Ok(Json(query_results))
}
