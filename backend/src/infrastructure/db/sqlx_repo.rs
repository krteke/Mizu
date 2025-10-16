use async_trait::async_trait;

use crate::{
    domain::{articles::Article, repositories::ArticleRepository},
    errors::Result,
};

pub struct SqlxArticleRepository {
    pool: sqlx::PgPool,
}

#[async_trait]
impl ArticleRepository for SqlxArticleRepository {
    async fn save(&self, article: &crate::domain::articles::Article) -> crate::errors::Result<()> {
        todo!()
    }

    async fn delete_by_path(&self, path: &str) -> crate::errors::Result<()> {
        todo!()
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Article>> {
        let article: Option<Article> = sqlx::query_as("SELECT * FROM articles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(article)
    }
}
