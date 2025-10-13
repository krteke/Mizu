#[cfg(feature = "webhook")]
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use time::OffsetDateTime;

use crate::search::SearchService;

// 定义应用的状态结构体 AppState，它将在整个应用中共享
#[derive(Clone)]
pub struct AppState {
    // 数据库连接池
    pub db_pool: PgPool,
    // 用于 JWT (JSON Web Token) 的密钥
    pub jwt_secret: String,
    // 搜索服务
    pub search_service: SearchService,
    #[cfg(feature = "webhook")]
    pub github_webhook_secret: String,
    #[cfg(feature = "webhook")]
    pub allowed_repositories: HashSet<String>,
}

// 定义 Article 结构体，对应数据库中的 `articles` 表
#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Article {
    // 文章 ID
    pub id: String,
    // 文章标题
    pub title: String,
    // 文章标签
    pub tags: Vec<String>,
    // 文章分类
    pub category: PostCategory,
    // 文章摘要
    pub summary: String,
    // 文章内容
    pub content: String,
    // 文章状态
    pub status: String,
    // 创建时间
    pub created_at: OffsetDateTime,
    // 更新时间
    pub updated_at: OffsetDateTime,
}

// 定义搜索结果的结构体
#[derive(Serialize, Deserialize)]
pub struct SearchHit {
    // 文章 ID
    pub id: String,
    // 文章标题
    pub title: String,
    // 文章分类
    pub category: PostCategory,
    // 文章摘要 (可能包含高亮)
    pub summary: String,
    // 文章内容 (可能包含高亮)
    pub content: String,
}

// 定义获取文章列表接口的响应结构体
#[derive(Serialize, Deserialize)]
pub struct PostResponse {
    pub id: String,
    // 文章标题
    pub title: String,
    // 文章标签
    pub tags: Vec<String>,
    // 文章内容
    pub content: String,
}

#[cfg(feature = "webhook")]
#[derive(Deserialize)]
pub struct WebhookQuery {
    pub hub_mode: Option<String>,
    pub hub_challenge: Option<String>,
    pub hub_verify_token: Option<String>,
}

// 定义文章分类的枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
// 使用 serde 的 rename 属性，在序列化/反序列化时使用小写字符串
#[serde(rename_all = "lowercase")]
pub enum PostCategory {
    Article,
    Note,
    Think,
    Pictures,
    Talk,
}

// 为 PostCategory 枚举实现方法
impl PostCategory {
    // 将枚举成员转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            PostCategory::Article => "article",
            PostCategory::Note => "note",
            PostCategory::Pictures => "pictures",
            PostCategory::Talk => "talk",
            PostCategory::Think => "think",
        }
    }

    // 从字符串转换为枚举成员
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "article" => Some(PostCategory::Article),
            "note" => Some(PostCategory::Note),
            "pictures" => Some(PostCategory::Pictures),
            "talk" => Some(PostCategory::Talk),
            "think" => Some(PostCategory::Think),
            // 如果字符串不匹配任何分类，则返回 None
            _ => None,
        }
    }
}
