use serde::{Deserialize, Serialize};

use crate::domain::search::SearchHit;

// 定义获取文章列表接口的响应结构体
#[derive(Serialize, Deserialize)]
pub struct PostResponse {
    pub id: String,
    // 文章标题
    pub title: String,
    // 文章标签
    pub tags: Vec<String>,
    // 文章内容
    pub content: String,
}

// 定义搜索响应的结构体，将被序列化为 JSON
#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    pub total_hits: usize,       // 匹配的总命中数
    pub total_pages: usize,      // 总页数
    pub current_page: usize,     // 当前页码
    pub results: Vec<SearchHit>, // 搜索结果列表
}

// 定义搜索请求的查询参数结构体
#[derive(Deserialize, Debug)]
pub struct SearchParams {
    pub q: String,   // 搜索查询字符串
    pub page: usize, // 请求的页码
}
