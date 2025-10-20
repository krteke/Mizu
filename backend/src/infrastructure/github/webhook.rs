use chrono::{DateTime, Utc};
use octocrab::models::webhook_events::{
    WebhookEvent, WebhookEventType, payload::WebhookEventPayload,
};
use serde::{Deserialize, Serialize};

use crate::errors::{Result, WebHooksError};

/// Represents a file change detected in a webhook event
///
/// This struct captures information about a single file that was added,
/// modified, or removed in a Git commit. It's used to track which files
/// need to be processed when handling push events.
///
/// # Fields
///
/// * `file_path` - Relative path to the file within the repository
/// * `status` - Change status: "added", "modified", or "removed"
/// * `row_url` - Optional URL to view the file (currently unused)
///
/// # Example
///
/// ```rust
/// use backend::infrastructure::github::webhook::FileChange;
///
/// let change = FileChange {
///     file_path: "posts/my-article.md".to_string(),
///     status: "added".to_string(),
///     row_url: None,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct FileChange {
    /// Path to the file within the repository (e.g., "posts/article.md")
    pub file_path: String,

    /// Status of the change: "added", "modified", or "removed"
    pub status: String,

    pub timestamp: DateTime<Utc>,

    /// Optional URL to view the file in the repository
    /// Note: Currently not populated, reserved for future use
    pub row_url: Option<String>,
}

/// GitHub push event payload structure
///
/// Represents the data sent by GitHub when a push event occurs.
/// This is a simplified version focusing on the fields we need
/// for article processing.
///
/// # Note
///
/// This struct is currently defined but not actively used, as we rely on
/// octocrab's built-in `WebhookEvent` parsing. It's kept for potential
/// future custom parsing needs.
#[derive(Debug, Deserialize)]
pub struct PushEvent {
    /// List of commits in this push event
    pub commits: Option<Vec<CommitInfo>>,

    /// Repository information
    pub repository: RepositoryInfo,

    /// User who triggered the push
    pub sender: OwnerInfo,
}

/// Information about a single commit in a push event
///
/// Contains details about files changed in a specific commit,
/// including which files were added, removed, or modified.
#[derive(Debug, Deserialize)]
pub struct CommitInfo {
    /// Commit SHA hash
    pub id: String,

    /// Commit message
    pub message: String,

    /// List of file paths that were added in this commit
    pub added: Vec<String>,

    /// List of file paths that were removed in this commit
    pub removed: Vec<String>,

    /// List of file paths that were modified in this commit
    pub modified: Vec<String>,
}

/// Repository information from webhook payload
///
/// Contains identifying information about the repository
/// that triggered the webhook event.
#[derive(Debug, Deserialize)]
pub struct RepositoryInfo {
    /// Short repository name (e.g., "my-repo")
    pub name: String,

    /// Full repository name including owner (e.g., "owner/my-repo")
    pub full_name: String,

    /// Repository owner information
    pub owner: OwnerInfo,
}

/// Owner/user information from webhook payload
///
/// Represents a GitHub user or organization that owns a repository
/// or triggered an event.
#[derive(Debug, Deserialize)]
pub struct OwnerInfo {
    /// GitHub username or organization name
    pub login: String,
}

/// Trait for extracting information from GitHub webhook events
///
/// This trait provides convenience methods for parsing webhook payloads
/// and extracting commonly needed information like file changes and
/// repository details.
///
/// # Implementations
///
/// Currently implemented for `WebhookEvent` from the octocrab crate,
/// providing a unified interface for webhook processing.
///
/// # Example
///
/// ```rust
/// use backend::infrastructure::github::webhook::WebhookHandler;
/// use octocrab::models::webhook_events::WebhookEvent;
///
/// fn process_webhook(event: &WebhookEvent) {
///     // Extract repository name
///     let repo_name = event.get_repository_name().unwrap();
///
///     // Get list of changed files
///     let changes = event.get_push_file_changes();
///     for change in changes {
///         println!("{}: {}", change.status, change.file_path);
///     }
/// }
/// ```
pub trait WebhookHandler {
    /// Extract file changes from a push event
    ///
    /// Parses the webhook event and returns a list of all file changes
    /// across all commits in the push. Each file is categorized as
    /// added, modified, or removed.
    ///
    /// # Returns
    ///
    /// A vector of `FileChange` objects, one for each file that was
    /// changed in the push event. Returns an empty vector for non-push
    /// events or if no files were changed.
    ///
    /// # Example
    ///
    /// ```rust
    /// let changes = event.get_push_file_changes();
    /// for change in changes {
    ///     match change.status.as_str() {
    ///         "added" => println!("New file: {}", change.file_path),
    ///         "modified" => println!("Updated: {}", change.file_path),
    ///         "removed" => println!("Deleted: {}", change.file_path),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    fn get_push_file_changes(&self) -> (Vec<FileChange>, Vec<FileChange>, Vec<FileChange>);

    /// Extract the full repository name from the webhook event
    ///
    /// Returns the repository name in the format "owner/repo".
    /// This is used to identify which repository triggered the webhook.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Full repository name (e.g., "octocat/Hello-World")
    /// * `Err(WebHooksError::MissingRepositoryName)` - Repository info not found
    /// * `Err(WebHooksError::UnsupportedWebhookEvent)` - Event type doesn't include repo info
    ///
    /// # Supported Events
    ///
    /// - Push events
    /// - Pull request events
    ///
    /// # Example
    ///
    /// ```rust
    /// let repo = event.get_repository_name()?;
    /// if allowed_repos.contains(&repo) {
    ///     // Process the webhook
    /// }
    /// ```
    fn get_repository_name(&self) -> Result<String>;

