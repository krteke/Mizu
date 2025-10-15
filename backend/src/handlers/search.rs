use axum::{
    Json,
    extract::{Query, State},
};
use meilisearch_sdk::{client::Client, key::Key, search::Selectors};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    common::{AppState, SearchHit},
    config::Config,
    handlers::articles::Article,
    some_errors::{Result, SearchError},
};

// 定义每页显示的条目数
const PAGE_ITEMS: usize = 6;
// 定义默认的搜索索引名称
const DEFAULT_SEARCH_INDEX: &str = "articles";

// 定义一个枚举来区分不同权限的 Meilisearch 客户端
#[derive(Debug, Clone, Copy)]
pub enum ClientType {
    Search, // 仅用于搜索
    Admin,  // 用于管理操作，如添加/删除文档
}

// 定义搜索服务结构体，封装了与 Meilisearch 交互的逻辑
#[derive(Clone)]
pub struct SearchService {
    pub admin_client: Client,  // 具有管理权限的 Meilisearch 客户端
    pub search_client: Client, // 仅具有搜索权限的 Meilisearch 客户端
    pub index_name: String,    // 索引的名称
}

impl SearchService {
    // SearchService 的构造函数，用于创建一个新的实例
    pub async fn new(config: &Config, index_name: &str) -> Result<Self> {
        // 获取 Meilisearch 服务的 URL
        let meili_search_url = &config.meilisearch_url;

        // 使用主密钥创建一个临时的 master 客户端，用于获取其他 API Key
        let master_client = &Self::create_master_client(config)?;
        // 检查 Meilisearch 服务是否健康
        master_client.health().await?;

        // 获取管理员 API Key 和 搜索 API Key
        let admin_key = get_standard_key(master_client, StandardApiKey::Admin).await?;
        let search_key = get_standard_key(master_client, StandardApiKey::Search).await?;

        // 使用获取到的 Key 分别创建具有不同权限的客户端
        let admin_client = Client::new(meili_search_url, Some(admin_key))?;
        let search_client = Client::new(meili_search_url, Some(search_key))?;

        // 返回初始化完成的 SearchService 实例
        Ok(Self {
            admin_client,
            search_client,
            index_name: index_name.to_string(),
        })
    }

    // 更新或添加索引中的一篇文章
    pub async fn update_or_add_index_item(&self, article: &Article) -> Result<()> {
        let client = &self.admin_client;
        let index = client.index(&self.index_name);

        // 添加或替换文档，使用 "id" 作为主键
        index
            .add_or_replace(&[article], Some("id"))
            .await?
            // 等待 Meilisearch 处理完成该任务
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(())
    }

    // 从索引中删除一篇文章
    pub async fn delete_index_item(&self, article: &Article) -> Result<()> {
        let client = &self.admin_client;
        let index = client.index(&self.index_name);

        // 根据文章 ID 删除文档
        index
            .delete_document(&article.id)
            .await?
            // 等待 Meilisearch 处理完成该任务
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(())
    }

    // 创建一个使用主密钥的 Meilisearch 客户端
    fn create_master_client(config: &Config) -> Result<Client> {
        let meili_search_url = &config.meilisearch_url;
        let key = &config.meili_master_key;

        // 使用 URL 和主密钥初始化客户端
        Ok(Client::new(meili_search_url, Some(key))?)
    }
}

// 定义一个枚举来表示不同类型的 API Key
#[derive(Debug, Clone, Copy)]
enum StandardApiKey {
    Search, // 搜索 Key
    Admin,  // 管理 Key
}

// 定义搜索请求的查询参数结构体
#[derive(Deserialize, Debug)]
pub struct SearchParams {
    q: String,   // 搜索查询字符串
    page: usize, // 请求的页码
}

// 定义搜索响应的结构体，将被序列化为 JSON
#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    pub total_hits: usize,       // 匹配的总命中数
    pub total_pages: usize,      // 总页数
    pub current_page: usize,     // 当前页码
    pub results: Vec<SearchHit>, // 搜索结果列表
}

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

    // 获取搜索客户端并指定要搜索的索引
    let index = &state
        .search_service
        .search_client
        .index(DEFAULT_SEARCH_INDEX);

    // 构建搜索查询
    let search_result = index
        .search()
        .with_query(&params.q) // 设置查询词
        .with_offset(offset) // 设置偏移量
        .with_limit(PAGE_ITEMS) // 设置每页限制
        .with_attributes_to_highlight(Selectors::Some(&["title", "summary", "content"])) // 设置高亮的字段
        .with_highlight_pre_tag("<span class=\"highlight\">") // 设置高亮前缀标签
        .with_highlight_post_tag("</span>") // 设置高亮后缀标签
        .with_attributes_to_crop(Selectors::Some(&[("summary", None), ("content", None)])) // 设置要裁剪的字段
        .execute::<Article>() // 执行搜索
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

// 异步函数，用于获取所有的 API Key
async fn get_api_keys(config: &Config) -> Result<Vec<Key>> {
    let client = SearchService::create_master_client(config)?;
    Ok(client.get_keys().await?.results)
}

// 异步函数，根据类型获取特定的 API Key
async fn get_standard_key(client: &Client, key_type: StandardApiKey) -> Result<String> {
    // 根据 key_type 确定要查找的 Key 的名称
    let key_name = match &key_type {
        StandardApiKey::Admin => "Default Admin API Key",
        StandardApiKey::Search => "Default Search API Key",
    };

    // 获取所有的 Key
    let keys = client.get_keys().await?.results;

    // 查找具有指定名称的 Key
    if let Some(key) = keys.iter().find(|&k| k.name.as_deref() == Some(key_name)) {
        return Ok(key.key.clone());
    }

    // 如果找不到，则返回错误
    match key_type {
        StandardApiKey::Admin => Err(SearchError::DefaultAdminApiKeyNotFound.into()),
        StandardApiKey::Search => Err(SearchError::DefaultSearchApiKeyNotFound.into()),
    }
}

async fn get_custom_key(client: &Client, key_name: &str) -> Result<String> {
    // 获取所有的 Key
    let keys = client.get_keys().await?.results;

    // 查找具有指定名称的 Key
    if let Some(key) = keys.iter().find(|&k| k.name.as_deref() == Some(key_name)) {
        return Ok(key.key.clone());
    }

    // 如果找不到，则返回错误
    Err(SearchError::CustomApiKeyNotFound(key_name.to_string()).into())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::config::Config;
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
            category: crate::common::PostCategory::Article,
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
