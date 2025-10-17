use async_trait::async_trait;
use meilisearch_sdk::{
    client::Client,
    key::Key,
    search::{SearchResults, Selectors},
};

use crate::{
    config::Config,
    domain::{
        articles::Article,
        search::{SearchHit, SearchService},
    },
    errors::{Result, SearchError},
};

/// Client type enumeration for different Meilisearch permission levels
///
/// This enum distinguishes between different types of Meilisearch clients
/// based on their API key permissions. It's currently defined but not
/// actively used in the implementation.
///
/// # Variants
///
/// * `Search` - Read-only client for search operations
/// * `Admin` - Full-access client for administrative operations
#[derive(Debug, Clone, Copy)]
pub enum ClientType {
    /// Read-only client for search operations
    Search,
    /// Full-access client for administrative operations (add/update/delete)
    Admin,
}

/// Meilisearch service implementation for full-text search operations
///
/// This struct encapsulates all interactions with Meilisearch, a fast and
/// user-friendly search engine. It maintains two separate clients with
/// different permission levels for security best practices.
///
/// # Architecture
///
/// - Uses separate API keys for search (read-only) and admin (full access)
/// - Search client is used for public-facing search queries
/// - Admin client is used for indexing, updating, and deleting documents
/// - This separation follows the principle of least privilege
///
/// # Cloning
///
/// This struct is `Clone` because Meilisearch clients are internally
/// reference-counted and can be safely shared across tasks.
///
/// # Example
///
/// ```rust
/// use backend::infrastructure::search::index::MeiliSearchService;
/// use backend::config::Config;
///
/// let config = Config::new()?;
/// let search_service = MeiliSearchService::new(&config, "articles").await?;
/// ```
#[derive(Clone)]
pub struct MeiliSearchService {
    /// Admin client with full permissions for document management
    pub admin_client: Client,

    /// Search client with read-only permissions for queries
    pub search_client: Client,

    /// Name of the Meilisearch index (e.g., "articles")
    pub index_name: String,
}

impl MeiliSearchService {
    /// Create a new MeiliSearchService instance with configured clients
    ///
    /// This constructor initializes the search service by:
    /// 1. Creating a master client with the master key
    /// 2. Verifying Meilisearch service health
    /// 3. Retrieving separate admin and search API keys
    /// 4. Creating two clients with different permission levels
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration containing Meilisearch URL and credentials
    /// * `index_name` - Name of the search index to use (e.g., "articles")
    ///
    /// # Returns
    ///
    /// * `Ok(MeiliSearchService)` - Successfully initialized service
    /// * `Err(SearchError)` - Failed to connect or retrieve API keys
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Meilisearch URL is invalid or unreachable
    /// - Master key is incorrect
    /// - Health check fails
    /// - Default admin or search API keys don't exist
    ///
    /// # Security
    ///
    /// Uses principle of least privilege by creating separate clients:
    /// - Admin client: Used only for indexing operations
    /// - Search client: Used for public-facing search queries
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = Config::new()?;
    /// let service = MeiliSearchService::new(&config, "articles").await?;
    /// ```
    pub async fn new(config: &Config, index_name: &str) -> Result<Self> {
        // Extract Meilisearch server URL from configuration
        let meili_search_url = &config.meilisearch_url;

        // Create a temporary master client to retrieve API keys
        // The master key has full administrative privileges
        let master_client = &Self::create_master_client(config)?;

        // Verify Meilisearch service is healthy and responsive
        master_client.health().await?;

        // Retrieve the default admin and search API keys from Meilisearch
        // These keys are automatically created by Meilisearch with appropriate permissions
        let admin_key = get_standard_key(master_client, StandardApiKey::Admin).await?;
        let search_key = get_standard_key(master_client, StandardApiKey::Search).await?;

        // Create separate clients with different permission levels
        // This implements least privilege principle
        let admin_client = Client::new(meili_search_url, Some(admin_key))?;
        let search_client = Client::new(meili_search_url, Some(search_key))?;

        // Return the initialized service instance
        Ok(Self {
            admin_client,
            search_client,
            index_name: index_name.to_string(),
        })
    }