    /// Extract the repository owner username from the webhook event
    ///
    /// Returns the username or organization name that owns the repository.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Repository owner username (e.g., "octocat")
    /// * `Err(WebHooksError::MissingRepositoryName)` - Owner info not found
    /// * `Err(WebHooksError::UnsupportedWebhookEvent)` - Event type doesn't include owner info
    ///
    /// # Example
    ///
    /// ```rust
    /// let owner = event.get_repository_owner()?;
    /// let file_content = github_client.get_file_content(&owner, "repo", "path").await?;
    /// ```
    fn get_repository_owner(&self) -> Result<String>;
}

/// Implementation of WebhookHandler for octocrab's WebhookEvent
///
/// This implementation provides convenient methods for extracting information
/// from GitHub webhook events parsed by the octocrab library.
impl WebhookHandler for WebhookEvent {
    /// Extract all file changes from a push event
    ///
    /// This method iterates through all commits in a push event and collects
    /// all file changes (added, modified, removed) into a flat list.
    ///
    /// # Implementation Details
    ///
    /// 1. Checks if the event is a Push event
    /// 2. Extracts the push payload from the event
    /// 3. Iterates through all commits in the push
    /// 4. For each commit, collects added, removed, and modified files
    /// 5. Creates a FileChange object for each file with its status
    ///
    /// # Returns
    ///
    /// A vector containing all file changes across all commits. If the event
    /// is not a push event or contains no commits, returns an empty vector.
    fn get_push_file_changes(&self) -> (Vec<FileChange>, Vec<FileChange>, Vec<FileChange>) {
        let mut added_files = Vec::new();
        let mut removed_files = Vec::new();
        let mut modified_files = Vec::new();

        // Only process push events
        if self.kind == WebhookEventType::Push {
            // Extract push-specific payload
            if let WebhookEventPayload::Push(push_payload) = &self.specific {
                // Iterate through all commits in the push
                for commit in &push_payload.commits {
                    // Collect added files
                    for file in &commit.added {
                        added_files.push(FileChange {
                            file_path: file.clone(),
                            status: "added".to_string(),
                            timestamp: commit.timestamp,
                            row_url: None,
                        });
                    }

                    // Collect removed files
                    for file in &commit.removed {
                        removed_files.push(FileChange {
                            file_path: file.clone(),
                            status: "removed".to_string(),
                            timestamp: commit.timestamp,
                            row_url: None,
                        });
                    }

                    // Collect modified files
                    for file in &commit.modified {
                        modified_files.push(FileChange {
                            file_path: file.clone(),
                            status: "modified".to_string(),
                            timestamp: commit.timestamp,
                            row_url: None,
                        });
                    }
                }
            }
        }

        (added_files, removed_files, modified_files)
    }

    /// Get the full repository name from the webhook event
    ///
    /// Extracts the repository's full name (format: "owner/repo") from
    /// the webhook event payload. This works for push and pull request events.
    ///
    /// # Error Handling
    ///
    /// Returns an error if:
    /// - The event type doesn't support repository information
    /// - The repository field is missing from the payload
    /// - The repository's full_name field is None
    fn get_repository_name(&self) -> Result<String> {
        match &self.specific {
            // Handle push and pull request events
            WebhookEventPayload::Push(_) | WebhookEventPayload::PullRequest(_) => self
                .repository
                .as_ref()
                .and_then(|repo| repo.full_name.clone())
                .ok_or_else(|| WebHooksError::MissingRepositoryName.into()),

            // Other event types are not supported
            _ => Err(WebHooksError::UnsupportedWebhookEvent.into()),
        }
    }

    /// Get the repository owner username from the webhook event
    ///
    /// Extracts the repository owner's username from the webhook event.
    /// This is useful for making authenticated API requests to fetch
    /// file content from the repository.
    ///
    /// # Error Handling
    ///
    /// Returns an error if:
    /// - The event type doesn't support repository information
    /// - The repository field is missing from the payload
    /// - The owner field is missing or None
    fn get_repository_owner(&self) -> Result<String> {
        match &self.specific {
            // Handle push and pull request events
            WebhookEventPayload::Push(_) | WebhookEventPayload::PullRequest(_) => self
                .repository
                .as_ref()
                .and_then(|repo| repo.owner.as_ref())
                .map(|owner| owner.login.clone())
                .ok_or_else(|| WebHooksError::MissingRepositoryName.into()),

            // Other event types are not supported
            _ => Err(WebHooksError::UnsupportedWebhookEvent.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_change_creation() {
        let change = FileChange {
            file_path: "test.md".to_string(),
            status: "added".to_string(),
            timestamp: Utc::now(),
            row_url: None,
        };

        assert_eq!(change.file_path, "test.md");
        assert_eq!(change.status, "added");
        assert!(change.row_url.is_none());
    }

    #[test]
    fn test_file_change_serialization() {
        let change = FileChange {
            file_path: "test.md".to_string(),
            status: "modified".to_string(),
            timestamp: Utc::now(),
            row_url: Some("https://github.com/...".to_string()),
        };

        let json = serde_json::to_string(&change).unwrap();
        assert!(json.contains("test.md"));
        assert!(json.contains("modified"));
    }
}
