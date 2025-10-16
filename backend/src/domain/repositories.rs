use async_trait::async_trait;

use crate::{domain::articles::Article, errors::Result};

#[async_trait]
pub trait ArticleRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Article>>;

    async fn save(&self, article: &Article) -> Result<()>;

    async fn delete_by_path(&self, path: &str) -> Result<()>;
}
