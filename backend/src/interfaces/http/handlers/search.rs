use axum::{
    Json,
    extract::{Query, State},
};
use std::sync::Arc;

use crate::{
    app_state::AppState,
    domain::search::DEFAULT_SEARCH_INDEX,
    errors::Result,
    interfaces::http::dtos::{SearchParams, SearchResponse},
};

/// Number of search results to display per page
///
/// This constant controls the pagination size for search results.
/// A smaller value (6) is used to provide a responsive user experience
/// and reduce the initial load time.
const PAGE_ITEMS: usize = 6;

/// HTTP handler for full-text search on articles
///
/// This endpoint performs full-text search across articles using Meilisearch.
/// Results include highlighting and are paginated for optimal performance.
///
/// # Request Format
///
/// ```text
/// GET /search?q=rust+programming&page=1
/// ```
///
/// # Query Parameters
///
/// * `q` - Search query string (required)
///   - Supports full-text search with relevance ranking
///   - Empty or whitespace-only queries return no results
/// * `page` - Page number, 1-based indexing (optional, default: 1)
///
/// # Arguments
///
/// * `State(state)` - Shared application state containing services
/// * `Query(params)` - Query parameters extracted from the URL
///
/// # Returns
///
/// * `Ok(Json<SearchResponse>)` - Search results with metadata
/// * `Err(SomeError)` - Search service or other error occurred
///
/// # Response Format
///
/// ```json
/// {
///   "total_hits": 42,
///   "total_pages": 7,
///   "current_page": 1,
///   "results": [
///     {
///       "id": "article-1",
///       "title": "Introduction to <span class=\"highlight\">Rust</span>",
///       "category": "article",
///       "summary": "Learn <span class=\"highlight\">Rust</span> basics...",
///       "content": "..."
///     }
///   ]
/// }
/// ```
///
/// # Example Request
///
/// ```bash
/// curl "http://localhost:8124/api/search?q=rust&page=1"
/// ```
pub async fn get_search_results(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>> {
    // Return empty results for empty or whitespace-only queries
    // This prevents unnecessary search operations and maintains clean UX
    if params.q.trim().is_empty() {
        return Ok(Json(SearchResponse {
            total_hits: 0,
            total_pages: 0,
            current_page: params.page,
            results: vec![],
        }));
    }

    // Execute search through the article service
    // Returns results with highlighting, total counts, and pagination info
    let (results, total_hits, total_pages, current_page) = state
        .article_service
        .search(DEFAULT_SEARCH_INDEX, &params.q, params.page, PAGE_ITEMS)
        .await?;

    // Construct and return the JSON search response
    Ok(Json(SearchResponse {
        total_hits,
        total_pages,
        current_page,
        results,
    }))
}
