use std::sync::Arc;

#[cfg(feature = "webhook")]
use gray_matter::{Matter, engine::YAML};
#[cfg(feature = "webhook")]
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};

#[cfg(feature = "webhook")]
use crate::domain::articles::ArticleFrontMatter;
#[cfg(feature = "webhook")]
use crate::infrastructure::github::webhook::FileChange;
#[cfg(feature = "webhook")]
use crate::infrastructure::github::{client::GithubClient, webhook::WebhookHandler};

use crate::{
    config::AppConfig,
    domain::{
        articles::Article,
        repositories::ArticleRepository,
        search::{SearchHit, SearchService},
    },
    errors::Result,
    interfaces::http::dtos::PostResponse,
};

/// Article service layer containing business logic for article operations
///
/// This service coordinates between the repository layer (database), search service,
/// and external GitHub API (when webhook feature is enabled). It implements the
/// application's use cases following the clean architecture pattern.
///
/// # Responsibilities
///
/// - Article CRUD operations
/// - Search functionality
/// - Webhook event processing (when webhook feature is enabled)
/// - Business rule validation
/// - Coordination between different infrastructure services
pub struct ArticleService {
    /// Repository for article persistence operations
    db_repo: Arc<dyn ArticleRepository>,

    /// GitHub API client for fetching file content (webhook feature only)
    #[cfg(feature = "webhook")]
    github_client: Arc<dyn GithubClient>,

    /// Search service for full-text search operations
    search_service: Arc<dyn SearchService>,

    /// Application configuration including secrets and settings
    config: Arc<AppConfig>,
}

impl ArticleService {
    /// Create a new ArticleService instance with all required dependencies
    ///
    /// # Arguments
    ///
    /// * `db_repo` - Article repository implementation for database operations
    /// * `github_client` - GitHub API client (webhook feature only)
    /// * `search_service` - Search service implementation for full-text search
    /// * `config` - Application configuration
    ///
    /// # Returns
    ///
    /// A new ArticleService instance with all dependencies injected
    pub fn new(
        db_repo: Arc<dyn ArticleRepository>,
        #[cfg(feature = "webhook")] github_client: Arc<dyn GithubClient>,
        search_service: Arc<dyn SearchService>,
        config: Arc<AppConfig>,
    ) -> Self {
        Self {
            db_repo,
            #[cfg(feature = "webhook")]
            github_client,
            search_service,
            config,
        }
    }

    /// Process incoming GitHub webhook events
    ///
    /// This function handles webhook events from GitHub, validates the repository
    /// against the allowed list, and dispatches to appropriate handlers based on
    /// event type.
    ///
    /// # Arguments
    ///
    /// * `event` - The webhook event payload from GitHub
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Event processed successfully or ignored (if repository not allowed)
    /// * `Err(SomeError)` - Error occurred during processing
    ///
    /// # Supported Events
    ///
    /// - Push events: Processes file changes in the repository
    /// - Other events: Logged but not processed
    #[cfg(feature = "webhook")]
    pub async fn process_github_webhook_event(&self, event: &WebhookEvent) -> Result<()> {
        let repo_name = event.get_repository_name()?;

        // Verify the repository is in the allowed list
        if !self
            .config
            .allowed_repositories
            .read()
            .await
            .contains(&repo_name)
        {
            tracing::warn!("Repository {} is not allowed", repo_name);
            return Ok(());
        }

        tracing::info!("Processing webhook event for repository: {}", repo_name);

        // Dispatch based on event type
        match &event.kind {
            WebhookEventType::Push => {
                self.process_push_event(event).await?;
            }

            _ => {
                tracing::warn!("Unsupported webhook event type: {:?}", event.kind);
            }
        }

        Ok(())
    }

