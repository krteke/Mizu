use async_trait::async_trait;
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};

use crate::{domain::articles::PostCategory, errors::Result};

/// Default name of the search index in Meilisearch
///
/// This is the index name used for article full-text search operations.
pub const DEFAULT_SEARCH_INDEX: &str = "articles";

/// Search result item containing an article match with optional highlighting
///
/// This struct represents a single search result returned by the search service.
/// Fields may contain HTML highlighting tags (e.g., `<span class="highlight">`)
/// to indicate matching terms in the search query.
///
/// # Fields
///
/// * `id` - Unique identifier of the matched article
/// * `title` - Article title (may contain highlighting)
/// * `category` - Category classification of the article
/// * `summary` - Article summary with possible highlighting and truncation
/// * `content` - Article content excerpt with highlighting and truncation
///
/// # Example
///
/// ```rust
/// use backend::domain::search::SearchHit;
/// use backend::domain::articles::PostCategory;
///
/// let hit = SearchHit {
///     id: "article-123".to_string(),
///     title: "Introduction to <span class=\"highlight\">Rust</span>".to_string(),
///     category: PostCategory::Article,
///     summary: "A guide to <span class=\"highlight\">Rust</span> programming...".to_string(),
///     content: "Learn <span class=\"highlight\">Rust</span> basics...".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// Unique identifier of the article
    pub id: String,

    /// Article title (may include HTML highlighting tags)
    pub title: String,

    /// Category of the article
    pub category: PostCategory,

    /// Article summary, possibly truncated and with highlighting
    pub summary: String,

    /// Article content excerpt, possibly truncated and with highlighting
    pub content: String,
}

/// Search service trait for full-text search operations
///
/// This trait defines the interface for search functionality, abstracting the
/// underlying search engine implementation (e.g., Meilisearch, Elasticsearch).
/// It follows the Repository pattern, allowing different search implementations
/// to be swapped without changing business logic.
///
/// # Design Pattern
///
/// This trait is defined in the domain layer but implemented in the
/// infrastructure layer, following the Dependency Inversion Principle.
/// The actual search engine details are hidden behind this abstraction.
///
/// # Thread Safety
///
/// The trait requires `Send + Sync` to ensure implementations can be safely
/// shared across async tasks and thread boundaries.
///
/// # Example Implementation
///
/// ```rust
/// use async_trait::async_trait;
/// use backend::domain::search::{SearchService, SearchHit};
///
/// struct MySearchService {
///     // implementation details
/// }
///
/// #[async_trait]
/// impl SearchService for MySearchService {
///     async fn search(
///         &self,
///         query: &str,
///         index: &str,
///         offset: usize,
///         limit: usize,
///     ) -> Result<(Vec<SearchHit>, usize, usize, usize)> {
///         // implementation
///     }
///
///     async fn create_index_client(
///         &self,
///         index: &str,
///         searchable_attributes: &[&str],
///     ) -> Result<&Client> {
///         // implementation
///     }
/// }
/// ```
#[async_trait]
pub trait SearchService: Send + Sync {
    /// Perform a full-text search on articles
    ///
    /// This method searches for articles matching the given query with support
    /// for pagination, highlighting, and relevance ranking. Results are ordered
    /// by relevance score (most relevant first).
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string (supports full-text search syntax)
    /// * `index` - Name of the search index to query (e.g., "articles")
    /// * `offset` - Current page number (1-based indexing)
    /// * `limit` - Maximum number of results to return per page
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `Vec<SearchHit>` - List of search results with highlighting
    /// * `usize` - Total number of matching articles across all pages
    /// * `usize` - Total number of pages based on limit
    /// * `usize` - Current page number
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The search index doesn't exist
    /// * The search service is unavailable
    /// * The query syntax is invalid
    /// * A network or database error occurs
    ///
    /// # Example
    ///
    /// ```rust
    /// let (results, total, pages, current) =
    ///     search_service.search("rust programming", "articles", 1, 10).await?;
    ///
    /// println!("Found {} results across {} pages", total, pages);
    /// for hit in results {
    ///     println!("- {}: {}", hit.id, hit.title);
    /// }
    /// ```
    async fn search(
        &self,
        query: &str,
        index: &str,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<SearchHit>, usize, usize, usize)>;

    /// Create and configure a search index client
    ///
    /// This method creates a new search index with the specified name and
    /// configures which fields should be searchable. It returns a client
    /// that can be used for bulk operations like importing documents.
    ///
    /// # Arguments
    ///
    /// * `index` - Name for the new search index
    /// * `searchable_attributes` - Array of field names that should be searchable
    ///                             (e.g., ["title", "content", "summary"])
    ///
    /// # Returns
    ///
    /// * `Ok(&Client)` - A reference to the configured search client
    /// * `Err(SomeError)` - Error if index creation or configuration fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * An index with the same name already exists
    /// * The search service is unavailable
    /// * Invalid searchable attributes are specified
    /// * Insufficient permissions to create indexes
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = search_service
    ///     .create_index_client("articles", &["title", "summary", "content"])
    ///     .await?;
    ///
    /// // Use the client for bulk operations
    /// client.index("articles")
    ///     .add_documents(&articles, Some("id"))
    ///     .await?;
    /// ```
    ///
    /// # Note
    ///
    /// This method is typically used during initial setup or when rebuilding
    /// the search index from scratch. For regular document additions, use the
    /// existing index without creating a new one.
    async fn create_index_client(
        &self,
        index: &str,
        searchable_attributes: &[&str],
    ) -> Result<&Client>;
}
