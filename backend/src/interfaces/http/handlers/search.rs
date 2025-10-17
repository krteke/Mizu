use axum::{
    Json,
    extract::{Query, State},
};
use std::sync::Arc;

use crate::{
    app_state::AppState,
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

    let (results, total_hits, total_pages, current_page) = state
        .article_service
        .search(DEFAULT_SEARCH_INDEX, &params.q, params.page, PAGE_ITEMS)
        .await?;

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
    let searchable_attributes = ["title", "content", "summary"];

    state
        .article_service
        .create_index(DEFAULT_SEARCH_INDEX, searchable_attributes.as_slice())
        .await
}
