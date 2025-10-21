use std::collections::HashSet;

use async_trait::async_trait;

use crate::{domain::articles::Article, errors::Result, interfaces::http::dtos::PostResponse};

/// Repository trait for article persistence operations
///
/// This trait defines the interface for article data access, following the
/// Repository pattern. It abstracts the underlying data storage mechanism,
/// allowing different implementations (e.g., PostgreSQL, in-memory) without
/// changing business logic.
///
/// All methods are async and return a `Result` type, allowing proper error
/// handling throughout the application.
///
/// # Design Pattern
///
/// This follows the Repository pattern from Domain-Driven Design (DDD),
/// providing a collection-like interface for accessing domain entities.
/// The trait is defined in the domain layer but implemented in the
/// infrastructure layer, following the Dependency Inversion Principle.
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
/// use backend::domain::repositories::ArticleRepository;
///
/// struct MyArticleRepository {
///     // implementation details
/// }
///
/// #[async_trait]
/// impl ArticleRepository for MyArticleRepository {
///     async fn find_optional_by_id(&self, id: &str) -> Result<Option<Article>> {
///         // implementation
///     }
///     // ... other methods
/// }
/// ```
#[async_trait]
pub trait ArticleRepository: Send + Sync {
    /// Find an article by its unique identifier
    ///
    /// This method searches for an article with the given ID and returns
    /// `Some(Article)` if found, or `None` if no article exists with that ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the article to find
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Article))` - Article was found and returned
    /// * `Ok(None)` - No article exists with the given ID
    /// * `Err(SomeError)` - An error occurred during the database query
    ///
    /// # Example
    ///
    /// ```rust
    /// let article = repo.find_optional_by_id("article-123").await?;
    /// match article {
    ///     Some(a) => println!("Found: {}", a.title),
    ///     None => println!("Article not found"),
    /// }
    /// ```
    async fn find_optional_by_id(&self, id: &str) -> Result<Option<Article>>;

    /// Save article(s)
    ///
    /// This method persists article(s) to the database.
    ///
    /// # Arguments
    ///
    /// * `articles` - The article entities to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Articles was saved successfully
    /// * `Err(SomeError)` - An error occurred during the save operation
    ///
    /// # Example
    ///
    /// ```rust
    /// let article = Article {
    ///     id: "new-article".to_string(),
    ///     title: "My Article".to_string(),
    ///     // ... other fields
    /// };
    /// repo.save(&[article]).await?;
    /// ```
    async fn save(&self, articles: &[Article]) -> Result<()>;

    async fn update(&self, articles: &[Article]) -> Result<()>;

    async fn update_by_path(&self, article_with_path: &[(Article, String)]) -> Result<()>;

    /// Delete an article by its file path
    ///
    /// This method removes an article from the database using its file path
    /// as the identifier. This is typically used when processing file deletion
    /// events from webhooks.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path associated with the article to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Article was deleted successfully (or didn't exist)
    /// * `Err(SomeError)` - An error occurred during the delete operation
    ///
    /// # Note
    ///
    /// The implementation may choose to map the file path to an article ID
    /// or use the path directly as an identifier.
    async fn delete_by_path(&self, path: &str) -> Result<()>;

    /// Retrieve a paginated list of articles filtered by category
    ///
    /// This method fetches articles belonging to a specific category with
    /// pagination support. Results are typically ordered by creation date
    /// in descending order (newest first).
    ///
    /// # Arguments
    ///
    /// * `category` - The category to filter by (e.g., "article", "note", "think")
    /// * `page_size` - Maximum number of articles to return
    /// * `offset` - Number of articles to skip (for pagination)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<PostResponse>)` - List of articles (may be empty)
    /// * `Err(SomeError)` - An error occurred during the query
    ///
    /// # Example
    ///
    /// ```rust
    /// // Get first page (10 articles)
    /// let page1 = repo.get_posts_by_category("article", 10, 0).await?;
    ///
    /// // Get second page
    /// let page2 = repo.get_posts_by_category("article", 10, 10).await?;
    /// ```
    async fn get_posts_by_category(
        &self,
        category: &str,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<PostResponse>>;

    /// Retrieve a single article by its unique identifier
    ///
    /// This method fetches a complete article entity by its ID. Unlike
    /// `find_optional_by_id`, this method returns an error if the article
    /// is not found, making it suitable for cases where the article is
    /// expected to exist.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the article
    ///
    /// # Returns
    ///
    /// * `Ok(Article)` - The complete article entity
    /// * `Err(GetPostsError::ArticleNotFound)` - Article doesn't exist
    /// * `Err(SomeError)` - Other database error occurred
    ///
    /// # Example
    ///
    /// ```rust
    /// let article = repo.get_post_by_id("article-123").await?;
    /// println!("Title: {}", article.title);
    /// ```
    async fn get_post_by_id(&self, id: &str) -> Result<Article>;

    /// Retrieve all articles from the database
    ///
    /// This method fetches all articles without any filtering or pagination.
    /// It's primarily used for administrative tasks like rebuilding the search
    /// index or generating sitemaps.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Article>)` - List of all articles (may be empty)
    /// * `Err(SomeError)` - An error occurred during the query
    ///
    /// # Warning
    ///
    /// This method loads all articles into memory at once. For large datasets,
    /// consider using pagination or streaming alternatives to avoid memory issues.
    ///
    /// # Example
    ///
    /// ```rust
    /// let all_articles = repo.get_all().await?;
    /// println!("Total articles: {}", all_articles.len());
    /// ```
    async fn get_all(&self) -> Result<Vec<Article>>;

    async fn find_optional_by_file_path(&self, path: &str) -> Result<Option<Article>>;

    async fn get_by_paths(&self, paths: &HashSet<&str>) -> Result<HashSet<String>>;

    async fn begin_transaction(&self) -> Result<TransactionGuard>;
}

pub struct TransactionGuard {
    pub inner: Box<dyn TransactionOps>,
}

impl TransactionGuard {
    pub async fn insert_batch(&mut self, articles: &[Article]) -> Result<()> {
        self.inner.insert_batch(articles).await
    }

    pub async fn delete_batch(&mut self, paths: &HashSet<String>) -> Result<()> {
        self.inner.delete_batch(paths).await
    }

    pub async fn commit(self) -> Result<()> {
        self.inner.commit().await
    }
}

#[async_trait]
pub trait TransactionOps: Send {
    async fn insert_batch(&mut self, articles: &[Article]) -> Result<()>;
    async fn delete_batch(&mut self, id: &HashSet<String>) -> Result<()>;
    async fn commit(self: Box<Self>) -> Result<()>;
}