    /// Update or add an article to the search index
    ///
    /// This method performs an upsert operation: if a document with the same
    /// ID already exists in the index, it will be updated; otherwise, a new
    /// document will be added.
    ///
    /// # Arguments
    ///
    /// * `article` - The article entity to index
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Article successfully indexed
    /// * `Err(SomeError)` - Indexing operation failed
    ///
    /// # Behavior
    ///
    /// - Uses the admin client (requires write permissions)
    /// - Waits for the indexing operation to complete before returning
    /// - Uses "id" field as the primary key for documents
    /// - Replaces existing documents with the same ID
    ///
    /// # Performance
    ///
    /// This method is synchronous - it waits for Meilisearch to finish
    /// processing the document. For bulk operations, consider batching
    /// multiple documents together.
    ///
    /// # Example
    ///
    /// ```rust
    /// let article = Article { /* ... */ };
    /// service.update_or_add_index_item(&article).await?;
    /// ```
    pub async fn update_or_add_index_item(&self, article: &Article) -> Result<()> {
        let client = &self.admin_client;
        let index = client.index(&self.index_name);

        // Add or replace the document in the index, using "id" as primary key
        // If a document with this ID exists, it will be replaced
        index
            .add_or_replace(&[article], Some("id"))
            .await?
            // Wait for Meilisearch to finish processing this task
            // This ensures the document is immediately searchable
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(())
    }

    /// Delete an article from the search index
    ///
    /// Removes a document from the Meilisearch index by its ID. The article
    /// will no longer appear in search results after this operation completes.
    ///
    /// # Arguments
    ///
    /// * `article` - The article entity to remove (only ID is used)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Article successfully deleted
    /// * `Err(SomeError)` - Deletion operation failed
    ///
    /// # Behavior
    ///
    /// - Uses the admin client (requires delete permissions)
    /// - Waits for the deletion to complete before returning
    /// - If the document doesn't exist, the operation still succeeds (idempotent)
    ///
    /// # Example
    ///
    /// ```rust
    /// let article = Article { id: "article-123".to_string(), /* ... */ };
    /// service.delete_index_item(&article).await?;
    /// ```
    pub async fn delete_index_item(&self, article: &Article) -> Result<()> {
        let client = &self.admin_client;
        let index = client.index(&self.index_name);

        // Delete the document from the index by its ID
        index
            .delete_document(&article.id)
            .await?
            // Wait for Meilisearch to finish processing the deletion
            // This ensures the document is immediately removed from search results
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(())
    }

    /// Create a Meilisearch client with master key authentication
    ///
    /// This private method creates a client with full administrative privileges
    /// using the master key. It's used internally during initialization to
    /// retrieve the admin and search API keys.
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration containing Meilisearch credentials
    ///
    /// # Returns
    ///
    /// * `Ok(Client)` - Successfully authenticated client
    /// * `Err(SomeError)` - Client creation failed
    ///
    /// # Security
    ///
    /// The master key should never be exposed to client-side code or used
    /// for regular operations. It's only used during service initialization.
    fn create_master_client(config: &Config) -> Result<Client> {
        let meili_search_url = &config.meilisearch_url;
        let key = &config.meili_master_key;

        // Initialize client with master key for full administrative access
        Ok(Client::new(meili_search_url, Some(key))?)
    }

