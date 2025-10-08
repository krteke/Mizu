use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    common::{AppState, PostResponse},
    some_errors::{GetPostsError, Result},
};

// 定义一个结构体 PostParams，用于接收文章列表请求的查询参数
// #[derive(Deserialize, Clone)] 是一个派生宏：
// - Deserialize: 允许这个结构体从查询字符串等格式中自动反序列化
// - Clone: 允许创建这个结构体的副本
#[derive(Deserialize, Clone)]
pub struct PostParams {
    // 文章分类
    category: String,
}

/// 异步处理函数，用于根据分类获取文章列表
///
/// # 参数
/// - `Query(params)`: 从请求的 URL 查询字符串中提取 `PostParams`。例如, /posts?category=article
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
    // 定义一个有效的文章分类列表，用于验证传入的参数
    let valid_categories = ["article", "note", "think", "pictures", "talk"];

    // 检查请求的分类是否存在于有效分类列表中
    if !valid_categories.contains(&params.category.as_str()) {
        // 如果分类无效，则返回一个分类错误的 `Error`
        return Err(GetPostsError::CategoryError.into());
    }

    // 从共享状态中获取数据库连接池
    let pool = &state.db_pool;
    // 使用 sqlx 的 `query_as!` 宏来执行数据库查询
    // 这个宏在编译时检查 SQL 查询的语法和类型，并将查询结果直接映射到 PostResponse 结构体
    let query_results = sqlx::query_as!(
        PostResponse,
        // SQL 查询语句：从 `articles` 表中选择 `title`, `tags`, 和 `content` 列
        // WHERE 子句根据传入的分类进行筛选
        "SELECT title, tags, content FROM articles WHERE category = $1",
        // 将 `params.category` 作为查询参数绑定到 $1
        &params.category
    )
    // `fetch_all` 执行查询并返回所有的结果行
    .fetch_all(pool)
    // `await` 等待异步数据库操作完成
    // `?` 操作符用于错误传播：如果数据库操作失败，它会立即返回一个错误
    .await?;

    // 如果查询成功，将结果 `query_results` 包装在 `Json` 中，然后包装在 `Ok` 中返回
    Ok(Json(query_results))
}

// 使用 #[cfg(test)] 宏，表示这个模块只在测试时编译
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::AppState, config::Config, search::SearchService};
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    // 辅助函数，用于创建一个用于测试的应用状态
    // 这个函数会连接到真实数据库和 Meilisearch 实例
    async fn setup_test_app_state() -> AppState {
        // 加载环境变量
        dotenvy::dotenv().ok();
        let config = Config::from_env().expect("Failed to load config for testing");

        // 创建数据库连接池
        let db_pool = PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&config.database_url)
            .await
            .expect("Failed to connect to database for testing");

        // 初始化 SearchService
        let search_service = SearchService::new(&config, "test_index".to_string())
            .await
            .expect("Failed to create SearchService for testing");

        AppState {
            db_pool,
            jwt_secret: config.jwt_secret,
            search_service,
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
            category: "article".to_string(),
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
        let params = PostParams {
            category: "invalid_category".to_string(),
        };
        let response = get_posts(Query(params), State(state.clone())).await;

        // 3. 断言
        assert!(response.is_err(), "Expected an error for invalid category");
        if let Err(e) = response {
            // 将错误转换为字符串进行比较
            let error_string = e.to_string();
            assert!(
                error_string.contains("Invalid Category type."),
                "Expected category error, but got: {}",
                error_string
            );
        }
    }
}
