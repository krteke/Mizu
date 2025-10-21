use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

/// Article domain entity representing a blog post or article
///
/// This struct maps directly to the `articles` table in the database
/// and represents the core article entity in the domain model.
///
/// # Fields
///
/// * `id` - Unique identifier for the article
/// * `title` - Article title
/// * `tags` - List of tags associated with the article
/// * `category` - Category classification (article, note, think, etc.)
/// * `summary` - Brief summary or excerpt of the article
/// * `content` - Full article content (markdown format)
/// * `status` - Publication status (draft, published, archived, etc.)
/// * `created_at` - Timestamp when the article was created
/// * `updated_at` - Timestamp when the article was last modified
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Article {
    /// Unique identifier for the article
    pub id: String,

    /// Article title
    pub title: String,

    /// List of tags for categorization and search
    pub tags: Vec<String>,

    /// Article category (article, note, think, pictures, talk)
    pub category: PostCategory,

    /// Brief summary or excerpt of the article
    pub summary: Option<String>,

    /// Full article content in markdown format
    pub content: String,

    /// Publication status (e.g., "draft", "published", "archived")
    pub status: String,

    /// Timestamp when the article was created
    pub created_at: OffsetDateTime,

    /// Timestamp when the article was last updated
    pub updated_at: OffsetDateTime,

    /// Timestamp when the article was deleted
    pub deleted_at: Option<OffsetDateTime>,
}

/// Front matter structure for articles loaded from markdown files
///
/// This struct is used when processing webhook events that include
/// markdown files with YAML front matter. It represents the metadata
/// extracted from the front matter section.
///
/// Only available when the "webhook" feature is enabled.
#[cfg(feature = "webhook")]
#[derive(Debug, Clone, Deserialize)]
pub struct ArticleFrontMatter {
    /// Unique identifier from front matter
    pub id: String,

    /// Article title from front matter
    pub title: String,

    /// List of tags from front matter
    pub tags: Vec<String>,

    /// Article category from front matter
    pub category: PostCategory,

    /// Optional summary from front matter
    pub summary: Option<String>,

    /// Publication status from front matter
    pub status: String,
}

/// Enumeration of article categories
///
/// This enum represents the different types of content that can be published.
/// Each variant corresponds to a different content category with specific
/// characteristics and presentation styles.
///
/// # Variants
///
/// * `Article` - Long-form articles and tutorials
/// * `Note` - Short notes and quick thoughts
/// * `Think` - Reflective pieces and opinions
/// * `Pictures` - Photo galleries and image-focused content
/// * `Talk` - Talks, presentations, and speeches
///
/// # Serialization
///
/// The enum is serialized to lowercase strings in both JSON and database
/// representations (e.g., "article", "note", "think").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PostCategory {
    /// Long-form articles and tutorials
    Article,

    /// Short notes and quick thoughts
    Note,

    /// Reflective pieces and opinions
    Think,

    /// Photo galleries and image-focused content
    Pictures,

    /// Talks, presentations, and speeches
    Talk,
}

impl PostCategory {
    /// Convert the category enum to its string representation
    ///
    /// # Returns
    ///
    /// A static string slice representing the category in lowercase
    ///
    /// # Examples
    ///
    /// ```
    /// use backend::domain::articles::PostCategory;
    ///
    /// assert_eq!(PostCategory::Article.as_str(), "article");
    /// assert_eq!(PostCategory::Note.as_str(), "note");
    /// ```
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

/// Implement FromStr trait for parsing strings into PostCategory
///
/// This allows converting string representations (e.g., from URL parameters)
/// into the PostCategory enum. The conversion is case-sensitive and only
/// accepts lowercase strings.
impl FromStr for PostCategory {
    type Err = String;

    /// Parse a string into a PostCategory
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse (must be lowercase)
    ///
    /// # Returns
    ///
    /// * `Ok(PostCategory)` - Successfully parsed category
    /// * `Err(String)` - Error message if the string is not a valid category
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use backend::domain::articles::PostCategory;
    ///
    /// let category = PostCategory::from_str("article").unwrap();
    /// assert_eq!(category, PostCategory::Article);
    ///
    /// let invalid = PostCategory::from_str("invalid");
    /// assert!(invalid.is_err());
    /// ```
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

/// Query parameters for fetching paginated lists of articles
///
/// This struct represents the query parameters that can be provided
/// when requesting a list of articles. It includes category filtering
/// and pagination controls.
///
/// # Fields
///
/// * `category` - Filter articles by category
/// * `page` - Page number (1-based indexing, defaults to 1)
/// * `page_size` - Number of items per page (defaults to 20)
///
/// # Example Query String
///
/// ```text
/// /posts?category=article&page=2&page_size=10
/// ```
#[derive(Deserialize, Clone)]
pub struct PostParams {
    /// Filter articles by this category
    pub category: PostCategory,

    /// Page number for pagination (1-based, defaults to 1)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Number of articles per page (defaults to 20)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

/// Default page number for pagination
///
/// Returns 1 as the default starting page for pagination queries
fn default_page() -> i64 {
    1
}

/// Default page size for pagination
///
/// Returns 20 as the default number of items per page
fn default_page_size() -> i64 {
    20
}
