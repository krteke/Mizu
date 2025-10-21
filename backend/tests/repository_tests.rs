//! Repository layer integration tests
//! Requires a real database connection
//! Set DATABASE_URL environment variable before running these tests

mod common;

use backend::domain::articles::{Article, PostCategory};
use backend::domain::repositories::ArticleRepository;
use backend::infrastructure::db::sqlx_repo::SqlxArticleRepository;
use sqlx::PgPool;
use time::OffsetDateTime;

/// Set up test database
/// Note: This requires a real database connection
async fn setup_test_db() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Clean up test data
async fn cleanup_test_article(pool: &PgPool, id: &str) {
    let _ = sqlx::query("DELETE FROM articles WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await;
}

#[tokio::test]
#[ignore] // Ignored by default, run manually when database is available: cargo test -- --ignored
async fn test_sqlx_repo_find_optional_by_id_exists() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool.clone());

    let article = Article {
        id: "test-find-exists".to_string(),
        title: "Test Find Exists".to_string(),
        tags: vec!["test".to_string()],
        category: PostCategory::Article,
        summary: Some("Test summary".to_string()),
        content: "Test content".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        deleted_at: None,
    };

    // Save the article
    repo.save(&[article]).await.unwrap();

    // Find the article
    let found = repo.find_optional_by_id("test-find-exists").await.unwrap();
    assert!(found.is_some());
    let found_article = found.unwrap();
    assert_eq!(found_article.title, "Test Find Exists");
    assert_eq!(found_article.id, "test-find-exists");

    // Clean up
    cleanup_test_article(&pool, "test-find-exists").await;
}

#[tokio::test]
#[ignore]
async fn test_sqlx_repo_find_optional_by_id_not_exists() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool);

    let found = repo
        .find_optional_by_id("definitely-does-not-exist")
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
#[ignore]
async fn test_sqlx_repo_get_by_category() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool.clone());

    // 创建测试文章
    let article1 = Article {
        id: "test-category-1".to_string(),
        title: "Article 1".to_string(),
        tags: vec!["rust".to_string()],
        category: PostCategory::Article,
        summary: Some("Summary 1".to_string()),
        content: "Content 1".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        deleted_at: None,
    };

    let article2 = Article {
        id: "test-category-2".to_string(),
        title: "Article 2".to_string(),
        tags: vec!["web".to_string()],
        category: PostCategory::Article,
        summary: Some("Summary 2".to_string()),
        content: "Content 2".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        deleted_at: None,
    };

    let note = Article {
        id: "test-note-1".to_string(),
        title: "Note 1".to_string(),
        tags: vec![],
        category: PostCategory::Note,
        summary: Some("Note summary".to_string()),
        content: "Note content".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        deleted_at: None,
    };

    repo.save(&[article1, article2, note]).await.unwrap();

    // Query for article category
    let results = repo.get_posts_by_category("article", 10, 0).await.unwrap();
    assert!(results.len() >= 2);

    // Verify that article category was returned
    let our_articles: Vec<_> = results
        .iter()
        .filter(|a| a.id == "test-category-1" || a.id == "test-category-2")
        .collect();
    assert_eq!(our_articles.len(), 2);

    // Clean up
    cleanup_test_article(&pool, "test-category-1").await;
    cleanup_test_article(&pool, "test-category-2").await;
    cleanup_test_article(&pool, "test-note-1").await;
}

#[tokio::test]
#[ignore]
async fn test_sqlx_repo_get_by_category_pagination() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool.clone());

    // Create 5 articles
    for i in 0..5 {
        let article = Article {
            id: format!("test-pagination-{}", i),
            title: format!("Pagination Test {}", i),
            tags: vec![],
            category: PostCategory::Think,
            summary: Some(format!("Summary {}", i)),
            content: format!("Content {}", i),
            status: "published".to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            deleted_at: None,
        };
        repo.save(&[article]).await.unwrap();
    }

    // First page: get 3 items
    let page1 = repo.get_posts_by_category("think", 3, 0).await.unwrap();
    assert!(page1.len() >= 3);

    // Second page: get 2 items
    let page2 = repo.get_posts_by_category("think", 3, 3).await.unwrap();
    assert!(page2.len() >= 2);

    // Clean up
    for i in 0..5 {
        cleanup_test_article(&pool, &format!("test-pagination-{}", i)).await;
    }
}

