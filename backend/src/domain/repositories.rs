use async_trait::async_trait;

use crate::{domain::articles::Article, errors::Result, interfaces::http::dtos::PostResponse};

#[async_trait]
pub trait ArticleRepository: Send + Sync {
    async fn find_optional_by_id(&self, id: &str) -> Result<Option<Article>>;

    async fn save(&self, article: &Article) -> Result<()>;

    async fn delete_by_path(&self, path: &str) -> Result<()>;

    async fn get_posts_by_category(
        &self,
        category: &str,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<PostResponse>>;

    async fn get_post_by_id(&self, id: &str) -> Result<Article>;

    async fn get_all(&self) -> Result<Vec<Article>>;
}
