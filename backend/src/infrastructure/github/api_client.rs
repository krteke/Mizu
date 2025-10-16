use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD};

use crate::{errors::Result, infrastructure::github::client::GithubClient};

pub struct GithubApiClient {
    client: octocrab::Octocrab,
}

impl GithubApiClient {
    pub fn new(token: &str) -> Result<Self> {
        let client = octocrab::OctocrabBuilder::new()
            .personal_token(token)
            .build()?;

        Ok(GithubApiClient { client })
    }
}

#[async_trait]
impl GithubClient for GithubApiClient {
    async fn get_file_content(&self, owner: &str, repo: &str, path: &str) -> Result<String> {
        let response = self
            .client
            .repos(owner, repo)
            .get_content()
            .path(path)
            .send()
            .await?;

        if let Some(item) = response.items.first() {
            if let Some(content) = &item.content {
                let decoded_content = BASE64_STANDARD.decode(content)?;
                let content = String::from_utf8(decoded_content)?;

                return Ok(content);
            }
        }

        Err(anyhow::anyhow!("No content found for file: {}", path).into())
    }
}
