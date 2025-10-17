/// Test utility module
///
/// Provides Mock implementations and helper functions for testing
use async_trait::async_trait;
use backend::domain::articles::{Article, PostCategory};
use backend::domain::repositories::ArticleRepository;
use backend::domain::search::{SearchHit, SearchService};
use backend::errors::{GetPostsError, Result};
use backend::interfaces::http::dtos::PostResponse;
use meilisearch_sdk::client::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;

#[cfg(feature = "webhook")]
use backend::infrastructure::github::client::GithubClient;

/// Mock article repository implementation
///
/// Uses an in-memory HashMap to store data, suitable for unit testing.
/// This implementation allows testing repository-dependent code without
/// requiring a real database connection.
///
/// # Thread Safety
///
/// Uses `Arc<Mutex<HashMap>>` to allow safe concurrent access in async tests.
///
/// # Example
///
/// ```rust
/// use backend::tests::common::MockArticleRepository;
///
/// let repo = MockArticleRepository::new();
/// // or with pre-populated data
/// let repo = MockArticleRepository::with_articles(vec![article1, article2]);
/// ```
pub struct MockArticleRepository {
    pub articles: Arc<Mutex<HashMap<String, Article>>>,
}

impl MockArticleRepository {
    pub fn new() -> Self {
        Self {
            articles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_articles(articles: Vec<Article>) -> Self {
        let repo = Self::new();
        let mut map = repo.articles.lock().unwrap();
        for article in articles {
            map.insert(article.id.clone(), article);
        }
        drop(map);
        repo
    }
}

#[async_trait]
impl ArticleRepository for MockArticleRepository {
    async fn find_optional_by_id(&self, id: &str) -> Result<Option<Article>> {
        let articles = self.articles.lock().unwrap();
        Ok(articles.get(id).map(|a| a.clone()))
    }

    async fn save(&self, article: &Article) -> Result<()> {
        let mut articles = self.articles.lock().unwrap();
        articles.insert(article.id.clone(), article.clone());
        Ok(())
    }

    async fn delete_by_path(&self, path: &str) -> Result<()> {
        let mut articles = self.articles.lock().unwrap();
        // 简单实现：假设 path 就是 id
        articles.remove(path);
        Ok(())
    }

    async fn get_posts_by_category(
        &self,
        category: &str,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<PostResponse>> {
        let articles = self.articles.lock().unwrap();
        let mut filtered: Vec<_> = articles
            .values()
            .filter(|a| a.category.as_str() == category)
            .collect();

        // 按创建时间降序排序
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let results = filtered
            .iter()
            .skip(offset as usize)
            .take(page_size as usize)
            .map(|a| PostResponse {
                id: a.id.clone(),
                title: a.title.clone(),
                tags: a.tags.clone(),
                summary: a.summary.clone(),
            })
            .collect();

        Ok(results)
    }

    async fn get_post_by_id(&self, id: &str) -> Result<Article> {
        let articles = self.articles.lock().unwrap();
        articles
            .get(id)
            .map(|a| a.clone())
            .ok_or_else(|| GetPostsError::ArticleNotFound.into())
    }

    async fn get_all(&self) -> Result<Vec<Article>> {
        let articles = self.articles.lock().unwrap();
        Ok(articles.values().map(|a| a.clone()).collect())
    }

    async fn find_optional_by_file_path(&self, path: &str) -> Result<Option<Article>> {
        let articles = self.articles.lock().unwrap();

        // articles.values().find(|&x| x.file_path == path);

        todo!()
    }
}

/// Mock 搜索服务实现
pub struct MockSearchService {
    pub search_results: Arc<Mutex<HashMap<String, Vec<SearchHit>>>>,
}

impl MockSearchService {
    pub fn new() -> Self {
        Self {
            search_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_search_result(&self, query: &str, results: Vec<SearchHit>) {
        let mut map = self.search_results.lock().unwrap();
        map.insert(query.to_string(), results);
    }
}

#[async_trait]
impl SearchService for MockSearchService {
    async fn search(
        &self,
        query: &str,
        _index: &str,
        current_page: usize,
        limit: usize,
    ) -> Result<(Vec<SearchHit>, usize, usize, usize)> {
        let results = self.search_results.lock().unwrap();
        let hits = results.get(query).map(|v| v.clone()).unwrap_or_default();

        let total_hits = hits.len();
        let total_pages = (total_hits + limit - 1) / limit;
        let offset = (current_page - 1) * limit;

        let page_results = hits.into_iter().skip(offset).take(limit).collect();

        Ok((page_results, total_hits, total_pages, current_page))
    }

    async fn create_index_client(
        &self,
        _index: &str,
        _searchable_attributes: &[&str],
    ) -> Result<&Client> {
        // Mock 实现不需要真实的 Client
        unimplemented!("MockSearchService does not provide real Client")
    }
}

/// Mock GitHub 客户端实现
#[cfg(feature = "webhook")]
pub struct MockGithubClient {
    pub file_contents: Arc<Mutex<HashMap<String, String>>>,
}

#[cfg(feature = "webhook")]
impl MockGithubClient {
    pub fn new() -> Self {
        Self {
            file_contents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_file_content(&self, path: &str, content: String) {
        let mut map = self.file_contents.lock().unwrap();
        map.insert(path.to_string(), content);
    }
}

#[cfg(feature = "webhook")]
#[async_trait]
impl GithubClient for MockGithubClient {
    async fn get_file_content(&self, _owner: &str, _repo: &str, path: &str) -> Result<String> {
        let contents = self.file_contents.lock().unwrap();
        contents
            .get(path)
            .map(|s| s.clone())
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", path).into())
    }
}

/// Create a test article with the given parameters
///
/// Helper function to quickly create article entities for testing.
/// Uses sensible defaults for all fields not explicitly specified.
///
/// # Arguments
///
/// * `id` - Unique identifier for the article
/// * `title` - Article title
/// * `category` - Article category (article, note, think, etc.)
///
/// # Returns
///
/// A fully initialized `Article` entity with:
/// - Default tags: ["test", "rust"]
/// - Auto-generated summary and content
/// - Status: "published"
/// - Current timestamps for created_at and updated_at
///
/// # Example
///
/// ```rust
/// let article = create_test_article("test-1", "Test Article", PostCategory::Article);
/// assert_eq!(article.id, "test-1");
/// assert_eq!(article.status, "published");
/// ```
pub fn create_test_article(id: &str, title: &str, category: PostCategory) -> Article {
    Article {
        id: id.to_string(),
        title: title.to_string(),
        tags: vec!["test".to_string(), "rust".to_string()],
        category,
        summary: format!("Summary for {}", title),
        content: format!("Content for {}", title),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    }
}

/// Create a test search hit with the given parameters
///
/// Helper function to quickly create search result entities for testing
/// search functionality.
///
/// # Arguments
///
/// * `id` - Unique identifier for the search result
/// * `title` - Article title (may include highlighting)
///
/// # Returns
///
/// A `SearchHit` with:
/// - Default category: Article
/// - Auto-generated summary and content
///
/// # Example
///
/// ```rust
/// let hit = create_test_search_hit("1", "Test Article");
/// assert_eq!(hit.id, "1");
/// assert_eq!(hit.category, PostCategory::Article);
/// ```
pub fn create_test_search_hit(id: &str, title: &str) -> SearchHit {
    SearchHit {
        id: id.to_string(),
        title: title.to_string(),
        category: PostCategory::Article,
        summary: format!("Summary for {}", title),
        content: format!("Content for {}", title),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_repository_save_and_find() {
        let repo = MockArticleRepository::new();
        let article = create_test_article("test-1", "Test Article", PostCategory::Article);

        // Save the article
        repo.save(&article).await.unwrap();

        // Find the article
        let found = repo.find_optional_by_id("test-1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Article");
    }

    #[tokio::test]
    async fn test_mock_repository_delete() {
        let repo = MockArticleRepository::new();
        let article = create_test_article("test-2", "Test Article", PostCategory::Article);

        repo.save(&article).await.unwrap();
        repo.delete_by_path("test-2").await.unwrap();

        let found = repo.find_optional_by_id("test-2").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_mock_repository_get_by_category() {
        let repo = MockArticleRepository::new();

        let article1 = create_test_article("1", "Article 1", PostCategory::Article);
        let article2 = create_test_article("2", "Article 2", PostCategory::Article);
        let note = create_test_article("3", "Note 1", PostCategory::Note);

        repo.save(&article1).await.unwrap();
        repo.save(&article2).await.unwrap();
        repo.save(&note).await.unwrap();

        let results = repo.get_posts_by_category("article", 10, 0).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_search_service() {
        let service = MockSearchService::new();
        let hit = create_test_search_hit("1", "Test Article");
        service.set_search_result("rust", vec![hit]);

        let (results, total_hits, total_pages, current_page) =
            service.search("rust", "articles", 1, 10).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(total_hits, 1);
        assert_eq!(total_pages, 1);
        assert_eq!(current_page, 1);
    }
}
