//! Domain layer unit tests
//! Tests domain models, enum conversions, and pure business logic

use backend::domain::articles::{Article, PostCategory};
use std::str::FromStr;
use time::OffsetDateTime;

#[test]
fn test_post_category_as_str() {
    assert_eq!(PostCategory::Article.as_str(), "article");
    assert_eq!(PostCategory::Note.as_str(), "note");
    assert_eq!(PostCategory::Think.as_str(), "think");
    assert_eq!(PostCategory::Pictures.as_str(), "pictures");
    assert_eq!(PostCategory::Talk.as_str(), "talk");
}

#[test]
fn test_post_category_from_str() {
    assert_eq!(
        PostCategory::from_str("article").unwrap(),
        PostCategory::Article
    );
    assert_eq!(PostCategory::from_str("note").unwrap(), PostCategory::Note);
    assert_eq!(
        PostCategory::from_str("think").unwrap(),
        PostCategory::Think
    );
    assert_eq!(
        PostCategory::from_str("pictures").unwrap(),
        PostCategory::Pictures
    );
    assert_eq!(PostCategory::from_str("talk").unwrap(), PostCategory::Talk);

    // Test invalid inputs
    assert!(PostCategory::from_str("invalid").is_err());
    assert!(PostCategory::from_str("ARTICLE").is_err()); // Case-sensitive
}

#[test]
fn test_post_category_roundtrip() {
    let categories = vec![
        PostCategory::Article,
        PostCategory::Note,
        PostCategory::Think,
        PostCategory::Pictures,
        PostCategory::Talk,
    ];

    for category in categories {
        let str_repr = category.as_str();
        let parsed = PostCategory::from_str(str_repr).unwrap();
        assert_eq!(parsed, category);
    }
}

#[test]
fn test_post_category_serialization() {
    use serde_json;

    // Test serialization
    let category = PostCategory::Article;
    let json = serde_json::to_string(&category).unwrap();
    assert_eq!(json, "\"article\"");

    // Test deserialization
    let deserialized: PostCategory = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, PostCategory::Article);
}

#[test]
fn test_article_creation() {
    let article = Article {
        id: "test-123".to_string(),
        title: "Test Article".to_string(),
        tags: vec!["rust".to_string(), "testing".to_string()],
        category: PostCategory::Article,
        summary: "Test summary".to_string(),
        content: "Test content".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };

    assert_eq!(article.id, "test-123");
    assert_eq!(article.title, "Test Article");
    assert_eq!(article.tags.len(), 2);
    assert_eq!(article.category, PostCategory::Article);
}

#[test]
fn test_article_serialization() {
    use serde_json;

    let article = Article {
        id: "test-456".to_string(),
        title: "Serialization Test".to_string(),
        tags: vec!["test".to_string()],
        category: PostCategory::Note,
        summary: "Summary".to_string(),
        content: "Content".to_string(),
        status: "draft".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };

    // Test serialization
    let json = serde_json::to_string(&article).unwrap();
    assert!(json.contains("test-456"));
    assert!(json.contains("Serialization Test"));
    assert!(json.contains("note")); // Category should be serialized as lowercase

    // Test deserialization
    let deserialized: Article = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, article.id);
    assert_eq!(deserialized.category, PostCategory::Note);
}

#[test]
fn test_article_with_empty_tags() {
    let article = Article {
        id: "no-tags".to_string(),
        title: "Article Without Tags".to_string(),
        tags: vec![],
        category: PostCategory::Think,
        summary: "Summary".to_string(),
        content: "Content".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };

    assert_eq!(article.tags.len(), 0);
    assert!(article.tags.is_empty());
}

#[test]
fn test_article_with_multiple_tags() {
    let tags = vec![
        "rust".to_string(),
        "web".to_string(),
        "backend".to_string(),
        "axum".to_string(),
    ];

    let article = Article {
        id: "multi-tags".to_string(),
        title: "Article With Multiple Tags".to_string(),
        tags: tags.clone(),
        category: PostCategory::Article,
        summary: "Summary".to_string(),
        content: "Content".to_string(),
        status: "published".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };

    assert_eq!(article.tags.len(), 4);
    assert_eq!(article.tags, tags);
}

#[test]
fn test_article_status_values() {
    let statuses = vec!["draft", "published", "archived"];

    for status in statuses {
        let article = Article {
            id: format!("article-{}", status),
            title: "Test".to_string(),
            tags: vec![],
            category: PostCategory::Article,
            summary: "Summary".to_string(),
            content: "Content".to_string(),
            status: status.to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };

        assert_eq!(article.status, status);
    }
}

#[test]
fn test_all_post_categories() {
    let categories = vec![
        PostCategory::Article,
        PostCategory::Note,
        PostCategory::Think,
        PostCategory::Pictures,
        PostCategory::Talk,
    ];

    assert_eq!(categories.len(), 5);

    // Ensure each category has a unique string representation
    let str_categories: Vec<&str> = categories.iter().map(|c| c.as_str()).collect();
    let mut unique = str_categories.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), str_categories.len());
}
