use std::sync::Arc;

#[cfg(feature = "webhook")]
use gray_matter::{Matter, ParsedEntity, engine::YAML};
#[cfg(feature = "webhook")]
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};

#[cfg(feature = "webhook")]
use crate::domain::articles::ArticleFrontMatter;
#[cfg(feature = "webhook")]
use crate::infrastructure::github::{client::GithubClient, webhook::WebhookHandler};

use crate::{
    config::AppConfig,
    domain::{
        articles::Article,
        repositories::ArticleRepository,
        search::{SearchHit, SearchService},
    },
    errors::Result,
    interfaces::http::dtos::PostResponse,
};

pub struct ArticleService {
    db_repo: Arc<dyn ArticleRepository>,
    #[cfg(feature = "webhook")]
    github_client: Arc<dyn GithubClient>,
    search_service: Arc<dyn SearchService>,
    config: Arc<AppConfig>,
}

impl ArticleService {
    pub fn new(
        db_repo: Arc<dyn ArticleRepository>,
        #[cfg(feature = "webhook")] github_client: Arc<dyn GithubClient>,
        search_service: Arc<dyn SearchService>,
        config: Arc<AppConfig>,
    ) -> Self {
        Self {
            db_repo,
            #[cfg(feature = "webhook")]
            github_client,
            search_service,
            config,
        }
    }

    #[cfg(feature = "webhook")]
    pub async fn process_github_webhook_event(&self, event: &WebhookEvent) -> Result<()> {
        let repo_name = event.get_repository_name()?;

        if !self
            .config
            .allowed_repositories
            .read()
            .await
            .contains(&repo_name)
        {
            tracing::warn!("Repository {} is not allowed", repo_name);
            return Ok(());
        }

        tracing::info!("Processing webhook event for repository: {}", repo_name);

        match &event.kind {
            WebhookEventType::Push => {
                self.process_push_event(event).await?;
            }

            _ => {
                tracing::warn!("Unsupported webhook event type: {:?}", event.kind);
            }
        }

        Ok(())
    }

    #[cfg(feature = "webhook")]
    async fn process_push_event(&self, event: &WebhookEvent) -> Result<()> {
        let repo_name = event.get_repository_name()?;
        let owner = event.get_repository_owner()?;

        tracing::info!("Processing push event for repository: {}", repo_name);

        let changed_files = event.get_push_file_changes();

        for file_change in changed_files {
            if !self.is_valid_file(&file_change.file_path) {
                continue;
            }

            match file_change.status.as_str() {
                "added" => {
                    self.process_added_file(&owner, &repo_name, &file_change.file_path)
                        .await?;

                    tracing::info!("File {} added", file_change.file_path);
                }

                "modified" => {
                    self.process_modified_file(&owner, &repo_name, &file_change.file_path)
                        .await?;

                    tracing::info!("File {} modified", file_change.file_path);
                }

                "removed" => {
                    self.process_removed_file(&file_change.file_path).await?;

                    tracing::info!("File {} removed", file_change.file_path);
                }

                _ => {
                    tracing::warn!(
                        "Unknown file change status {} for file {}",
                        file_change.status,
                        file_change.file_path
                    );
                }
            }
        }

        Ok(())
    }

    // Check if a file is valid
    #[cfg(feature = "webhook")]
    fn is_valid_file(&self, file_path: &str) -> bool {
        use std::path::Path;

        let allowed_extensions = ["md", "mdx"];

        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| allowed_extensions.contains(&ext))
            .unwrap_or(false)
    }

    #[cfg(feature = "webhook")]
    pub async fn process_added_file(&self, owner: &str, repo: &str, file_path: &str) -> Result<()> {
        let client = &self.github_client;
        let content = client.get_file_content(owner, repo, file_path).await?;

        let matter = Matter::<YAML>::new();
        let result: ParsedEntity = matter.parse(&content)?;

        if let Some(data) = result.data {
            let front_matter: ArticleFrontMatter = data.deserialize()?;
        }

        todo!()
    }

    #[cfg(feature = "webhook")]
    pub async fn process_modified_file(
        &self,
        owner: &str,
        repo: &str,
        file_path: &str,
    ) -> Result<()> {
        // 修改文件的处理逻辑与添加相同：获取最新内容并更新
        let client = &self.github_client;
        let content = client.get_file_content(owner, repo, file_path).await?;

        let matter = Matter::<YAML>::new();
        let _result: ParsedEntity = matter.parse(&content)?;

        // TODO: 提取 frontmatter 并更新数据库
        // TODO: 更新搜索索引

        todo!()
    }

    #[cfg(feature = "webhook")]
    pub async fn process_removed_file(&self, file_path: &str) -> Result<()> {
        // TODO: 从文件路径提取文章 ID
        // TODO: 从数据库删除文章
        // TODO: 从搜索索引删除

        tracing::info!("Would remove file: {}", file_path);
        Ok(())
    }

    // if id has already been used, return false, otherwise return true
    pub async fn is_valid_id(&self, id: &str) -> bool {
        let result = match self.db_repo.find_optional_by_id(id).await {
            Ok(item) => {
                if let Some(_) = item {
                    false
                } else {
                    true
                }
            }
            Err(_) => true,
        };

        result
    }

    pub async fn get_posts_by_category(
        &self,
        category: &str,
        page_size: i64,
        offset: i64,
    ) -> Result<Vec<PostResponse>> {
        let result = self
            .db_repo
            .get_posts_by_category(category, page_size, offset)
            .await;

        result
    }

    pub async fn get_article_by_id(&self, id: &str) -> Result<Article> {
        let result = self.db_repo.get_post_by_id(id).await;

        result
    }

    pub async fn search(
        &self,
        index: &str,
        query: &str,
        current_page: usize,
        limit: usize,
    ) -> Result<(Vec<SearchHit>, usize, usize, usize)> {
        let result = self
            .search_service
            .search(query, index, current_page, limit)
            .await;

        result
    }

    pub async fn create_index(&self, index: &str, searchable_attributes: &[&str]) -> Result<()> {
        let client = &self
            .search_service
            .create_index_client(index, searchable_attributes)
            .await?;

        let db_articles = self.db_repo.get_all().await?;

        client
            .index(index)
            .add_documents(&db_articles, Some("id"))
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(())
    }
}
