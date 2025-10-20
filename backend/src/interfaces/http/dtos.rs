use serde::{Deserialize, Serialize};

use crate::domain::search::SearchHit;

/// Data Transfer Object for article list responses
///
/// This DTO is used when returning lists of articles (e.g., from `/posts` endpoint).
/// It contains a subset of article fields, excluding the full content to reduce
/// payload size and improve performance for list views.
///
/// # Fields
///
/// * `id` - Unique identifier of the article
/// * `title` - Article title
/// * `tags` - List of tags associated with the article
/// * `summary` - Brief summary or excerpt of the article
///
/// # Serialization
///
/// This struct is serialized to JSON when sent to clients and deserialized
/// from JSON when received (though typically only used for responses).
///
/// # Example JSON
///
/// ```json
/// {
///   "id": "my-first-post",
///   "title": "My First Post",
///   "tags": ["rust", "programming"],
///   "summary": "This is a brief introduction to Rust programming..."
/// }
/// ```
///
/// # Usage
///
/// ```rust
/// use backend::interfaces::http::dtos::PostResponse;
///
/// let post = PostResponse {
///     id: "article-123".to_string(),
///     title: "Introduction to Rust".to_string(),
///     tags: vec!["rust".to_string(), "tutorial".to_string()],
///     summary: Some("Learn Rust basics...".to_string()),
/// };
/// ```
#[derive(Serialize, Deserialize)]
pub struct PostResponse {
    /// Unique identifier of the article
    pub id: String,

    /// Article title displayed in lists and detail views
    pub title: String,

    /// List of tags for categorization and filtering
    pub tags: Vec<String>,

    /// Brief summary or excerpt of the article content
    /// Used in list views instead of full content
    pub summary: Option<String>,
}

/// Data Transfer Object for search results
///
/// This DTO wraps search results with pagination metadata, providing
/// everything a client needs to display search results and implement
/// pagination UI.
///
/// # Fields
///
/// * `total_hits` - Total number of matching articles across all pages
/// * `total_pages` - Total number of pages based on page size
/// * `current_page` - Current page number (1-based indexing)
/// * `results` - Array of search results with highlighting
///
/// # Pagination
///
/// The pagination information allows clients to:
/// - Display "Page X of Y" indicators
/// - Enable/disable previous/next buttons
/// - Generate page number links
///
/// # Example JSON
///
/// ```json
/// {
///   "total_hits": 42,
///   "total_pages": 7,
///   "current_page": 2,
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
/// # Usage
///
/// ```rust
/// use backend::interfaces::http::dtos::SearchResponse;
///
/// let response = SearchResponse {
///     total_hits: 42,
///     total_pages: 7,
///     current_page: 2,
///     results: vec![],
/// };
/// ```
#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    /// Total number of articles matching the search query
    /// Used to display result counts to users
    pub total_hits: usize,

    /// Total number of pages available for this search
    /// Calculated as: (total_hits + page_size - 1) / page_size
    pub total_pages: usize,

    /// Current page number being displayed (1-based indexing)
    pub current_page: usize,

    /// Array of search results with highlighted matching terms
    /// Each result may contain HTML highlighting tags
    pub results: Vec<SearchHit>,
}

/// Query parameters for search requests
///
/// This DTO captures the query parameters sent by clients when performing
/// a search. It's automatically deserialized from URL query strings by Axum.
///
/// # Fields
///
/// * `q` - Search query string entered by the user
/// * `page` - Requested page number (1-based indexing)
///
/// # Query String Format
///
/// ```text
/// /search?q=rust+programming&page=2
/// ```
///
/// # Validation
///
/// - Empty or whitespace-only queries are handled by returning empty results
/// - Page numbers less than 1 are clamped to 1 in the handler
/// - No maximum query length is enforced at this level
///
/// # Example
///
/// ```rust
/// use backend::interfaces::http::dtos::SearchParams;
/// use axum::extract::Query;
///
/// async fn search_handler(Query(params): Query<SearchParams>) {
///     println!("Searching for: {}", params.q);
///     println!("Page: {}", params.page);
/// }
/// ```
///
/// # URL Encoding
///
/// The query string should be URL-encoded:
/// - Spaces: `%20` or `+`
/// - Special characters: `%XX` format
/// - Example: `rust+web+development` or `rust%20web%20development`
#[derive(Deserialize, Debug)]
pub struct SearchParams {
    /// Search query string (URL-decoded by Axum)
    /// Can contain multiple words, special characters, etc.
    pub q: String,

    /// Requested page number for pagination (1-based indexing)
    /// Defaults to 1 if not specified (handled by handler logic)
    pub page: usize,
}
