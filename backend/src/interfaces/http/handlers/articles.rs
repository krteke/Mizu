use axum::{
    Json,
    extract::{Path, Query, State},
};
use std::sync::Arc;

use crate::{
    app_state::AppState,
    domain::articles::{Article, PostParams},
    errors::Result,
    interfaces::http::dtos::PostResponse,
};

/// Maximum number of articles allowed per page
///
/// This limit prevents excessive memory usage and database load from
/// requests asking for too many results at once.
const MAX_PAGE_SIZE: i64 = 100;

/// HTTP handler to retrieve a paginated list of articles by category
///
/// This endpoint returns articles filtered by category with pagination support.
/// Results are ordered by creation date in descending order (newest first).
///
/// # Request Format
///
/// ```text
/// GET /posts?category=article&page=1&page_size=20
/// ```
///
/// # Query Parameters
///
/// * `category` - Filter articles by category (required)
///   - Valid values: "article", "note", "think", "pictures", "talk"
/// * `page` - Page number, 1-based indexing (optional, default: 1)
/// * `page_size` - Number of items per page (optional, default: 20, max: 100)
///
/// # Arguments
///
/// * `Query(params)` - Query parameters extracted from the URL query string
/// * `State(state)` - Shared application state containing services and configuration
///
/// # Returns
///
/// * `Ok(Json<Vec<PostResponse>>)` - JSON array of article summaries
/// * `Err(GetPostsError::CategoryError)` - Invalid category provided
/// * `Err(SomeError)` - Database or other error occurred
///
/// # Response Format
///
/// ```json
/// [
///   {
///     "id": "article-1",
///     "title": "Introduction to Rust",
///     "tags": ["rust", "programming"],
///     "summary": "A beginner's guide to Rust..."
///   },
///   ...
/// ]
/// ```
///
/// # Example Request
///
/// ```bash
/// curl "http://localhost:8124/api/posts?category=article&page=2&page_size=10"
/// ```
pub async fn get_posts(
    Query(params): Query<PostParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PostResponse>>> {
    // Ensure page number is at least 1 (1-based indexing)
    let page = params.page.max(1);

    // Clamp page size between 1 and MAX_PAGE_SIZE to prevent abuse
    let page_size = params.page_size.min(MAX_PAGE_SIZE).max(1);

    // Calculate database offset for pagination
    // Example: page 1 with size 20 → offset 0
    //          page 2 with size 20 → offset 20
    let offset = (page - 1) * page_size;

    // Convert category enum to string for database query
    let category = params.category.as_str();

    // Fetch articles from the service layer
    let query_results = state
        .article_service
        .get_posts_by_category(category, page_size, offset)
        .await?;

    // Wrap the results in JSON response and return
    Ok(Json(query_results))
}

/// HTTP handler to retrieve a single article by its ID
///
/// This endpoint returns the complete article entity including full content.
/// The category parameter in the URL is currently not used but kept for
/// potential future category-based routing or validation.
///
/// # Request Format
///
/// ```text
/// GET /posts/{category}/{id}
/// ```
///
/// # Path Parameters
///
/// * `category` - Article category (currently unused, reserved for future use)
/// * `id` - Unique identifier of the article (required)
///
/// # Arguments
///
/// * `Path((category, id))` - Path parameters extracted from the URL
/// * `State(state)` - Shared application state containing services
///
/// # Returns
///
/// * `Ok(Json<Article>)` - Complete article entity with full content
/// * `Err(GetPostsError::ArticleNotFound)` - Article with given ID doesn't exist
/// * `Err(SomeError)` - Database or other error occurred
///
/// # Response Format
///
/// ```json
/// {
///   "id": "article-1",
///   "title": "Introduction to Rust",
///   "tags": ["rust", "programming"],
///   "category": "article",
///   "summary": "A beginner's guide...",
///   "content": "# Introduction\n\nRust is...",
///   "status": "published",
///   "created_at": "2024-01-15T10:30:00Z",
///   "updated_at": "2024-01-15T10:30:00Z"
/// }
/// ```
///
/// # Example Request
///
/// ```bash
/// curl "http://localhost:8124/api/posts/article/my-first-post"
/// ```
///
/// # Note
///
/// The category path parameter is currently unused. It's kept in the signature
/// for potential future use in category-based validation or routing logic.
pub async fn get_post_digital(
    Path((_category, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Article>> {
    // Fetch the complete article from the service layer
    let result = state.article_service.get_article_by_id(&id).await?;

    // Wrap the article in JSON response and return
    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add integration tests for article handlers
    // - Test pagination with different page sizes
    // - Test invalid category handling
    // - Test article not found scenarios
    // - Test boundary conditions (page 0, negative page size, etc.)
}
