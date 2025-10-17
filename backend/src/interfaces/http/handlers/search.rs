use axum::{
    Json,
    extract::{Query, State},
};
use std::sync::Arc;

use crate::{
    app_state::AppState,
    errors::Result,
    interfaces::http::dtos::{SearchParams, SearchResponse},
};

/// Number of search results to display per page
///
/// This constant controls the pagination size for search results.
/// A smaller value (6) is used to provide a responsive user experience
/// and reduce the initial load time.
const PAGE_ITEMS: usize = 6;

/// Default name of the search index in Meilisearch
///
/// This is the index name used for article full-text search operations.
const DEFAULT_SEARCH_INDEX: &str = "articles";

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

/// Create search index and populate it with existing articles
///
/// This handler is used for initial setup or rebuilding the search index
/// from scratch. It creates a new Meilisearch index with specified searchable
/// attributes and imports all articles from the database.
///
/// # Request Format
///
/// ```text
/// POST /search/index
/// ```
///
/// # Arguments
///
/// * `State(state)` - Shared application state containing services
///
/// # Returns
///
/// * `Ok(())` - Index created and populated successfully
/// * `Err(SomeError)` - Index creation or population failed
///
/// # Searchable Attributes
///
/// The following fields are configured as searchable:
/// - `title` - Article title
/// - `content` - Full article content
/// - `summary` - Article summary/excerpt
///
/// # Warning
///
/// This operation can take significant time for large datasets as it:
/// 1. Creates a new search index
/// 2. Fetches all articles from the database
/// 3. Imports all articles into the search index
///
/// # Example Request
///
/// ```bash
/// curl -X POST "http://localhost:8124/api/search/index"
/// ```
pub async fn create_search_index(State(state): State<Arc<AppState>>) -> Result<()> {
    // Define which article fields should be searchable
    // These fields will be indexed and available for full-text search
    let searchable_attributes = ["title", "content", "summary"];

    // Create the index and import all articles from database
    state
        .article_service
        .create_index(DEFAULT_SEARCH_INDEX, searchable_attributes.as_slice())
        .await
}
