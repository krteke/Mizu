use axum::{
    Json,
    extract::{Query, State},
};
use meilisearch_sdk::{client::Client, key::Key, search::Selectors};
use serde::{Deserialize, Serialize};

use crate::{
    common::{AppState, Article, SearchHit},
    config::Config,
    some_errors::{Result, SearchError},
};

const PAGE_ITEMS: usize = 6;
const DEFAULT_SEARCH_INDEX: &str = "articles";

pub enum ClientType {
    Search,
    Admin,
}

#[derive(Clone)]
pub struct SearchService {
    pub admin_client: Client,
    pub search_client: Client,
    pub index_name: String,
}

impl SearchService {
    pub async fn new(config: &Config, index_name: String) -> Result<Self> {
        let meili_search_url = &config.meilisearch_url;

        let master_client = Self::create_master_client(config)?;
        master_client.health().await?;

        let admin_key = get_key(&master_client, ApiKeyType::Admin).await?;
        let search_key = get_key(&master_client, ApiKeyType::Search).await?;

        let admin_client = Client::new(meili_search_url, Some(admin_key))?;
        let search_client = Client::new(meili_search_url, Some(search_key))?;

        Ok(Self {
            admin_client,
            search_client,
            index_name,
        })
    }

    pub async fn update_or_add_index_item(&self, article: &Article) -> Result<()> {
        let client = &self.admin_client;
        let index = client.index(&self.index_name);

        index
            .add_or_replace(&[article], Some("id"))
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(())
    }

    pub async fn delete_index_item(&self, article: &Article) -> Result<()> {
        let client = &self.admin_client;
        let index = client.index(&self.index_name);

        index
            .delete_document(&article.id)
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(())
    }

    fn create_master_client(config: &Config) -> Result<Client> {
        let meili_search_url = &config.meilisearch_url;
        let key = &config.meilisearch_master_key;

        Ok(Client::new(meili_search_url, Some(key))?)
    }
}

#[derive(Debug)]
enum ApiKeyType {
    Search,
    Admin,
    Other(String),
}

#[derive(Deserialize, Debug)]
pub struct SearchParams {
    q: String,
    page: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    pub total_hits: usize,
    pub total_pages: usize,
    pub current_page: usize,
    pub results: Vec<SearchHit>,
}

pub async fn get_search_results(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>> {
    if params.q.trim().is_empty() {
        return Ok(Json(SearchResponse {
            total_hits: 0,
            total_pages: 0,
            current_page: params.page,
            results: vec![],
        }));
    }

    let current_page = params.page.max(1);
    let offset = PAGE_ITEMS * (current_page - 1);

    let index = &state
        .search_service
        .search_client
        .index(DEFAULT_SEARCH_INDEX);

    let search_result = index
        .search()
        .with_query(&params.q)
        .with_offset(offset)
        .with_limit(PAGE_ITEMS)
        .with_attributes_to_highlight(Selectors::Some(&["title", "summary", "content"]))
        .with_highlight_pre_tag("<span class=\"highlight\">")
        .with_highlight_post_tag("</span>")
        .with_attributes_to_crop(Selectors::Some(&[("summary", None), ("content", None)]))
        .execute::<Article>()
        .await?;

    let total_hits = search_result.total_hits.unwrap_or(0);
    let total_pages = (total_hits + PAGE_ITEMS - 1) / PAGE_ITEMS;

    let results: Vec<SearchHit> = search_result
        .hits
        .into_iter()
        .map(|r| {
            let mut hit_result = SearchHit {
                id: r.result.id.clone(),
                category: r.result.category.clone(),
                title: r.result.title.clone(),
                summary: "".to_string(),
                content: "".to_string(),
            };

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

    Ok(Json(SearchResponse {
        total_hits,
        total_pages,
        current_page,
        results,
    }))
}

pub async fn create_search_index(State(state): State<AppState>) -> Result<()> {
    let client = &state.search_service.admin_client;
    client
        .create_index(DEFAULT_SEARCH_INDEX, Some("id"))
        .await?
        .wait_for_completion(&client, None, None)
        .await?;

    let searchable_attributes = ["title", "tags", "content"];
    client
        .index(DEFAULT_SEARCH_INDEX)
        .set_filterable_attributes(&searchable_attributes)
        .await?;

    let db_pool = state.db_pool;

    let db_articles = sqlx::query_as!(Article, "SELECT * FROM articles")
        .fetch_all(&db_pool)
        .await?;

    client
        .index(DEFAULT_SEARCH_INDEX)
        .add_documents(&db_articles, Some("id"))
        .await?
        .wait_for_completion(&client, None, None)
        .await?;

    Ok(())
}

async fn get_api_keys(config: &Config) -> Result<Vec<Key>> {
    let client = SearchService::create_master_client(config)?;

    Ok(client.get_keys().await?.results)
}

async fn get_key(client: &Client, key_type: ApiKeyType) -> Result<String> {
    let key_name = match &key_type {
        ApiKeyType::Admin => "Default Admin API Key",
        ApiKeyType::Search => "Default Search API Key",
        ApiKeyType::Other(t) => t,
    };

    let keys = client.get_keys().await?.results;

    if let Some(key) = keys.iter().find(|k| k.name.as_deref() == Some(key_name)) {
        return Ok(key.key.clone());
    }

    match key_type {
        ApiKeyType::Admin => Err(SearchError::DefaultAdminApiKeyNotFound.into()),
        ApiKeyType::Search => Err(SearchError::DefaultSearchApiKeyNotFound.into()),
        ApiKeyType::Other(name) => Err(SearchError::KeyNameNotFound(name).into()),
    }
}

// mod test {
//     use super::*;

//     fn create_client() -> Result<Client, SomeError> {
//         SearchService::create_master_client()
//     }

//     #[tokio::test]
//     async fn test_get_api_keys() {
//         let api_keys = get_api_keys().await.unwrap();

//         println!("{:#?}", api_keys);
//     }

//     #[tokio::test]
//     async fn test_get_search_api_key() {
//         let client = create_client().unwrap();
//         let search_api_key = get_key(&client, ApiKeyType::Search).await.unwrap();

//         println!("{}", search_api_key);
//     }

//     #[tokio::test]
//     async fn test_get_admin_api_key() {
//         let client = create_client().unwrap();
//         let admin_api_key = get_key(&client, ApiKeyType::Admin).await.unwrap();

//         println!("{}", admin_api_key);
//     }

//     #[test]
//     fn test_new_master_client() {
//         let client = SearchService::create_master_client();

//         assert!(client.is_ok());
//     }
// }