    /// Execute a search query and return raw results
    ///
    /// This method performs a full-text search with highlighting and content
    /// cropping. It's a lower-level method that returns raw SearchResults
    /// from Meilisearch.
    ///
    /// # Arguments
    ///
    /// * `index` - Name of the index to search
    /// * `page_limit` - Maximum number of results to return
    /// * `params` - Search query string
    /// * `offset` - Number of results to skip (for pagination)
    ///
    /// # Returns
    ///
    /// * `Ok(SearchResults<Article>)` - Raw search results from Meilisearch
    /// * `Err(SomeError)` - Search operation failed
    ///
    /// # Features
    ///
    /// - **Highlighting**: Matching terms are wrapped in `<span class="highlight">`
    /// - **Cropping**: Summary and content are truncated around matches
    /// - **Pagination**: Supports offset and limit for pagination
    ///
    /// # Note
    ///
    /// This is an internal helper method. Most code should use the
    /// `SearchService::search` trait method instead.
    pub async fn get_search_result(
        &self,
        index: &str,
        page_limit: usize,
        params: &str,
        offset: usize,
    ) -> Result<SearchResults<Article>> {
        let search_index = &self.search_client.index(index);

        // Build and execute the search query with highlighting and cropping
        let search_result = search_index
            .search()
            .with_query(params) // Set the search query string
            .with_offset(offset) // Set pagination offset
            .with_limit(page_limit) // Set maximum number of results
            .with_attributes_to_highlight(Selectors::Some(&["title", "summary", "content"])) // Fields to highlight
            .with_highlight_pre_tag("<span class=\"highlight\">") // HTML tag before highlighted text
            .with_highlight_post_tag("</span>") // HTML tag after highlighted text
            .with_attributes_to_crop(Selectors::Some(&[("summary", None), ("content", None)])) // Fields to truncate
            .execute::<Article>() // Execute search and parse results as Article entities
            .await?;

        Ok(search_result)
    }
}

/// Enumeration of standard Meilisearch API key types
///
/// Meilisearch automatically creates two default API keys with different
/// permission levels. This enum is used to identify which key to retrieve.
///
/// # Variants
///
/// * `Search` - Read-only key for search operations
/// * `Admin` - Full-access key for administrative operations
#[derive(Debug, Clone, Copy)]
enum StandardApiKey {
    /// Read-only API key for search operations
    Search,
    /// Full-access API key for administrative operations
    Admin,
}

/// Retrieve all API keys from Meilisearch
///
/// This function fetches the complete list of API keys configured in
/// Meilisearch. It's currently unused but kept for potential future use.
///
/// # Arguments
///
/// * `config` - Application configuration containing Meilisearch credentials
///
/// # Returns
///
/// * `Ok(Vec<Key>)` - List of all API keys
/// * `Err(SomeError)` - Failed to retrieve keys
async fn get_api_keys(config: &Config) -> Result<Vec<Key>> {
    let client = MeiliSearchService::create_master_client(config)?;
    Ok(client.get_keys().await?.results)
}

/// Retrieve a specific standard API key from Meilisearch
///
/// Meilisearch automatically creates two default API keys with predefined names.
/// This function retrieves one of these keys based on the specified type.
///
/// # Arguments
///
/// * `client` - Meilisearch client (must have permission to list keys)
/// * `key_type` - Type of key to retrieve (Admin or Search)
///
/// # Returns
///
/// * `Ok(String)` - The API key value
/// * `Err(SearchError)` - Key not found or API call failed
///
/// # Default Key Names
///
/// - Admin: "Default Admin API Key"
/// - Search: "Default Search API Key"
///
/// These names are set by Meilisearch and should not be changed.
async fn get_standard_key(client: &Client, key_type: StandardApiKey) -> Result<String> {
    // Determine the key name based on the requested type
    let key_name = match &key_type {
        StandardApiKey::Admin => "Default Admin API Key",
        StandardApiKey::Search => "Default Search API Key",
    };

    // Fetch all API keys from Meilisearch
    let keys = client.get_keys().await?.results;

    // Search for the key with the matching name
    if let Some(key) = keys.iter().find(|&k| k.name.as_deref() == Some(key_name)) {
        return Ok(key.key.clone());
    }

    // Return appropriate error if key not found
    match key_type {
        StandardApiKey::Admin => Err(SearchError::DefaultAdminApiKeyNotFound.into()),
        StandardApiKey::Search => Err(SearchError::DefaultSearchApiKeyNotFound.into()),
    }
}

