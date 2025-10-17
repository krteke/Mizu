use async_trait::async_trait;

use crate::errors::Result;

/// GitHub client trait for fetching repository content
///
/// This trait defines the interface for interacting with the GitHub API,
/// abstracting the underlying HTTP client implementation. It follows the
/// Dependency Inversion Principle, allowing different implementations
/// (e.g., octocrab, reqwest, or mock clients for testing).
///
/// # Design Pattern
///
/// This trait is defined in the infrastructure layer but allows for multiple
/// implementations. The trait-based approach enables:
/// - Easy testing with mock implementations
/// - Swapping between different GitHub API clients
/// - Abstraction of authentication and rate limiting details
///
/// # Thread Safety
///
/// The trait requires `Send + Sync` to ensure implementations can be safely
/// shared across async tasks and thread boundaries, which is essential for
/// concurrent webhook processing.
///
/// # Authentication
///
/// Implementations should handle GitHub authentication (personal access tokens,
/// GitHub App credentials, etc.) internally, keeping authentication details
/// encapsulated.
///
/// # Rate Limiting
///
/// Implementations should be aware of GitHub's API rate limits:
/// - 5,000 requests per hour for authenticated requests
/// - 60 requests per hour for unauthenticated requests
///
/// Consider implementing retry logic with exponential backoff for rate limit errors.
///
/// # Example Implementation
///
/// ```rust
/// use async_trait::async_trait;
/// use backend::infrastructure::github::client::GithubClient;
/// use backend::errors::Result;
///
/// struct MyGithubClient {
///     token: String,
/// }
///
/// #[async_trait]
/// impl GithubClient for MyGithubClient {
///     async fn get_file_content(
///         &self,
///         owner: &str,
///         repo: &str,
///         path: &str
///     ) -> Result<String> {
///         // Implementation using octocrab, reqwest, etc.
///         Ok("file content".to_string())
///     }
/// }
/// ```
#[async_trait]
pub trait GithubClient: Send + Sync {
    /// Fetch the content of a file from a GitHub repository
    ///
    /// This method retrieves the raw content of a file from a specified GitHub
    /// repository. It's primarily used when processing webhook events to fetch
    /// the latest version of markdown files containing article content.
    ///
    /// # Arguments
    ///
    /// * `owner` - The username or organization that owns the repository
    /// * `repo` - The repository name (without the owner prefix)
    /// * `path` - The file path within the repository (e.g., "posts/my-article.md")
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The decoded file content as a UTF-8 string
    /// * `Err(SomeError)` - Error if:
    ///   - File doesn't exist in the repository
    ///   - Repository is not accessible (private without proper auth)
    ///   - Network error or API timeout
    ///   - Content cannot be decoded as UTF-8
    ///   - GitHub API rate limit exceeded
    ///
    /// # GitHub API Details
    ///
    /// This method typically uses the GitHub Contents API:
    /// ```text
    /// GET /repos/{owner}/{repo}/contents/{path}
    /// ```
    ///
    /// The API returns base64-encoded content which implementations should
    /// automatically decode before returning.
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = GithubApiClient::new("ghp_token")?;
    ///
    /// // Fetch a markdown file from the repository
    /// let content = client.get_file_content(
    ///     "octocat",
    ///     "Hello-World",
    ///     "posts/introduction.md"
    /// ).await?;
    ///
    /// println!("File content: {}", content);
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// - Files are fetched from GitHub's API, not cached locally
    /// - Large files may take longer to fetch and decode
    /// - Consider implementing caching for frequently accessed files
    /// - Be mindful of API rate limits when processing multiple files
    ///
    /// # Error Handling
    ///
    /// Implementations should convert GitHub-specific errors into the
    /// application's error types, providing clear error messages for
    /// common scenarios like missing files or permission issues.
    async fn get_file_content(&self, owner: &str, repo: &str, path: &str) -> Result<String>;
}
