use async_trait::async_trait;

use crate::{
    domain::{articles::Article, repositories::ArticleRepository},
    errors::{GetPostsError, Result},
    interfaces::http::dtos::PostResponse,
};

pub struct SqlxArticleRepository {
    pool: sqlx::PgPool,
}

impl SqlxArticleRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ArticleRepository for SqlxArticleRepository {
    async fn save(&self, article: &crate::domain::articles::Article) -> crate::errors::Result<()> {
        todo!()
    }

    async fn delete_by_path(&self, path: &str) -> crate::errors::Result<()> {
        todo!()
    }

    async fn find_optional_by_id(&self, id: &str) -> Result<Option<Article>> {
        let article: Option<Article> = sqlx::query_as("SELECT * FROM articles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(article)
    }

    async fn get_posts_by_category(
        &self,
        category: &str,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<PostResponse>> {
        let query_results = sqlx::query_as!(
            PostResponse,
            // SQL 查询语句：从 `articles` 表中选择需要的列，并添加分页
            "SELECT id, title, tags, summary
             FROM articles
             WHERE category = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
            category,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(query_results)
    }

    async fn get_post_by_id(&self, id: &str) -> Result<Article> {
        let result = sqlx::query_as::<_, Article>("SELECT * FROM articles WHERE id = $1")
            .bind(&id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| GetPostsError::ArticleNotFound)?;

        Ok(result)
    }

    async fn get_all(&self) -> Result<Vec<Article>> {
        let db_items = sqlx::query_as::<_, Article>("SELECT * FROM articles")
            .fetch_all(&self.pool)
            .await?;

        Ok(db_items)
    }
}