/// Retrieve a custom-named API key from Meilisearch
///
/// This function looks up an API key by its custom name. It's useful for
/// retrieving non-standard keys that were manually created in Meilisearch.
///
/// # Arguments
///
/// * `client` - Meilisearch client (must have permission to list keys)
/// * `key_name` - Name of the custom key to retrieve
///
/// # Returns
///
/// * `Ok(String)` - The API key value
/// * `Err(SearchError::CustomApiKeyNotFound)` - Key with given name doesn't exist
///
/// # Note
///
/// This function is currently unused but kept for potential future use
/// with custom API keys.
async fn get_custom_key(client: &Client, key_name: &str) -> Result<String> {
    // Fetch all API keys from Meilisearch
    let keys = client.get_keys().await?.results;

    // Search for the key with the matching custom name
    if let Some(key) = keys.iter().find(|&k| k.name.as_deref() == Some(key_name)) {
        return Ok(key.key.clone());
    }

    // Return error if custom key not found
    Err(SearchError::CustomApiKeyNotFound(key_name.to_string()).into())
}

/// Implementation of the SearchService trait for MeiliSearchService
///
/// This implementation provides the concrete search functionality using
/// Meilisearch as the backend search engine.
#[async_trait]
impl SearchService for MeiliSearchService {
    /// Perform a full-text search and return processed results
    ///
    /// This method implements the SearchService trait's search functionality,
    /// executing a query against Meilisearch and processing the results into
    /// a format suitable for the application.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query string
    /// * `index` - Name of the search index
    /// * `current_page` - Page number (1-based indexing)
    /// * `limit` - Number of results per page
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `Vec<SearchHit>` - Search results with highlighting
    /// * `usize` - Total number of matching documents
    /// * `usize` - Total number of pages
    /// * `usize` - Current page number
    ///
    /// # Implementation Details
    ///
    /// - Uses the search client (read-only permissions)
    /// - Ensures page number is at least 1
    /// - Calculates offset for pagination
    /// - Processes results with highlighting and cropping
    /// - Extracts formatted content with HTML highlighting
    async fn search(
        &self,
        query: &str,
        index: &str,
        current_page: usize,
        limit: usize,
    ) -> Result<(Vec<SearchHit>, usize, usize, usize)> {
        let index = &self.search_client.index(index);

        // Ensure page number is at least 1 (1-based indexing)
        let current_page = current_page.max(1);

        // Calculate offset for pagination
        let offset = (current_page - 1) * limit;

        let search_result = index
            .search()
            .with_query(query)
            .with_offset(offset)
            .with_limit(limit)
            .with_attributes_to_highlight(Selectors::Some(&["title", "summary", "content"])) // 设置高亮的字段
            .with_highlight_pre_tag("<span class=\"highlight\">") // 设置高亮前缀标签
            .with_highlight_post_tag("</span>") // 设置高亮后缀标签
            .with_attributes_to_crop(Selectors::Some(&[("summary", None), ("content", None)])) // 设置要裁剪的字段
            .execute::<Article>() // 执行搜索
            .await?;

        // 计算总命中数和总页数
        let total_hits = search_result.total_hits.unwrap_or(0);
        let total_pages = (total_hits + limit - 1) / limit;

        let results: Vec<SearchHit> = search_result
            .hits
            .into_iter()
            .map(|r| {
                // 创建一个默认的 SearchHit
                let mut hit_result = SearchHit {
                    id: r.result.id.clone(),
                    category: r.result.category.clone(),
                    title: r.result.title.clone(),
                    summary: String::new(),
                    content: String::new(),
                };

                // 如果有格式化（高亮和裁剪）的结果，则使用它们
                if let Some(formatted) = &r.formatted_result {
                    hit_result.summary = formatted
                        .get("summary")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    hit_result.content = formatted
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                }

                hit_result
            })
            .collect();

        Ok((results, total_hits, total_pages, current_page))
    }

    async fn create_index_client(
        &self,
        index: &str,
        searchable_attributes: &[&str],
    ) -> Result<&Client> {
        let client = &self.admin_client;

        client
            .create_index(index, Some("id"))
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        client
            .index(index)
            .set_filterable_attributes(searchable_attributes)
            .await?;

        Ok(client)
    }
}
