use std::sync::Arc;

use gray_matter::{Matter, ParsedEntity, engine::YAML};

use crate::{
    domain::{
        articles::{Article, ArticleFrontMatter},
        repositories::ArticleRepository,
    },
    errors::Result,
    infrastructure::github::client::GithubClient,
};

pub struct ArticleService {
    db_repo: Arc<dyn ArticleRepository>,
    github_client: Arc<dyn GithubClient>,
}

impl ArticleService {
    pub fn new(db_repo: Arc<dyn ArticleRepository>, github_client: Arc<dyn GithubClient>) -> Self {
        Self {
            db_repo,
            github_client,
        }
    }

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

    pub async fn process_removed_file(&self, file_path: &str) -> Result<()> {
        // TODO: 从文件路径提取文章 ID
        // TODO: 从数据库删除文章
        // TODO: 从搜索索引删除

        tracing::info!("Would remove file: {}", file_path);
        Ok(())
    }

    // if id has already been used, return false, otherwise return true
    pub async fn is_valid_id(&self, id: &str) -> bool {
        let result = match self.db_repo.find_by_id(id).await {
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
}
