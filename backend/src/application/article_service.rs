#[cfg(feature = "webhook")]
use std::collections::HashSet;
use std::sync::Arc;

#[cfg(feature = "webhook")]
use gray_matter::{Matter, engine::YAML};
#[cfg(feature = "webhook")]
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};
#[cfg(feature = "webhook")]
use time::OffsetDateTime;

#[cfg(feature = "webhook")]
use crate::domain::articles::ArticleFrontMatter;
#[cfg(feature = "webhook")]
use crate::domain::repositories::TransactionGuard;
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
        use crate::domain::search::DEFAULT_SEARCH_INDEX;

        let repo_name = event.get_repository_name()?;
        let owner = event.get_repository_owner()?;

        tracing::info!("Processing push event for repository: {}", repo_name);

        let (mut added_files, removed_files, modified_files) = event.get_push_file_changes();

        let modified_articles = self
            .process_modified_event(&owner, &repo_name, &modified_files)
            .await?;

        added_files.retain(|f| self.is_valid_file(&f.file_path));
        let (added, modified, removed) = self
            .process_added_and_removed_event(&owner, &repo_name, &added_files, &removed_files)
            .await?;

        let upsert_articles: Vec<Article> = modified_articles
            .into_iter()
            .chain(modified)
            .chain(added)
            .collect();

        let mut tx = self.db_repo.begin_transaction().await?;
        self.process_upsert_files(&upsert_articles, &mut tx).await?;
        self.process_deleted_files(&removed, &mut tx).await?;
        tx.commit().await?;

        self.create_index(DEFAULT_SEARCH_INDEX).await?;

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
    ) -> Result<(Vec<Article>, Vec<Article>, HashSet<String>)> {
        use time::OffsetDateTime;

        use crate::infrastructure::time_utils::chrono_to_offset;

        let removed_paths: Vec<String> = removed.into_iter().map(|f| f.file_path.clone()).collect();
        let mut removed_files_id = self.db_repo.get_by_paths(&removed_paths).await?;

        let mut add = Vec::new();
        let mut modify = Vec::new();

        let added_contents = self.github_client.fetch_files(owner, repo, added).await;
        for (timestamp, content, file_path) in added_contents {
            match content {
                Ok(content) => match self.extract_article(&content) {
                    Ok((info, content)) => {
                        let offset_timestamp = chrono_to_offset(timestamp).unwrap_or_else(|_| {
                            tracing::warn!("Failed to parse timestamp");
                            OffsetDateTime::now_utc()
                        });

                        if removed_files_id.contains(&info.id) {
                            removed_files_id.remove(&info.id);

                            modify.push(build_article(
                                info,
                                file_path,
                                content,
                                offset_timestamp,
                                offset_timestamp,
                            ));
                        } else {
                            add.push(build_article(
                                info,
                                file_path,
                                content,
                                offset_timestamp,
                                offset_timestamp,
                            ));
                        }
                    }

                    Err(e) => {
                        tracing::warn!("Failed to extract article: {}", e);
                    }
                },

                Err(e) => {
                    tracing::warn!("Failed to fetch file content: {}", e)
                }
            }
        }

        Ok((add, modify, removed_files_id))
    }

    #[cfg(feature = "webhook")]
    pub async fn process_modified_event(
        &self,
        owner: &str,
        repo: &str,
        modified: &[FileChange],
    ) -> Result<Vec<Article>> {
        use time::OffsetDateTime;

        use crate::infrastructure::time_utils::chrono_to_offset;

        let contents = self
            .github_client
            .fetch_files(&owner, &repo, &modified)
            .await;

        let mut articles = Vec::new();

        for (timestamp, content, file_path) in contents {
            if let Ok(content) = content {
                let (article_info, content) = match self.extract_article(&content) {
                    Ok((article_info, content)) => (article_info, content),
                    Err(err) => {
                        tracing::warn!(
                            "Failed to extract article from file {}: {}",
                            file_path,
                            err
                        );
                        continue;
                    }
                };

                let offset_timestamp = chrono_to_offset(timestamp).unwrap_or_else(|_| {
                    tracing::warn!("Failed to parse timestamp");
                    OffsetDateTime::now_utc()
                });

                articles.push(build_article(
                    article_info,
                    file_path,
                    content,
                    offset_timestamp,
                    offset_timestamp,
                ));
            }
        }

        Ok(articles)
    }

    #[cfg(feature = "webhook")]
    pub async fn process_upsert_files(
        &self,
        upsert_articles: &[Article],
        tx: &mut TransactionGuard,
    ) -> Result<()> {
        if !upsert_articles.is_empty() {
            tx.upsert_batch(upsert_articles).await?;
        }

        Ok(())
    }

    #[cfg(feature = "webhook")]
    pub async fn process_deleted_files(
        &self,
        deleted: &HashSet<String>,
        tx: &mut TransactionGuard,
    ) -> Result<()> {
        if !deleted.is_empty() {
            tx.delete_batch(deleted).await?;
        }

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
    pub async fn create_index(&self, index: &str) -> Result<()> {
        let searchable_attributes = ["title", "content", "summary"];

        // Create the index with specified searchable attributes
        let client = self
            .search_service
            .create_index_client(index, &searchable_attributes)
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

#[cfg(feature = "webhook")]
fn build_article(
    front_matter: ArticleFrontMatter,
    path: String,
    content: String,
    create_at: OffsetDateTime,
    update_at: OffsetDateTime,
) -> Article {
    Article {
        id: front_matter.id,
        path: path,
        title: front_matter.title,
        tags: front_matter.tags,
        category: front_matter.category,
        summary: front_matter.summary,
        content: content,
        status: front_matter.status,
        created_at: create_at,
        updated_at: update_at,
    }
}
