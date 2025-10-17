use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD};

use crate::{errors::Result, infrastructure::github::client::GithubClient};

/// GitHub API client implementation using the Octocrab library
///
/// This struct provides a concrete implementation of the `GithubClient` trait
/// using the `octocrab` crate, which is a type-safe GitHub API client for Rust.
/// It handles authentication, request building, and response parsing.
///
/// # Features
///
/// - Personal access token authentication
/// - Automatic base64 decoding of file content
/// - Type-safe API interactions via octocrab
/// - Async/await support for non-blocking operations
///
/// # Authentication
///
/// This client uses GitHub personal access tokens for authentication.
/// The token should have appropriate permissions:
/// - `repo` scope for private repositories
/// - Public repositories work with any token or no authentication
///
/// # Example
///
/// ```rust
/// use backend::infrastructure::github::api_client::GithubApiClient;
///
/// let client = GithubApiClient::new("ghp_your_token_here")?;
/// let content = client.get_file_content("owner", "repo", "path/to/file.md").await?;
/// ```
pub struct GithubApiClient {
    /// Octocrab client instance configured with authentication
    ///
    /// This client maintains connection pooling and handles rate limiting
    /// internally. It can be safely cloned and shared across tasks.
    client: octocrab::Octocrab,
}

impl GithubApiClient {
    /// Create a new GitHub API client with personal access token authentication
    ///
    /// Initializes an octocrab client configured with the provided personal
    /// access token. The token is used for all subsequent API requests.
    ///
    /// # Arguments
    ///
    /// * `token` - GitHub personal access token (format: "ghp_...")
    ///
    /// # Returns
    ///
    /// * `Ok(GithubApiClient)` - Successfully initialized client
    /// * `Err(SomeError)` - Failed to build the octocrab client
    ///
    /// # Token Requirements
    ///
    /// The token should have the following permissions:
    /// - `repo` - Full control of private repositories (if accessing private repos)
    /// - `public_repo` - Access to public repositories (minimum requirement)
    ///
    /// # Security
    ///
    /// - Never hardcode tokens in source code
    /// - Load tokens from environment variables or secure configuration
    /// - Use GitHub App tokens for production systems when possible
    /// - Rotate tokens regularly for security
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::infrastructure::github::api_client::GithubApiClient;
    ///
    /// // Load token from environment
    /// let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
    /// let client = GithubApiClient::new(&token)?;
    /// ```
    pub fn new(token: &str) -> Result<Self> {
        // Build octocrab client with personal access token authentication
        let client = octocrab::OctocrabBuilder::new()
            .personal_token(token)
            .build()?;

        Ok(GithubApiClient { client })
    }
}

/// Implementation of the GithubClient trait for GithubApiClient
///
/// This implementation uses the GitHub Contents API to fetch file content
/// from repositories. The API returns base64-encoded content which is
/// automatically decoded into UTF-8 strings.
#[async_trait]
impl GithubClient for GithubApiClient {
    /// Fetch and decode file content from a GitHub repository
    ///
    /// This method uses the GitHub Contents API to retrieve file content.
    /// The API returns files as base64-encoded strings, which this method
    /// automatically decodes to UTF-8 text.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (username or organization)
    /// * `repo` - Repository name
    /// * `path` - Path to the file within the repository
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Decoded UTF-8 file content
    /// * `Err(SomeError)` - Error occurred during fetching or decoding
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File doesn't exist at the specified path
    /// - Repository is not accessible (private repo without proper permissions)
    /// - Network error or API timeout
    /// - Content is not base64-encoded (unexpected API response)
    /// - Content cannot be decoded as valid UTF-8
    /// - GitHub API rate limit exceeded
    ///
    /// # API Endpoint
    ///
    /// Uses: `GET /repos/{owner}/{repo}/contents/{path}`
    ///
    /// # Example
    ///
    /// ```rust
    /// let content = client.get_file_content(
    ///     "octocat",
    ///     "Hello-World",
    ///     "README.md"
    /// ).await?;
    /// ```
    ///
    /// # Implementation Details
    ///
    /// 1. Sends GET request to GitHub Contents API
    /// 2. Extracts the first item from the response (handles directory listings)
    /// 3. Retrieves base64-encoded content from the response
    /// 4. Decodes base64 to bytes
    /// 5. Converts bytes to UTF-8 string
    /// 6. Returns the decoded content or an error
    async fn get_file_content(&self, owner: &str, repo: &str, path: &str) -> Result<String> {
        // Fetch file content from GitHub API
        // This returns a response that may contain multiple items if path is a directory
        let response = self
            .client
            .repos(owner, repo)
            .get_content()
            .path(path)
            .send()
            .await?;

        // Extract the first item (for files, there's typically only one)
        if let Some(item) = response.items.first() {
            // Check if the item contains content (files have content, directories don't)
            if let Some(content) = &item.content {
                // Decode base64 content to bytes
                // GitHub API returns file content as base64-encoded string
                let decoded_content = BASE64_STANDARD.decode(content)?;

                // Convert bytes to UTF-8 string
                // This will fail if the file contains invalid UTF-8 (e.g., binary files)
                let content = String::from_utf8(decoded_content)?;

                return Ok(content);
            }
        }

        // No content found - either file doesn't exist or path points to a directory
        Err(anyhow::anyhow!("No content found for file: {}", path).into())
    }
}