#[tokio::test]
#[ignore]
async fn test_sqlx_repo_get_by_id() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool.clone());

    let article = Article {
        id: "test-get-by-id".to_string(),
        title: "Get By ID Test".to_string(),
        tags: vec!["rust".to_string(), "testing".to_string()],
        category: PostCategory::Note,
        summary: Some("Summary".to_string()),
        content: "Content".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        deleted_at: None,
    };

    repo.save(&[article]).await.unwrap();

    let found = repo.get_post_by_id("test-get-by-id").await.unwrap();
    assert_eq!(found.id, "test-get-by-id");
    assert_eq!(found.title, "Get By ID Test");
    assert_eq!(found.category, PostCategory::Note);
    assert_eq!(found.tags.len(), 2);

    // Clean up
    cleanup_test_article(&pool, "test-get-by-id").await;
}

#[tokio::test]
#[ignore]
async fn test_sqlx_repo_article_not_found() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool);

    let result = repo.get_post_by_id("non-existent-id").await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_sqlx_repo_get_all() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool.clone());

    // Create test article
    let article = Article {
        id: "test-get-all".to_string(),
        title: "Get All Test".to_string(),
        tags: vec![],
        category: PostCategory::Talk,
        summary: Some("Summary".to_string()),
        content: "Content".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        deleted_at: None,
    };

    repo.save(&[article]).await.unwrap();

    let all_articles = repo.get_all().await.unwrap();
    assert!(!all_articles.is_empty());

    // Verify our article is in the list
    let found = all_articles.iter().any(|a| a.id == "test-get-all");
    assert!(found);

    // Clean up
    cleanup_test_article(&pool, "test-get-all").await;
}

#[tokio::test]
#[ignore]
async fn test_sqlx_repo_delete_by_path() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool.clone());

    let article = Article {
        id: "test-delete".to_string(),
        title: "Delete Test".to_string(),
        tags: vec![],
        category: PostCategory::Pictures,
        summary: Some("Summary".to_string()),
        content: "Content".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        deleted_at: None,
    };

    // Save article
    repo.save(&[article]).await.unwrap();

    // Verify article exists
    let exists = repo.find_optional_by_id("test-delete").await.unwrap();
    assert!(exists.is_some());

    // Delete article
    repo.delete_by_path("test-delete").await.unwrap();

    // Verify article was deleted
    let deleted = repo.find_optional_by_id("test-delete").await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
#[ignore]
async fn test_sqlx_repo_save_updates_existing() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool.clone());

    // Create original article
    let mut article = Article {
        id: "test-update".to_string(),
        title: "Original Title".to_string(),
        tags: vec!["original".to_string()],
        category: PostCategory::Article,
        summary: Some("Original summary".to_string()),
        content: "Original content".to_string(),
        status: "draft".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        deleted_at: None,
    };

    repo.save(&[article.clone()]).await.unwrap();

    // Update article
    article.title = "Updated Title".to_string();
    article.status = "published".to_string();
    article.updated_at = OffsetDateTime::now_utc();

    repo.update(&[article]).await.unwrap();

    // Verify update
    let updated = repo.get_post_by_id("test-update").await.unwrap();
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.status, "published");

    // Clean up
    cleanup_test_article(&pool, "test-update").await;
}

#[tokio::test]
#[ignore]
async fn test_sqlx_repo_multiple_categories() {
    let pool = setup_test_db().await;
    let repo = SqlxArticleRepository::new(pool.clone());

    // Create articles with different categories
    let categories = vec![
        ("test-multi-article", PostCategory::Article),
        ("test-multi-note", PostCategory::Note),
        ("test-multi-think", PostCategory::Think),
        ("test-multi-pictures", PostCategory::Pictures),
        ("test-multi-talk", PostCategory::Talk),
    ];

    for (id, category) in &categories {
        let article = Article {
            id: id.to_string(),
            title: format!("Test {}", id),
            tags: vec![],
            category: category.clone(),
            summary: Some("Summary".to_string()),
            content: "Content".to_string(),
            status: "published".to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            deleted_at: None,
        };
        repo.save(&[article]).await.unwrap();
    }

    // Verify each category can be queried
    for (id, category) in &categories {
        let results = repo
            .get_posts_by_category(category.as_str(), 10, 0)
            .await
            .unwrap();
        let found = results.iter().any(|a| a.id == *id);
        assert!(
            found,
            "Should find article in category {}",
            category.as_str()
        );
    }

    // Clean up
    for (id, _) in &categories {
        cleanup_test_article(&pool, id).await;
    }
}
