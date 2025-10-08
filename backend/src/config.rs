use crate::some_errors::{DBError, Result, SearchError, SomeError};

// 定义一个 Config 结构体，用于存储应用的配置信息
// #[derive(Debug, Clone)] 允许该结构体被打印调试和克隆
#[derive(Debug, Clone)]
pub struct Config {
    // 数据库连接 URL
    pub database_url: String,
    // Meilisearch 服务的 URL
    pub meilisearch_url: String,
    // Meilisearch 的主密钥，用于认证
    pub meilisearch_master_key: String,
    // 用于生成和验证 JSON Web Tokens (JWT) 的密钥
    pub jwt_secret: String,
    // 服务器绑定的主机地址，是可选的
    pub host: Option<String>,
    // 服务器监听的端口号，是可选的
    pub port: Option<u16>,
}

// 为 Config 结构体实现方法
impl Config {
    /// 从环境变量中加载配置信息
    ///
    /// 这个函数会读取一系列预定义的环境变量来构建一个 `Config` 实例。
    /// 对于必须的变量，如果它们未被设置或为空，函数会返回一个错误。
    ///
    /// # 返回
    ///
    /// - `Ok(Self)`: 如果所有必须的环境变量都被成功加载，则返回一个 `Config` 实例。
    /// - `Err(SomeError)`: 如果任何一个必须的环境变量缺失或为空，则返回一个错误。
    pub fn from_env() -> Result<Self> {
        // 定义一个内部辅助函数，用于获取并验证非空的环境变量
        fn get_non_empty_var(key: &str) -> Result<String> {
            // 尝试从环境中获取指定 key 的变量
            let value = std::env::var(key).map_err(|_| match key {
                // 如果获取失败，根据 key 的名称匹配并返回一个具体的错误类型
                "DATABASE_URL" => SomeError::from(DBError::DatabaseUrlMissing),
                "MEILISEARCH_URL" => SomeError::from(SearchError::MeilisearchUrlMissing),
                "MEILI_MASTER_KEY" => SomeError::from(SearchError::MasterKeyMissing),
                "JWT_SECRET" => SomeError::from(anyhow::Error::msg(format!("{} not set", key))),
                // 对于未知的 key，返回一个通用错误
                _ => SomeError::from(anyhow::Error::msg(format!("Unknown env var: {}", key))),
            })?;

            // 检查获取到的值去除首尾空格后是否为空
            if value.trim().is_empty() {
                // 如果为空，返回一个错误
                Err(anyhow::Error::msg(format!("{} is empty", key)).into())
            } else {
                // 如果不为空，返回获取到的值
                Ok(value)
            }
        }

        // 尝试获取 "HOST" 环境变量，.ok() 会将 Result 转换为 Option，忽略错误
        let host = std::env::var("HOST").ok();
        // 尝试获取 "PORT" 环境变量，并尝试将其解析为 u16 类型
        let port = std::env::var("PORT").ok().and_then(|p| p.parse().ok());

        // 创建并返回一个 Config 实例
        Ok(Self {
            // 使用辅助函数获取必须的环境变量
            database_url: get_non_empty_var("DATABASE_URL")?,
            meilisearch_url: get_non_empty_var("MEILISEARCH_URL")?,
            meilisearch_master_key: get_non_empty_var("MEILI_MASTER_KEY")?,
            jwt_secret: get_non_empty_var("JWT_SECRET")?,
            // 设置可选的 host 和 port
            host,
            port,
        })
    }
}
