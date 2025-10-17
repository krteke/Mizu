use async_trait::async_trait;
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};

use crate::{domain::articles::PostCategory, errors::Result};

// 定义搜索结果的结构体
#[derive(Serialize, Deserialize)]
pub struct SearchHit {
    // 文章 ID
    pub id: String,
    // 文章标题
    pub title: String,
    // 文章分类
    pub category: PostCategory,
    // 文章摘要 (可能包含高亮)
    pub summary: String,
    // 文章内容 (可能包含高亮)
    pub content: String,
}

#[async_trait]
pub trait SearchService: Send + Sync {
    async fn search(
        &self,
        query: &str,
        index: &str,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<SearchHit>, usize, usize, usize)>;

    async fn create_index_client(
        &self,
        index: &str,
        searchable_attributes: &[&str],
    ) -> Result<&Client>;
}
