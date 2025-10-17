use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::str::FromStr;
use time::OffsetDateTime;

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

#[cfg(feature = "webhook")]
#[derive(Debug, Deserialize)]
pub struct ArticleFrontMatter {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub category: PostCategory,
    pub summary: Option<String>,
    pub status: String,
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
}

// 实现 FromStr trait，符合 Rust 标准惯例
impl FromStr for PostCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "article" => Ok(PostCategory::Article),
            "note" => Ok(PostCategory::Note),
            "pictures" => Ok(PostCategory::Pictures),
            "talk" => Ok(PostCategory::Talk),
            "think" => Ok(PostCategory::Think),
            _ => Err(format!("Invalid category: {}", s)),
        }
    }
}

// 定义一个结构体 PostParams，用于接收文章列表请求的查询参数
// #[derive(Deserialize, Clone)] 是一个派生宏：
// - Deserialize: 允许这个结构体从查询字符串等格式中自动反序列化
// - Clone: 允许创建这个结构体的副本
#[derive(Deserialize, Clone)]
pub struct PostParams {
    // 文章分类
    pub category: PostCategory,
    // 页码（从 1 开始）
    #[serde(default = "default_page")]
    pub page: i64,
    // 每页数量
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

// 默认页码为 1
fn default_page() -> i64 {
    1
}

// 默认每页 20 条
fn default_page_size() -> i64 {
    20
}
