#[cfg(feature = "webhook")]
use std::collections::HashSet;

use config::{Config as ConfigLoader, Environment, File};
use serde::Deserialize;
#[cfg(feature = "webhook")]
use tokio::sync::RwLock;

use crate::errors::Result;

// 定义一个 Config 结构体，用于存储应用的配置信息
// #[derive(Debug, Clone)] 允许该结构体被打印调试和克隆
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // 数据库连接 URL
    pub database_url: String,
    // Meilisearch 服务的 URL
    pub meilisearch_url: String,
    // Meilisearch 的主密钥，用于认证
    pub meili_master_key: String,
    // 用于生成和验证 JSON Web Tokens (JWT) 的密钥
    pub jwt_secret: String,
    // 服务器绑定的主机地址，是可选的
    pub host: Option<String>,
    // 服务器监听的端口号，是可选的
    pub port: Option<u16>,
    #[cfg(feature = "webhook")]
    pub github_webhook_secret: String,
    #[cfg(feature = "webhook")]
    #[serde(default)]
    pub allowed_repositories: HashSet<String>,
    #[cfg(feature = "webhook")]
    pub github_token: String,
}

// 为 Config 结构体实现方法
impl Config {
    pub fn new() -> Result<Self> {
        let builder = ConfigLoader::builder()
            .add_source(File::with_name("config.toml").required(false))
            .add_source(Environment::default());

        let settings = builder.build()?;
        let config = settings.try_deserialize()?;
        Ok(config)
    }
}

pub struct AppConfig {
    pub jwt_secret: String,
    #[cfg(feature = "webhook")]
    pub github_webhook_secret: String,
    #[cfg(feature = "webhook")]
    pub github_token: String,
    #[cfg(feature = "webhook")]
    pub allowed_repositories: RwLock<HashSet<String>>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            jwt_secret: String::new(),
            github_webhook_secret: String::new(),
            github_token: String::new(),
            allowed_repositories: RwLock::new(HashSet::new()),
        }
    }

    pub fn jwt_secret(mut self, secret: &str) -> Self {
        self.jwt_secret = secret.to_string();
        self
    }

    pub fn github_webhook_secret(mut self, secret: &str) -> Self {
        self.github_webhook_secret = secret.to_string();
        self
    }

    pub fn github_token(mut self, token: &str) -> Self {
        self.github_token = token.to_string();
        self
    }

    pub async fn allowed_repositories(self, repositories: HashSet<String>) -> Self {
        *self.allowed_repositories.write().await = repositories;
        self
    }

    pub async fn build(self) -> Result<Self> {
        if !self.jwt_secret.trim().is_empty()
            && !self.github_token.trim().is_empty()
            && !self.github_webhook_secret.trim().is_empty()
        {
            Ok(self)
        } else {
            Err(anyhow::anyhow!("Missing required configuration").into())
        }
    }
}
