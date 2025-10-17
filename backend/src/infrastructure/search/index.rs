use async_trait::async_trait;
use meilisearch_sdk::{
    client::Client,
    key::Key,
    search::{SearchResults, Selectors},
};

use crate::{
    config::Config,
    domain::{
        articles::Article,
        search::{SearchHit, SearchService},
    },
    errors::{Result, SearchError},
};

// 定义一个枚举来区分不同权限的 Meilisearch 客户端
#[derive(Debug, Clone, Copy)]
pub enum ClientType {
    Search, // 仅用于搜索
    Admin,  // 用于管理操作，如添加/删除文档
}

// 定义搜索服务结构体，封装了与 Meilisearch 交互的逻辑
#[derive(Clone)]
pub struct MeiliSearchService {
    pub admin_client: Client,  // 具有管理权限的 Meilisearch 客户端
    pub search_client: Client, // 仅具有搜索权限的 Meilisearch 客户端
    pub index_name: String,    // 索引的名称
}

impl MeiliSearchService {
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

    pub async fn get_search_result(
        &self,
        index: &str,
        page_limit: usize,
        params: &str,
        offset: usize,
    ) -> Result<SearchResults<Article>> {
        let search_index = &self.search_client.index(index);

        // 构建搜索查询
        let search_result = search_index
            .search()
            .with_query(params) // 设置查询词
            .with_offset(offset) // 设置偏移量
            .with_limit(page_limit) // 设置每页限制
            .with_attributes_to_highlight(Selectors::Some(&["title", "summary", "content"])) // 设置高亮的字段
            .with_highlight_pre_tag("<span class=\"highlight\">") // 设置高亮前缀标签
            .with_highlight_post_tag("</span>") // 设置高亮后缀标签
            .with_attributes_to_crop(Selectors::Some(&[("summary", None), ("content", None)])) // 设置要裁剪的字段
            .execute::<Article>() // 执行搜索
            .await?;

        Ok(search_result)
    }
}

// 定义一个枚举来表示不同类型的 API Key
#[derive(Debug, Clone, Copy)]
enum StandardApiKey {
    Search, // 搜索 Key
    Admin,  // 管理 Key
}

// 异步函数，用于获取所有的 API Key
async fn get_api_keys(config: &Config) -> Result<Vec<Key>> {
    let client = MeiliSearchService::create_master_client(config)?;
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

#[async_trait]
impl SearchService for MeiliSearchService {
    async fn search(
        &self,
        query: &str,
        index: &str,
        current_page: usize,
        limit: usize,
    ) -> Result<(Vec<SearchHit>, usize, usize, usize)> {
        let index = &self.search_client.index(index);

        let current_page = current_page.max(1);
        let offset = (current_page - 1) * limit;

        let search_result = index
            .search()
            .with_query(query)
            .with_offset(offset)
            .with_limit(limit)
            .with_attributes_to_highlight(Selectors::Some(&["title", "summary", "content"])) // 设置高亮的字段
            .with_highlight_pre_tag("<span class=\"highlight\">") // 设置高亮前缀标签
            .with_highlight_post_tag("</span>") // 设置高亮后缀标签
            .with_attributes_to_crop(Selectors::Some(&[("summary", None), ("content", None)])) // 设置要裁剪的字段
            .execute::<Article>() // 执行搜索
            .await?;

        // 计算总命中数和总页数
        let total_hits = search_result.total_hits.unwrap_or(0);
        let total_pages = (total_hits + limit - 1) / limit;

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

        Ok((results, total_hits, total_pages, current_page))
    }

    async fn create_index_client(
        &self,
        index: &str,
        searchable_attributes: &[&str],
    ) -> Result<&Client> {
        let client = &self.admin_client;

        client
            .create_index(index, Some("id"))
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        client
            .index(index)
            .set_filterable_attributes(searchable_attributes)
            .await?;

        Ok(client)
    }
}
