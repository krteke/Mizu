use mimalloc::MiMalloc;
use std::{net::SocketAddr, sync::Arc};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::app_state::AppState;
use crate::interfaces::http::route::router;

// Module declarations
// These tell the Rust compiler where to find the module definitions
mod app_state;
mod application;
mod config; // Configuration management module
mod domain;
mod errors; // Custom error handling module
mod infrastructure;
mod interfaces;

// Global memory allocator using MiMalloc
// MiMalloc is a high-performance memory allocator that can significantly
// improve allocation performance compared to the system's default allocator
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Server default configuration constants
const DEFAULT_PORT: u16 = 8124; // Default server port
const DEFAULT_HOST: &str = "0.0.0.0"; // Default host address (listens on all network interfaces)
// const DEFAULT_MAX_CONNECTIONS_PER_IP: u32 = 100;  // Maximum connections per IP (reserved for future use)

/// Main application entry point
///
/// This function initializes the server, sets up the application state,
/// and starts listening for incoming HTTP connections.
///
/// The `#[tokio::main]` macro sets up the async runtime automatically
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    // Using .ok() to ignore errors if the file doesn't exist
    dotenvy::dotenv().ok();

    // Initialize the tracing subscriber for logging
    create_tracing_subscriber();

    // Load configuration from environment variables
    // The application will panic with an error message if configuration loading fails
    let config = config::Config::new()
        .expect("Failed to load configuration. Please check your environment variables.");

    // Extract host address from config, falling back to default if not set
    let host = config.host.as_deref().unwrap_or(DEFAULT_HOST);

    // Extract port from config, falling back to default if not set
    let port = config.port.unwrap_or(DEFAULT_PORT);

    // Format the address as "host:port" string
    let addr = format!("{}:{}", host, port);

    // Parse the string into a SocketAddr
    // Will panic if the format is invalid
    let address: SocketAddr = addr.parse().expect("Invalid HOST:PORT format");

    // Bind a TCP listener to the specified address
    // This will fail if the port is already in use
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to bind to address. The port might already be in use.");

    // Print the listening address for developer convenience
    println!("Listening on http://{}", address);

    // Initialize application state with database, search service, etc.
    let state = AppState::new(config).await?;

    // Wrap the state in Arc (Atomic Reference Counting) for safe sharing across threads
    // This allows multiple handlers to access the state concurrently
    let state = Arc::new(state);

    // Spawn a background task to watch config file changes (webhook feature only)
    #[cfg(feature = "webhook")]
    tokio::spawn(watch_config_file(state.clone()));

    // Create the router with all routes configured
    let router = router().with_state(state);

    // Start the axum server and begin accepting connections
    if let Err(e) = axum::serve(listener, router).await {
        panic!("Server exited with error: {}", e);
    }

    Ok(())
}

/// Watch for configuration file changes and reload allowed repositories
///
/// This function runs in a background task and monitors the config.toml file
/// for any modifications. When changes are detected, it reloads the configuration
/// and updates the allowed repositories list.
///
/// This feature is only available when the "webhook" feature flag is enabled.
#[cfg(feature = "webhook")]
async fn watch_config_file(state: Arc<AppState>) {
    use std::path::Path;

    use notify::{Config as NotifyConfig, RecommendedWatcher, Watcher};
    use tokio::sync::mpsc;

    // Create a channel for receiving file system events
    let (tx, mut rx) = mpsc::channel(1);

    // Create a file system watcher
    let mut watcher: RecommendedWatcher = match Watcher::new(
        move |res| {
            // Send file change events through the channel
            if let Err(e) = tx.blocking_send(res) {
                tracing::error!("Failed to send file change event to channel: {}", e);
            }
        },
        NotifyConfig::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("Failed to create watcher: {}", e);
            return;
        }
    };

    // Path to the configuration file
    let file_path = Path::new("config.toml");

    // Start watching the config file (non-recursive)
    if let Err(e) = watcher.watch(file_path, notify::RecursiveMode::NonRecursive) {
        tracing::error!(
            "Failed to watch config file at '{}': {}",
            file_path.display(),
            e
        );
        return;
    }

    tracing::info!(
        "Started watching config file for changes: {}",
        file_path.display()
    );

    // Event processing loop
    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                // Check if the event is a modification or creation
                if event.kind.is_modify() || event.kind.is_create() {
                    use crate::config::Config;

                    tracing::info!("Config file change detected, attempting to reload...");

                    // Attempt to reload the configuration
                    match Config::new() {
                        Ok(new_config) => {
                            // Update the allowed repositories in the application state
                            let mut config_writer =
                                state.app_config.allowed_repositories.write().await;
                            *config_writer = new_config.allowed_repositories;

                            tracing::info!("Config file reloaded successfully");
                        }
                        Err(e) => {
                            tracing::error!("Failed to reload config file: {}", e);
                        }
                    }
                }
            }

            Err(e) => {
                tracing::error!("Failed to receive event: {}", e);
            }
        }
    }
}

/// Create and configure the tracing subscriber for application logging
///
/// This function initializes the logging system with a log level read from
/// the RUST_LOG environment variable. If not set, defaults to INFO level.
///
/// Supported log levels: trace, debug, info, warn, error
fn create_tracing_subscriber() {
    // Read log level from environment variable, defaulting to "info"
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();

    // Map the string log level to the tracing Level enum
    let level = match log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO, // Default to INFO for invalid values
    };

    // Build the subscriber with the configured log level
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    // Set as the global default subscriber
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set default tracing subscriber");
}
