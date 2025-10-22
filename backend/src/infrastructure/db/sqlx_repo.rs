use std::collections::HashSet;

use async_trait::async_trait;
use sqlx::{Acquire, Postgres, Transaction, types::Uuid};

use crate::{
    domain::{
        articles::Article,
        repositories::{ArticleRepository, TransactionGuard, TransactionOps},
    },
    errors::{GetPostsError, Result},
    interfaces::http::dtos::PostResponse,
};

/// SQLx-based implementation of the ArticleRepository trait
///
/// This struct provides PostgreSQL database access for article operations using
/// the SQLx async database library. It implements the ArticleRepository trait
/// defined in the domain layer, following the Repository pattern and Dependency
/// Inversion Principle.
///
/// # Architecture
///
/// This implementation sits in the infrastructure layer and provides concrete
/// database operations while the trait definition remains in the domain layer.
/// This allows the domain and application layers to remain independent of
/// specific database technologies.
///
/// # Database Schema
///
/// This repository expects the following table structure:
///
/// ```sql
/// CREATE TABLE articles (
///     id TEXT PRIMARY KEY,
///     title TEXT NOT NULL,
///     tags TEXT[] NOT NULL DEFAULT '{}',
///     category TEXT NOT NULL,
///     summary TEXT NOT NULL DEFAULT '',
///     content TEXT NOT NULL,
///     status TEXT NOT NULL DEFAULT 'draft',
///     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
///     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
/// );
/// ```
///
/// # Example
///
/// ```rust
/// use sqlx::PgPool;
/// use backend::infrastructure::db::sqlx_repo::SqlxArticleRepository;
///
/// let pool = PgPool::connect("postgres://...").await?;
/// let repo = SqlxArticleRepository::new(pool);
/// ```
pub struct SqlxArticleRepository {
    /// PostgreSQL connection pool for executing queries
    ///
    /// The pool manages a set of database connections that can be reused
    /// across multiple requests, improving performance and resource utilization.
    pool: sqlx::PgPool,
}

