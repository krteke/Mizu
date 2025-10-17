//! Application Service layer tests
//! Unit tests using Mock implementations

mod common;

use backend::application::article_service::ArticleService;
use backend::config::AppConfig;
use backend::domain::articles::PostCategory;
use common::{
    MockArticleRepository, MockSearchService, create_test_article, create_test_search_hit,
};
use std::sync::Arc;

#[cfg(feature = "webhook")]
use common::MockGithubClient;
#[cfg(feature = "webhook")]
use std::collections::HashSet;

fn create_test_config() -> Arc<AppConfig> {
    Arc::new(AppConfig::new(
        "test_jwt_secret",
        #[cfg(feature = "webhook")]
        "test_webhook_secret",
        #[cfg(feature = "webhook")]
        "test_github_token",
        #[cfg(feature = "webhook")]
        HashSet::new(),
    ))
}

#[tokio::test]
async fn test_article_service_is_valid_id_new() {
    let repo = Arc::new(MockArticleRepository::new());
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    // New ID should be valid (available)
    assert!(service.is_valid_id("new-id").await);
}

#[tokio::test]
async fn test_article_service_is_valid_id_existing() {
    let article = create_test_article("existing-id", "Existing Article", PostCategory::Article);
    let repo = Arc::new(MockArticleRepository::with_articles(vec![article]));
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    // Existing ID should be invalid (not available)
    assert!(!service.is_valid_id("existing-id").await);
}

#[tokio::test]
async fn test_article_service_get_posts_by_category() {
    let article1 = create_test_article("1", "Article 1", PostCategory::Article);
    let article2 = create_test_article("2", "Article 2", PostCategory::Article);
    let note = create_test_article("3", "Note 1", PostCategory::Note);

    let repo = Arc::new(MockArticleRepository::with_articles(vec![
        article1, article2, note,
    ]));
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    let results = service
        .get_posts_by_category("article", 10, 0)
        .await
        .unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_article_service_get_posts_by_category_pagination() {
    let articles: Vec<_> = (0..25)
        .map(|i| {
            create_test_article(
                &format!("article-{}", i),
                &format!("Article {}", i),
                PostCategory::Article,
            )
        })
        .collect();

    let repo = Arc::new(MockArticleRepository::with_articles(articles));
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    // 第一页：10 条
    let page1 = service
        .get_posts_by_category("article", 10, 0)
        .await
        .unwrap();
    assert_eq!(page1.len(), 10);

    // 第二页：10 条
    let page2 = service
        .get_posts_by_category("article", 10, 10)
        .await
        .unwrap();
    assert_eq!(page2.len(), 10);

    // 第三页：5 条
    let page3 = service
        .get_posts_by_category("article", 10, 20)
        .await
        .unwrap();
    assert_eq!(page3.len(), 5);
}

#[tokio::test]
async fn test_article_service_get_posts_empty_category() {
    let repo = Arc::new(MockArticleRepository::new());
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    let results = service
        .get_posts_by_category("article", 10, 0)
        .await
        .unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_article_service_get_article_by_id() {
    let article = create_test_article("test-123", "Test Article", PostCategory::Article);
    let repo = Arc::new(MockArticleRepository::with_articles(vec![article]));
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    let found = service.get_article_by_id("test-123").await.unwrap();
    assert_eq!(found.title, "Test Article");
    assert_eq!(found.id, "test-123");
}

#[tokio::test]
async fn test_article_service_get_article_by_id_not_found() {
    let repo = Arc::new(MockArticleRepository::new());
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    let result = service.get_article_by_id("non-existent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_article_service_search() {
    let repo = Arc::new(MockArticleRepository::new());
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    // 设置搜索结果
    let hit1 = create_test_search_hit("1", "Rust Programming");
    let hit2 = create_test_search_hit("2", "Rust Web Development");
    search.set_search_result("rust", vec![hit1, hit2]);

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    let (results, total_hits, total_pages, current_page) =
        service.search("articles", "rust", 1, 10).await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(total_hits, 2);
    assert_eq!(total_pages, 1);
    assert_eq!(current_page, 1);
}

#[tokio::test]
async fn test_article_service_search_pagination() {
    let repo = Arc::new(MockArticleRepository::new());
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    // 设置 15 个搜索结果
    let hits: Vec<_> = (0..15)
        .map(|i| create_test_search_hit(&format!("id-{}", i), &format!("Article {}", i)))
        .collect();
    search.set_search_result("test", hits);

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    // 第一页：5 条
    let (results1, total1, pages1, page1) = service.search("articles", "test", 1, 5).await.unwrap();
    assert_eq!(results1.len(), 5);
    assert_eq!(total1, 15);
    assert_eq!(pages1, 3);
    assert_eq!(page1, 1);

    // 第二页：5 条
    let (results2, _total2, _pages2, page2) =
        service.search("articles", "test", 2, 5).await.unwrap();
    assert_eq!(results2.len(), 5);
    assert_eq!(page2, 2);

    // 第三页：5 条
    let (results3, _total3, _pages3, page3) =
        service.search("articles", "test", 3, 5).await.unwrap();
    assert_eq!(results3.len(), 5);
    assert_eq!(page3, 3);
}

#[tokio::test]
async fn test_article_service_search_empty_query() {
    let repo = Arc::new(MockArticleRepository::new());
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    let (results, total_hits, _, _) = service
        .search("articles", "nonexistent", 1, 10)
        .await
        .unwrap();

    assert_eq!(results.len(), 0);
    assert_eq!(total_hits, 0);
}

#[tokio::test]
async fn test_article_service_search_with_different_page_sizes() {
    let repo = Arc::new(MockArticleRepository::new());
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    // 设置 20 个搜索结果
    let hits: Vec<_> = (0..20)
        .map(|i| create_test_search_hit(&format!("id-{}", i), &format!("Article {}", i)))
        .collect();
    search.set_search_result("test", hits);

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    // 测试不同的页面大小
    let (results_10, _, pages_10, _) = service.search("articles", "test", 1, 10).await.unwrap();
    assert_eq!(results_10.len(), 10);
    assert_eq!(pages_10, 2);

    let (results_5, _, pages_5, _) = service.search("articles", "test", 1, 5).await.unwrap();
    assert_eq!(results_5.len(), 5);
    assert_eq!(pages_5, 4);

    let (results_20, _, pages_20, _) = service.search("articles", "test", 1, 20).await.unwrap();
    assert_eq!(results_20.len(), 20);
    assert_eq!(pages_20, 1);
}

#[tokio::test]
async fn test_article_service_multiple_categories() {
    let articles = vec![
        create_test_article("a1", "Article 1", PostCategory::Article),
        create_test_article("a2", "Article 2", PostCategory::Article),
        create_test_article("n1", "Note 1", PostCategory::Note),
        create_test_article("n2", "Note 2", PostCategory::Note),
        create_test_article("t1", "Think 1", PostCategory::Think),
    ];

    let repo = Arc::new(MockArticleRepository::with_articles(articles));
    let search = Arc::new(MockSearchService::new());
    let config = create_test_config();

    let service = ArticleService::new(
        repo,
        #[cfg(feature = "webhook")]
        Arc::new(MockGithubClient::new()),
        search,
        config,
    );

    // 测试不同分类
    let articles = service
        .get_posts_by_category("article", 10, 0)
        .await
        .unwrap();
    assert_eq!(articles.len(), 2);

    let notes = service.get_posts_by_category("note", 10, 0).await.unwrap();
    assert_eq!(notes.len(), 2);

    let thinks = service.get_posts_by_category("think", 10, 0).await.unwrap();
    assert_eq!(thinks.len(), 1);
}
