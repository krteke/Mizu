use axum::{
    Json,
    extract::{Query, State},
};
use std::sync::Arc;

use crate::{
    app_state::AppState,
    domain::{articles::Article, search::SearchHit},
    errors::Result,
    interfaces::http::dtos::{SearchParams, SearchResponse},
};

// 定义每页显示的条目数
const PAGE_ITEMS: usize = 6;
// 定义默认的搜索索引名称
const DEFAULT_SEARCH_INDEX: &str = "articles";

// axum 的处理器函数，用于处理搜索请求
pub async fn get_search_results(
    State(state): State<Arc<AppState>>, // 从 axum 状态中提取共享的应用状态
    Query(params): Query<SearchParams>, // 从 URL 查询字符串中提取参数
) -> Result<Json<SearchResponse>> {
    // 如果查询字符串为空或只包含空格，则返回空的搜索结果
    if params.q.trim().is_empty() {
        return Ok(Json(SearchResponse {
            total_hits: 0,
            total_pages: 0,
            current_page: params.page,
            results: vec![],
        }));
    }

    // 确保当前页码至少为 1
    let current_page = params.page.max(1);
    // 根据页码和每页条目数计算偏移量
    let offset = PAGE_ITEMS * (current_page - 1);

    let search_result = state
        .search_service
        .get_search_result(DEFAULT_SEARCH_INDEX, PAGE_ITEMS, &params.q, offset)
        .await?;

    // 计算总命中数和总页数
    let total_hits = search_result.total_hits.unwrap_or(0);
    let total_pages = (total_hits + PAGE_ITEMS - 1) / PAGE_ITEMS;

    // 将 Meilisearch 的搜索结果映射到我们自定义的 SearchHit 结构体
    let results: Vec<SearchHit> = search_result
        .hits
        .into_iter()
        .map(|r| {
            // 创建一个默认的 SearchHit
            let mut hit_result = SearchHit {
                id: r.result.id.clone(),
                category: r.result.category.clone(),
                title: r.result.title.clone(),
                summary: String::new(),
                content: String::new(),
            };

            // 如果有格式化（高亮和裁剪）的结果，则使用它们
            if let Some(formatted) = &r.formatted_result {
                hit_result.summary = formatted
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                hit_result.content = formatted
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
            }

            hit_result
        })
        .collect();

    // 返回 JSON 格式的搜索响应
    Ok(Json(SearchResponse {
        total_hits,
        total_pages,
        current_page,
        results,
    }))
}

// 创建搜索索引并从数据库导入数据（通常用于初始化）
pub async fn create_search_index(State(state): State<Arc<AppState>>) -> Result<()> {
    let client = &state.search_service.admin_client;
    // 创建索引，并设置主键为 "id"
    client
        .create_index(DEFAULT_SEARCH_INDEX, Some("id"))
        .await?
        .wait_for_completion(&client, None, None)
        .await?;

    // 设置可搜索的属性
    let searchable_attributes = ["title", "content", "summary"];
    client
        .index(DEFAULT_SEARCH_INDEX)
        .set_filterable_attributes(&searchable_attributes)
        .await?;

    // 从数据库中获取所有文章
    let db_pool = &state.db_pool;
    let db_articles = sqlx::query_as::<_, Article>("SELECT * FROM articles")
        .fetch_all(db_pool)
        .await?;

    // 将所有文章添加到搜索索引中
    client
        .index(DEFAULT_SEARCH_INDEX)
        .add_documents(&db_articles, Some("id"))
        .await?
        .wait_for_completion(&client, None, None)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{config::Config, infrastructure::search::index::SearchService};
    use sqlx::PgPool;
    use time::OffsetDateTime;
    use tokio::sync::RwLock;

    // 辅助函数，设置测试环境
    async fn setup_test_app_state_for_search() -> AppState {
        dotenvy::dotenv().ok();
        let config = Config::new().expect("Failed to load config for testing");
        let search_service = SearchService::new(&config, DEFAULT_SEARCH_INDEX)
            .await
            .expect("Failed to create SearchService for testing");
        // 清理旧的测试数据
        let _ = search_service
            .admin_client
            .delete_index(DEFAULT_SEARCH_INDEX)
            .await;

        let db_pool = PgPool::connect(&config.database_url)
            .await
            .expect("Failed to connect to db");

        let github_webhook_secret =
            std::env::var("GITHUB_WEBHOOK_SECRET").expect("GITHUB_WEBHOOK_SECRET not set");

        let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");

        let allowed_repositories = RwLock::new(HashSet::new());

        AppState {
            db_pool,
            jwt_secret: config.jwt_secret.clone(),
            search_service,
            github_webhook_secret,
            github_token,
            allowed_repositories,
        }
    }
    // 辅助函数，创建一个测试文章
    fn create_test_article(id: &str, title: &str, content: &str) -> Article {
        Article {
            id: id.to_string(),
            title: title.to_string(),
            tags: vec!["rust".to_string(), "testing".to_string()],
            category: crate::domain::articles::PostCategory::Article,
            summary: "This is a test article about searching.".to_string(),
            content: content.to_string(),
            status: "published".to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    #[tokio::test]
    async fn test_get_search_results_with_query() {
        // 1. 设置
        let state = Arc::new(setup_test_app_state_for_search().await);
        let article = create_test_article(
            "search-id-1",
            "A Test Article",
            "This content is about Axum.",
        );

        // 添加文档到 Meilisearch
        state
            .search_service
            .update_or_add_index_item(&article)
            .await
            .unwrap();

        // 2. 执行
        let params = SearchParams {
            q: "Axum".to_string(),
            page: 1,
        };
        let response = get_search_results(State(state.clone()), Query(params)).await;

        // 3. 断言
        assert!(response.is_ok());
        let search_response = response.unwrap().0;
        assert_eq!(search_response.total_hits, 1);
        assert_eq!(search_response.results.len(), 1);
        assert_eq!(search_response.results[0].id, "search-id-1");
        assert!(
            search_response.results[0]
                .content
                .contains("<span class=\"highlight\">Axum</span>")
        );

        // 4. 清理
        state
            .search_service
            .delete_index_item(&article)
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn test_get_search_results_empty_query() {
        // 1. 设置
        let state = Arc::new(setup_test_app_state_for_search().await);

        // 2. 执行
        let params = SearchParams {
            q: " ".to_string(),
            page: 1,
        };
        let response = get_search_results(State(state.clone()), Query(params)).await;

        // 3. 断言
        assert!(response.is_ok());
        let search_response = response.unwrap().0;
        assert_eq!(search_response.total_hits, 0);
        assert!(search_response.results.is_empty());
    }
    #[tokio::test]
    async fn test_get_search_results_no_match() {
        // 1. 设置
        let state = Arc::new(setup_test_app_state_for_search().await);
        let article = create_test_article(
            "search-id-2",
            "Another Article",
            "Content about something else.",
        );
        state
            .search_service
            .update_or_add_index_item(&article)
            .await
            .unwrap();

        // 2. 执行
        let params = SearchParams {
            q: "nonexistentword".to_string(),
            page: 1,
        };
        let response = get_search_results(State(state.clone()), Query(params)).await;

        // 3. 断言
        assert!(response.is_ok());
        let search_response = response.unwrap().0;
        assert_eq!(search_response.total_hits, 0);
        assert!(search_response.results.is_empty());

        // 4. 清理
        state
            .search_service
            .delete_index_item(&article)
            .await
            .unwrap();
    }
}
