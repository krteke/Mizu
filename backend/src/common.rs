use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
}

// 定义 Article 结构体，对应数据库中的 `articles` 表
#[derive(Debug, Deserialize, Serialize)]
pub struct Article {
    // 文章 ID
    pub id: String,
    // 文章标题
    pub title: String,
    // 文章标签
    pub tags: Vec<String>,
    // 文章分类
    pub category: String,
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
    pub category: String,
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

// 定义文章分类的枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PostCategory {
    // 使用 serde 的 rename 属性，在序列化/反序列化时使用小写字符串
    #[serde(rename = "article")]
    Article,
    #[serde(rename = "note")]
    Note,
    #[serde(rename = "think")]
    Think,
    #[serde(rename = "pictures")]
    Pictures,
    #[serde(rename = "talk")]
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

// 使用 #[cfg(test)] 宏，表示这个模块只在测试时编译
#[cfg(test)]
mod tests {
    // 导入父模块的所有内容，方便测试
    use super::*;

    // 测试 as_str 方法是否能正确地将枚举转换为字符串
    #[test]
    fn test_post_category_as_str() {
        assert_eq!(PostCategory::Article.as_str(), "article");
        assert_eq!(PostCategory::Note.as_str(), "note");
        assert_eq!(PostCategory::Think.as_str(), "think");
        assert_eq!(PostCategory::Pictures.as_str(), "pictures");
        assert_eq!(PostCategory::Talk.as_str(), "talk");
    }

    // 测试 from_str 方法是否能正确地从字符串转换为枚举
    #[test]
    fn test_post_category_from_str_valid() {
        assert_eq!(
            PostCategory::from_str("article"),
            Some(PostCategory::Article)
        );
        assert_eq!(PostCategory::from_str("note"), Some(PostCategory::Note));
        assert_eq!(PostCategory::from_str("think"), Some(PostCategory::Think));
        assert_eq!(
            PostCategory::from_str("pictures"),
            Some(PostCategory::Pictures)
        );
        assert_eq!(PostCategory::from_str("talk"), Some(PostCategory::Talk));
    }

    // 测试 from_str 方法在输入无效字符串时是否返回 None
    #[test]
    fn test_post_category_from_str_invalid() {
        assert_eq!(PostCategory::from_str("nonexistent"), None);
        assert_eq!(PostCategory::from_str(""), None);
        assert_eq!(PostCategory::from_str("ARTICLE"), None); // 测试大小写敏感
    }
}
