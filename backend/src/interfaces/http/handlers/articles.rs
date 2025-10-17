use axum::{
    Json,
    extract::{Path, Query, State},
};
use std::sync::Arc;

use crate::{
    app_state::AppState,
    domain::articles::{Article, PostCategory, PostParams},
    errors::{GetPostsError, Result},
    interfaces::http::dtos::PostResponse,
};

// 最大每页数量限制
const MAX_PAGE_SIZE: i64 = 100;

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
    // 确保页码至少为 1
    let page = params.page.max(1);
    // 限制每页数量，不超过最大值
    let page_size = params.page_size.min(MAX_PAGE_SIZE).max(1);
    // 计算 SQL OFFSET
    let offset = (page - 1) * page_size;

    let category = params.category.as_str();

    let query_results = state
        .article_service
        .get_posts_by_category(category, page_size, offset)
        .await?;

    // 如果查询成功，将结果 `query_results` 包装在 `Json` 中，然后包装在 `Ok` 中返回
    Ok(Json(query_results))
}

pub async fn get_post_digital(
    Path((_category, id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Article>> {
    let result = state.article_service.get_article_by_id(&id).await?;

    Ok(Json(result))
}

// test
mod test {}