impl SqlxArticleRepository {
    /// Create a new SqlxArticleRepository instance
    ///
    /// # Arguments
    ///
    /// * `pool` - A PostgreSQL connection pool from SQLx
    ///
    /// # Returns
    ///
    /// A new repository instance ready to execute database operations
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx::PgPool;
    /// use backend::infrastructure::db::sqlx_repo::SqlxArticleRepository;
    ///
    /// let pool = PgPool::connect("postgres://localhost/mydb").await?;
    /// let repo = SqlxArticleRepository::new(pool);
    /// ```
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ArticleRepository for SqlxArticleRepository {
    /// Retrieve a paginated list of articles filtered by category
    ///
    /// Fetches articles belonging to a specific category with pagination support.
    /// Results are ordered by creation date in descending order (newest first).
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
    /// * `Err(SomeError)` - Database query failed
    ///
    /// # SQL Query
    ///
    /// ```sql
    /// SELECT id, title, tags, summary
    /// FROM articles
    /// WHERE category = $1
    /// ORDER BY created_at DESC
    /// LIMIT $2 OFFSET $3
    /// ```
    ///
    /// # Note
    ///
    /// Returns PostResponse DTOs rather than full Article entities for efficiency,
    /// as list views typically don't need the complete article content.
    async fn get_posts_by_category(
        &self,
        category: &str,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<PostResponse>> {
        let query_results = sqlx::query_as!(
            PostResponse,
            // SQL query: Select required columns from articles table with pagination
            // Filters by category and orders by creation date (newest first)
            "SELECT id, title, tags, summary
             FROM articles
             WHERE category = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
            category,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(query_results)
    }

    /// Retrieve a single article by its unique identifier
    ///
    /// Fetches the complete article entity from the database. Unlike
    /// `find_optional_by_id`, this method returns an error if the article
    /// is not found, making it suitable for cases where the article is
    /// expected to exist (e.g., when handling a direct article request).
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the article
    ///
    /// # Returns
    ///
    /// * `Ok(Article)` - The complete article entity
    /// * `Err(GetPostsError::ArticleNotFound)` - Article doesn't exist
    /// * `Err(SomeError)` - Database query failed
    ///
    /// # SQL Query
    ///
    /// ```sql
    /// SELECT * FROM articles WHERE id = $1
    /// ```
    async fn get_post_by_id(&self, id: &str) -> Result<Article> {
        let result = sqlx::query_as::<_, Article>("SELECT * FROM articles WHERE id = $1")
            .bind(&id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| GetPostsError::ArticleNotFound)?;

        Ok(result)
    }

    /// Retrieve all articles from the database
    ///
    /// Fetches every article without any filtering or pagination. This is
    /// primarily used for administrative operations like rebuilding the search
    /// index or generating complete sitemaps.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Article>)` - List of all articles (may be empty)
    /// * `Err(SomeError)` - Database query failed
    ///
    /// # SQL Query
    ///
    /// ```sql
    /// SELECT * FROM articles
    /// ```
    ///
    /// # Performance Warning
    ///
    /// This method loads all articles into memory at once. For databases with
    /// thousands of articles, this could cause memory issues. Consider using
    /// pagination or streaming for large datasets in production.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Rebuild search index with all articles
    /// let all_articles = repo.get_all().await?;
    /// search_service.index_documents(&all_articles).await?;
    /// ```
    async fn get_all(&self) -> Result<Vec<Article>> {
        let db_items = sqlx::query_as::<_, Article>("SELECT * FROM articles")
            .fetch_all(&self.pool)
            .await?;

        Ok(db_items)
    }

    async fn get_by_paths(&self, paths: &[String]) -> Result<HashSet<String>> {
        let results = sqlx::query!("SELECT id FROM articles WHERE path = ANY($1)", paths)
            .fetch_all(&self.pool)
            .await?;

        let id_set: HashSet<String> = results.into_iter().map(|row| row.id.to_string()).collect();

        Ok(id_set)
    }

    async fn begin_transaction(&self) -> Result<TransactionGuard> {
        let tx = self.pool.begin().await?;

        Ok(TransactionGuard {
            inner: Box::new(SqlxTransaction { tx }),
        })
    }
}

struct SqlxTransaction {
    tx: Transaction<'static, Postgres>,
}

#[async_trait]
impl TransactionOps for SqlxTransaction {
    async fn upsert_batch(&mut self, articles: &[Article]) -> Result<()> {
        if articles.is_empty() {
            return Ok(());
        }

        let mut query = sqlx::QueryBuilder::new(
            "INSERT INTO articles (id, path, title, tags, category, summary, content, status, created_at, updated_at) ",
        );
        query.push_values(articles, |mut b, article| {
            b.push_bind(&article.id);
            b.push_bind(&article.path);
            b.push_bind(&article.title);
            b.push_bind(&article.tags);
            b.push_bind(&article.category);
            b.push_bind(&article.summary);
            b.push_bind(&article.content);
            b.push_bind(&article.status);
            b.push_bind(article.created_at);
            b.push_bind(article.updated_at);
        });
        query.push(
            " ON CONFLICT (id) DO UPDATE SET \
                    title = EXCLUDED.title, \
                    path = EXCLUDED.path, \
                    tags = EXCLUDED.tags, \
                    category = EXCLUDED.category, \
                    summary = EXCLUDED.summary, \
                    content = EXCLUDED.content, \
                    status = EXCLUDED.status, \
                    updated_at = EXCLUDED.updated_at",
        );
        query.build().execute(self.tx.acquire().await?).await?;

        Ok(())
    }

    async fn delete_batch(&mut self, id: &HashSet<String>) -> Result<()> {
        if id.is_empty() {
            return Ok(());
        }

        let del_id: Vec<Uuid> = id
            .into_iter()
            .filter_map(|p| Uuid::parse_str(&p).ok())
            .collect();

        sqlx::query!("DELETE FROM articles WHERE id = ANY($1)", &del_id)
            .execute(self.tx.acquire().await?)
            .await?;

        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<()> {
        self.tx.commit().await?;

        Ok(())
    }
}
