use async_trait::async_trait;

use crate::errors::Result;

#[async_trait]
pub trait GithubClient {
    async fn get_file_content(&self, owner: &str, repo: &str, path: &str) -> Result<String>;
}
