use crate::some_errors::{DBError, Result, SearchError, SomeError};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub meilisearch_url: String,
    pub meilisearch_master_key: String,
    pub jwt_secret: String,
    pub host: Option<String>,
    pub port: Option<u16>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        fn get_non_empty_var(key: &str) -> Result<String> {
            let value = std::env::var(key).map_err(|_| match key {
                "DATABASE_URL" => SomeError::from(DBError::DatabaseUrlMissing),
                "MEILISEARCH_URL" => SomeError::from(SearchError::MeilisearchUrlMissing),
                "MEILI_MASTER_KEY" => SomeError::from(SearchError::MasterKeyMissing),
                "JWT_SECRET" => SomeError::from(anyhow::Error::msg(format!("{} not set", key))),
                _ => SomeError::from(anyhow::Error::msg(format!("Unknown env var: {}", key))),
            })?;

            if value.trim().is_empty() {
                Err(anyhow::Error::msg(format!("{} is empty", key)).into())
            } else {
                Ok(value)
            }
        }

        let host = std::env::var("HOST").ok();
        let port = std::env::var("PORT").ok().and_then(|p| p.parse().ok());

        Ok(Self {
            database_url: get_non_empty_var("DATABASE_URL")?,
            meilisearch_url: get_non_empty_var("MEILISEARCH_URL")?,
            meilisearch_master_key: get_non_empty_var("MEILI_MASTER_KEY")?,
            jwt_secret: get_non_empty_var("JWT_SECRET")?,
            host,
            port,
        })
    }
}
