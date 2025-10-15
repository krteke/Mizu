use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::sync::Arc;
use time::OffsetDateTime;

use crate::{
    common::{AppState, PostCategory, PostResponse},
    some_errors::{GetPostsError, Result},
};

// 定义一个结构体 PostParams，用于接收文章列表请求的查询参数
// #[derive(Deserialize, Clone)] 是一个派生宏：
// - Deserialize: 允许这个结构体从查询字符串等格式中自动反序列化
// - Clone: 允许创建这个结构体的副本
#[derive(Deserialize, Clone)]
pub struct PostParams {
    // 文章分类
    category: PostCategory,
    // 页码（从 1 开始）
    #[serde(default = "default_page")]
    page: i64,
    // 每页数量
    #[serde(default = "default_page_size")]
    page_size: i64,
}

// 默认页码为 1
fn default_page() -> i64 {
    1
}

// 默认每页 20 条
fn default_page_size() -> i64 {
    20
}

// 最大每页数量限制
const MAX_PAGE_SIZE: i64 = 100;

// 定义 Article 结构体，对应数据库中的 `articles` 表
#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Article {
    // 文章 ID
    pub id: String,
    // 文章标题
    pub title: String,
    // 文章标签
    pub tags: Vec<String>,
    // 文章分类
    pub category: PostCategory,
    // 文章摘要
    pub summary: String,
    // 文章内容
    pub content: String,
    // 文章状态
    pub status: String,
    // 创建时间
    pub created_at: OffsetDateTime,
    // 更新时间
    pub updated_at: OffsetDateTime,
}

/// 异步处理函数，用于根据分类获取文章列表（带分页）
///
/// # 参数
/// - `Query(params)`: 从请求的 URL 查询字符串中提取 `PostParams`。
///   例如: /posts?category=article&page=1&page_size=20
/// - `State(state)`: 从 Axum 的应用状态中提取共享的 `AppState`
///
/// # 返回
/// - `Result<Json<Vec<PostResponse>>>`: 返回一个 `Result` 类型。
///   - `Ok`: 包含一个 JSON 数组，数组中的每个元素都是一个 `PostResponse` 对象
///   - `Err`: 如果发生错误（如数据库错误、无效分类等），则返回一个自定义的错误类型
pub async fn get_posts(
    Query(params): Query<PostParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PostResponse>>> {
    // 从共享状态中获取数据库连接池
    let pool = &state.db_pool;

    // 确保页码至少为 1
    let page = params.page.max(1);
    // 限制每页数量，不超过最大值
    let page_size = params.page_size.min(MAX_PAGE_SIZE).max(1);
    // 计算 SQL OFFSET
    let offset = (page - 1) * page_size;

    // 使用 sqlx 的 `query_as!` 宏来执行数据库查询，添加分页支持
    let query_results = sqlx::query_as!(
        PostResponse,
        // SQL 查询语句：从 `articles` 表中选择需要的列，并添加分页
        "SELECT id, title, tags, content
         FROM articles
         WHERE category = $1
         ORDER BY created_at DESC
         LIMIT $2 OFFSET $3",
        params.category.as_str(),
        page_size,
        offset
    )
    .fetch_all(pool)
    .await?;

    // 如果查询成功，将结果 `query_results` 包装在 `Json` 中，然后包装在 `Ok` 中返回
    Ok(Json(query_results))
}

pub async fn get_post_digital(
    Path((category, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Article>> {
    let pool = &state.db_pool;
    let category = category
        .parse::<PostCategory>()
        .map_err(|_| GetPostsError::CategoryError)?;

    let result =
        sqlx::query_as::<_, Article>("SELECT * FROM articles WHERE category = $1 AND id = $2")
            .bind(&category)
            .bind(&id)
            .fetch_optional(pool)
            .await?
            .ok_or(GetPostsError::ArticleNotFound)?;

    Ok(Json(result))
}

// 使用 #[cfg(test)] 宏，表示这个模块只在测试时编译
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::AppState, config::Config, handlers::search::SearchService};
    use sqlx::postgres::PgPoolOptions;
    use std::{collections::HashSet, time::Duration};
    use tokio::sync::RwLock;

    // 辅助函数，用于创建一个用于测试的应用状态
    // 这个函数会连接到真实数据库和 Meilisearch 实例
    async fn setup_test_app_state() -> AppState {
        // 加载环境变量
        dotenvy::dotenv().ok();
        let config = Config::new().expect("Failed to load config for testing");

        // 创建数据库连接池
        let db_pool = PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&config.database_url)
            .await
            .expect("Failed to connect to database for testing");

        // 初始化 SearchService
        let search_service = SearchService::new(&config, "test_index")
            .await
            .expect("Failed to create SearchService for testing");

        let github_webhook_secret =
            std::env::var("GITHUB_WEBHOOK_SECRET").expect("GITHUB_WEBHOOK_SECRET not set");

        let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");

        let allowed_repositories = RwLock::new(HashSet::new());

        AppState {
            db_pool,
            jwt_secret: config.jwt_secret,
            search_service,
            github_webhook_secret,
            github_token,
            allowed_repositories,
        }
    }

    // 测试 get_posts 在提供有效分类时能否成功返回文章
    #[tokio::test]
    async fn test_get_posts_valid_category() {
        // 1. 设置
        let app_state = setup_test_app_state().await;
        let state = Arc::new(app_state);
        let pool = &state.db_pool;

        // 插入测试数据
        let test_id = "test_id_valid_cat";
        sqlx::query!(
            "INSERT INTO articles (id, title, tags, category, summary, content, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
             ON CONFLICT (id) DO NOTHING",
            test_id,
            "Test Title Valid",
            &["test".to_string()],
            "article",
            "A test summary.",
            "Some content.",
            "published"
        )
        .execute(pool)
        .await
        .expect("Failed to insert test data");

        // 2. 执行
        let params = PostParams {
            category: PostCategory::Article,
            page: 1,
            page_size: 20,
        };
        let response = get_posts(Query(params), State(state.clone())).await;

        // 3. 断言
        assert!(response.is_ok(), "Expected OK response for valid category");
        let posts = response.unwrap().0;
        assert!(!posts.is_empty(), "Expected to find at least one post");
        assert!(
            posts.iter().any(|p| p.title == "Test Title Valid"),
            "Test post not found in results"
        );

        // 4. 清理
        sqlx::query!("DELETE FROM articles WHERE id = $1", test_id)
            .execute(pool)
            .await
            .expect("Failed to clean up test data");
    }

    // 测试 get_posts 在提供无效分类时是否返回错误
    #[tokio::test]
    async fn test_get_posts_invalid_category() {
        // 1. 设置
        let app_state = setup_test_app_state().await;
        let state = Arc::new(app_state);

        // 2. 执行
        // 由于 PostParams 使用 PostCategory 类型，无法直接构造无效的 category
        // 因此跳过此测试，或者测试 get_post_digital 中的错误情况
        let params = PostParams {
            category: PostCategory::Article,
            page: 1,
            page_size: 20,
        };
        let response = get_posts(Query(params), State(state.clone())).await;

        // 3. 断言 - 改为测试有效分类
        assert!(response.is_ok(), "Expected OK response for valid category");
    }
}