    /// Process a push event from GitHub webhook
    ///
    /// Extracts file changes from the push event and processes each changed file
    /// based on its status (added, modified, or removed). Only processes files
    /// with valid extensions (.md, .mdx).
    ///
    /// # Arguments
    ///
    /// * `event` - The webhook push event payload
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All file changes processed successfully
    /// * `Err(SomeError)` - Error occurred during file processing
    #[cfg(feature = "webhook")]
    async fn process_push_event(&self, event: &WebhookEvent) -> Result<()> {
        let repo_name = event.get_repository_name()?;
        let owner = event.get_repository_owner()?;

        tracing::info!("Processing push event for repository: {}", repo_name);

        let (mut added_files, removed_files, modified_files) = event.get_push_file_changes();

        self.process_modified_files(&owner, &repo_name, &modified_files)
            .await?;

        added_files.retain(|f| self.is_valid_file(&f.file_path));
        self.process_added_and_removed_event(&owner, &repo_name, &added_files, &removed_files)
            .await?;

        Ok(())
    }

    /// Check if a file is valid for processing
    ///
    /// Validates that the file has an allowed extension (.md or .mdx).
    /// Only markdown files should be processed as articles.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to validate
    ///
    /// # Returns
    ///
    /// * `true` - File has a valid extension
    /// * `false` - File extension is not allowed or could not be determined
    #[cfg(feature = "webhook")]
    fn is_valid_file(&self, file_path: &str) -> bool {
        use std::path::Path;

        let allowed_extensions = ["md", "mdx"];

        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| allowed_extensions.contains(&ext))
            .unwrap_or(false)
    }

    #[cfg(feature = "webhook")]
    pub async fn process_added_and_removed_event(
        &self,
        owner: &str,
        repo: &str,
        added: &[FileChange],
        removed: &[FileChange],
    ) -> Result<()> {
        use std::collections::{HashMap, HashSet};
        use time::OffsetDateTime;

        let db_article_metadatas = self.db_repo.get_all_metadata().await?;

        let mut db_path_to_id = HashMap::new();
        let mut db_id_to_path = HashMap::new();

        let mut add = Vec::new();
        let mut update = Vec::new();
        let mut remove = Vec::new();

        let mut updated_article_uuids = HashSet::new();

        for (id, path) in db_article_metadatas {
            db_id_to_path.insert(id.clone(), path.clone());
            db_path_to_id.insert(path, id);
        }

        let added_contents = self.github_client.fetch_files(owner, repo, added).await;
        for (timestamp, content) in added_contents {
            if let Ok(content) = content {
                let (info, content) = self.extract_article(&content)?;
                let timestamp = OffsetDateTime::from_unix_timestamp(timestamp)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc());

                if db_id_to_path.contains_key(&info.id) {
                    update.push(Article {
                        id: info.id.clone(),
                        title: info.title,
                        category: info.category,
                        status: info.status,
                        tags: info.tags,
                        summary: info.summary,
                        content: content,
                        created_at: OffsetDateTime::now_utc(),
                        updated_at: timestamp,
                    });

                    updated_article_uuids.insert(info.id);
                } else {
                    add.push(Article {
                        id: info.id,
                        title: info.title,
                        category: info.category,
                        status: info.status,
                        tags: info.tags,
                        summary: info.summary,
                        content: content,
                        created_at: timestamp,
                        updated_at: timestamp,
                    });
                }
            }
        }

        for (uuid, _) in db_id_to_path {
            if !updated_article_uuids.contains(&uuid) {
                remove.push(uuid);
            }
        }

        self.process_added_files(&add).await?;
        self.process_removed_files(&remove).await?;

        todo!()
    }

    /// Process a newly added file from GitHub
    ///
    /// Fetches the file content from GitHub API, parses the front matter (YAML metadata),
    /// and creates a new article in the database and search index.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner username
    /// * `repo` - Repository name
    /// * `file_path` - Path to the file within the repository
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File processed and article created successfully
    /// * `Err(SomeError)` - Error occurred during fetching, parsing, or saving
    ///
    /// # TODO
    ///
    /// - Extract article ID from front matter or file path
    /// - Create article entity from front matter and content
    /// - Save to database
    /// - Update search index
    #[cfg(feature = "webhook")]
    pub async fn process_added_files(&self, articles: &[Article]) -> Result<()> {
        todo!()
    }

    /// Process a modified file from GitHub
    ///
    /// Fetches the updated file content from GitHub API, parses the front matter,
    /// and updates the existing article in the database and search index.
    ///
    /// The processing logic is similar to adding a new file, but updates an
    /// existing article instead of creating a new one.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner username
    /// * `repo` - Repository name
    /// * `file_path` - Path to the modified file within the repository
    /// * `timestamp` - Timestamp of the modification
    ///
    /// # Returns
    ///
    /// * `Ok(())` - File processed and article updated successfully
    /// * `Err(SomeError)` - Error occurred during fetching, parsing, or updating
    ///
    #[cfg(feature = "webhook")]
    pub async fn process_modified_files(
        &self,
        owner: &str,
        repo: &str,
        modified: &[FileChange],
    ) -> Result<()> {
        use time::OffsetDateTime;

        let contents = self
            .github_client
            .fetch_files(&owner, &repo, &modified)
            .await;

        let mut articles = Vec::new();

        for (timestamp, content) in contents {
            if let Ok(content) = content {
                let (article_info, content) = self.extract_article(&content)?;
                let timestamp = OffsetDateTime::from_unix_timestamp(timestamp)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc());

                let article = Article {
                    id: article_info.id,
                    title: article_info.title,
                    category: article_info.category,
                    content: content,
                    status: article_info.status,
                    summary: article_info.summary,
                    tags: article_info.tags,
                    created_at: OffsetDateTime::now_utc(),
                    updated_at: timestamp,
                };

                articles.push(article);
            }
        }

        self.db_repo.update_by_path(&articles).await?;

        Ok(())
    }

    #[cfg(feature = "webhook")]
    fn extract_article(&self, content: &str) -> Result<(ArticleFrontMatter, String)> {
        let matter = Matter::<YAML>::new();
        let front_matter = matter.parse::<ArticleFrontMatter>(content)?;
        let info = front_matter
            .data
            .ok_or_else(|| anyhow::anyhow!("Extract article info failed."))?;

        Ok((info, front_matter.content))
    }

    /// Process a removed file from GitHub
    ///
    /// Extracts the article ID from the file path and removes the article
    /// from both the database and search index.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the removed file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Article removed successfully
    /// * `Err(SomeError)` - Error occurred during deletion
    ///
    /// # TODO
    ///
    /// - Extract article ID from file path
    /// - Delete from database using db_repo.delete_by_path()
    /// - Remove from search index
    #[cfg(feature = "webhook")]
    pub async fn process_removed_files<T: AsRef<str>>(&self, uuid: &[T]) -> Result<()> {
        // TODO: Extract article ID from file path
        // TODO: Delete article from database
        // TODO: Remove article from search index

        Ok(())
    }

    /// Check if an article ID is available for use
    ///
    /// Validates that the given ID is not already in use by an existing article.
    /// This is useful for preventing duplicate IDs when creating new articles.
    ///
    /// # Arguments
    ///
    /// * `id` - The article ID to check
    ///
    /// # Returns
    ///
    /// * `true` - ID is available (not in use)
    /// * `false` - ID is already taken by an existing article
    ///
    /// Note: Returns `true` if an error occurs during the check, assuming the ID
    /// is available to avoid blocking article creation.
    pub async fn is_valid_id(&self, id: &str) -> bool {
        match self.db_repo.find_optional_by_id(id).await {
            Ok(Some(_)) => false, // ID exists, not valid for new article
            Ok(None) => true,     // ID doesn't exist, valid for new article
            Err(_) => true,       // Error occurred, assume valid to avoid blocking
        }
    }

    /// Retrieve paginated list of articles by category
    ///
    /// Fetches articles filtered by category with pagination support.
    /// Results are typically ordered by creation date in descending order.
    ///
    /// # Arguments
    ///
    /// * `category` - Category to filter by (e.g., "article", "note", "think")
    /// * `page_size` - Number of articles to return per page
    /// * `offset` - Number of articles to skip (for pagination)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<PostResponse>)` - List of articles matching the criteria
    /// * `Err(SomeError)` - Error occurred during database query
    ///
    /// # Example
    ///
    /// ```rust
    /// // Get first page of articles (10 items)
    /// let articles = service.get_posts_by_category("article", 10, 0).await?;
    ///
    /// // Get second page
    /// let articles = service.get_posts_by_category("article", 10, 10).await?;
    /// ```
    pub async fn get_posts_by_category(
        &self,
        category: &str,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<PostResponse>> {
        self.db_repo
            .get_posts_by_category(category, page_size, offset)
            .await
    }

    /// Retrieve a single article by its ID
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier of the article
    ///
    /// # Returns
    ///
    /// * `Ok(Article)` - The complete article entity
    /// * `Err(GetPostsError::ArticleNotFound)` - Article with given ID doesn't exist
    /// * `Err(SomeError)` - Other error occurred during retrieval
    pub async fn get_article_by_id(&self, id: &str) -> Result<Article> {
        self.db_repo.get_post_by_id(id).await
    }

    pub async fn get_article_by_path(&self, path: &str) -> Result<Option<Article>> {
        self.db_repo.find_optional_by_file_path(path).await
    }

    /// Perform full-text search on articles
    ///
    /// Searches articles using the search service (Meilisearch) with support
    /// for pagination, highlighting, and relevance ranking.
    ///
    /// # Arguments
    ///
    /// * `index` - Name of the search index to query (typically "articles")
    /// * `query` - Search query string
    /// * `current_page` - Current page number (1-based)
    /// * `limit` - Number of results per page
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `Vec<SearchHit>` - List of search results with highlighting
    /// * `usize` - Total number of matching articles
    /// * `usize` - Total number of pages
    /// * `usize` - Current page number
    ///
    /// # Example
    ///
    /// ```rust
    /// let (results, total, pages, current) =
    ///     service.search("articles", "rust programming", 1, 10).await?;
    ///
    /// println!("Found {} results across {} pages", total, pages);
    /// for hit in results {
    ///     println!("{}: {}", hit.id, hit.title);
    /// }
    /// ```
    pub async fn search(
        &self,
        index: &str,
        query: &str,
        current_page: usize,
        limit: usize,
    ) -> Result<(Vec<SearchHit>, usize, usize, usize)> {
        self.search_service
            .search(query, index, current_page, limit)
            .await
    }

    /// Create a search index and populate it with existing articles
    ///
    /// This is typically used during initial setup or when rebuilding the
    /// search index from scratch. It creates a new index with the specified
    /// searchable attributes and imports all articles from the database.
    ///
    /// # Arguments
    ///
    /// * `index` - Name of the search index to create
    /// * `searchable_attributes` - Fields that should be searchable (e.g., ["title", "content"])
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Index created and populated successfully
    /// * `Err(SomeError)` - Error occurred during index creation or population
    ///
    /// # Example
    ///
    /// ```rust
    /// service.create_index("articles", &["title", "summary", "content"]).await?;
    /// ```
    pub async fn create_index(&self, index: &str, searchable_attributes: &[&str]) -> Result<()> {
        // Create the index with specified searchable attributes
        let client = self
            .search_service
            .create_index_client(index, searchable_attributes)
            .await?;

        // Fetch all articles from database
        let db_articles = self.db_repo.get_all().await?;

        // Import all articles into the search index
        client
            .index(index)
            .add_documents(&db_articles, Some("id"))
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(())
    }
}
