//! Error types and result handling for Tosho operations.
//!
//! This module defines the comprehensive error handling system used throughout Tosho.
//! All operations return a [`Result<T>`] which is a type alias for `std::result::Result<T, Error>`.
//!
//! # Error Categories
//!
//! Tosho errors are categorized into several types:
//!
//! - **Network Errors**: Connection issues, timeouts, HTTP errors
//! - **Parse Errors**: Invalid HTML, JSON, or data format issues
//! - **Source Errors**: Website-specific errors with context
//! - **Not Found**: Missing manga, chapters, or sources
//! - **Rate Limiting**: When requests are throttled
//! - **IO Errors**: File system or other IO operations
//! - **JSON Errors**: Serialization/deserialization failures
//!
//! # Examples
//!
//! ```rust
//! use tosho::prelude::*;
//! use tosho::error::{Result, Error};
//!
//! # async fn example() -> Result<()> {
//! let sources = Sources::new();
//!
//! match sources.search("nonexistent").from_source("invalid").await {
//!     Ok(results) => println!("Found {} results", results.len()),
//!     Err(Error::NotFound(msg)) => println!("Source not found: {}", msg),
//!     Err(Error::Network(e)) => println!("Network error: {}", e),
//!     Err(e) => println!("Other error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```

use thiserror::Error;

/// Type alias for Results with Tosho errors.
///
/// This is a convenience type alias that represents the standard Result type
/// with Tosho's [`enum@Error`] as the error type. All public APIs in Tosho return
/// this Result type.
///
/// # Examples
///
/// ```rust
/// use tosho::{Result, Error};
///
/// fn example_operation() -> Result<String> {
///     Ok("Success".to_string())
/// }
///
/// fn example_with_error() -> Result<()> {
///     Err(Error::parse("Something went wrong"))
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error type for all Tosho operations.
///
/// This enum covers all possible error conditions that can occur during
/// manga source operations, from network issues to parsing failures.
/// Each variant provides specific context about what went wrong.
///
/// # Variants
///
/// * [`Network`](Error::Network) - HTTP client and connection errors
/// * [`Parse`](Error::Parse) - Data parsing and format errors
/// * [`Source`](Error::Source) - Source-specific errors with context
/// * [`NotFound`](Error::NotFound) - Missing resources
/// * [`RateLimit`](Error::RateLimit) - Rate limiting responses
/// * [`Io`](Error::Io) - File system and IO errors
/// * [`Json`](Error::Json) - JSON serialization errors
/// * [`Other`](Error::Other) - Generic error messages
#[derive(Error, Debug)]
pub enum Error {
    /// Network-related errors from HTTP operations.
    ///
    /// This variant wraps errors from the underlying HTTP client (reqwest),
    /// including connection timeouts, DNS resolution failures, and HTTP
    /// transport errors.
    ///
    /// # Examples
    ///
    /// Common scenarios that produce this error:
    /// - Connection timeouts
    /// - DNS resolution failures
    /// - TLS/SSL certificate errors
    /// - Network connectivity issues
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// HTML/JSON parsing and data format errors.
    ///
    /// This variant is used when the received data cannot be parsed as expected,
    /// such as malformed HTML, unexpected JSON structure, or missing required
    /// fields in the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::Error;
    ///
    /// let error = Error::parse("Invalid manga ID format");
    /// let error = Error::parse("Missing title field in response");
    /// ```
    #[error("Parse error: {0}")]
    Parse(String),

    /// Source-specific errors with contextual information.
    ///
    /// This variant provides detailed error information when a specific manga
    /// source encounters an error. It includes both the source identifier and
    /// a descriptive error message.
    ///
    /// # Fields
    ///
    /// * `src` - The identifier of the source that encountered the error
    /// * `message` - Descriptive error message explaining what went wrong
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::Error;
    ///
    /// let error = Error::source("mangadex", "API rate limit exceeded");
    /// let error = Error::source("madara-site", "Invalid chapter ID");
    /// ```
    #[error("Source error [{src}]: {message}")]
    Source { src: String, message: String },

    /// Resource not found errors.
    ///
    /// This variant is used when a requested resource (manga, chapter, source, etc.)
    /// cannot be found. It provides a descriptive message about what was not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::Error;
    ///
    /// let error = Error::not_found("Manga with ID 'invalid-id'");
    /// let error = Error::not_found("Source: nonexistent-source");
    /// ```
    #[error("Not found: {0}")]
    NotFound(String),

    /// Rate limiting errors from manga sources.
    ///
    /// This variant indicates that the source has rate-limited the requests.
    /// It optionally includes the number of seconds to wait before retrying,
    /// as provided by the source's `Retry-After` header.
    ///
    /// # Fields
    ///
    /// * `retry_after` - Optional number of seconds to wait before retrying
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::Error;
    ///
    /// // Rate limited with specific retry time
    /// let error = Error::rate_limit(Some(60)); // Retry after 60 seconds
    ///
    /// // Rate limited without specific retry time
    /// let error = Error::rate_limit(None);
    /// ```
    #[error("Rate limited, retry after {retry_after:?} seconds")]
    RateLimit { retry_after: Option<u64> },

    /// File system and IO operation errors.
    ///
    /// This variant wraps standard IO errors that may occur during file
    /// operations, such as reading configuration files or writing cache data.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization and deserialization errors.
    ///
    /// This variant wraps errors from serde_json when parsing JSON responses
    /// from manga sources or serializing data structures.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Image processing errors.
    ///
    /// This variant wraps errors from image processing operations, such as
    /// converting images.
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    /// Join errors.
    ///
    /// This variant wraps errors from tokio tasks.
    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    /// Generic error messages.
    ///
    /// This variant is used for errors that don't fit into other specific
    /// categories. It contains a descriptive error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::Error;
    ///
    /// let error = Error::Other("Unexpected error condition".to_string());
    /// ```
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Creates a parse error with the given message.
    ///
    /// This is a convenience method for creating [`Error::Parse`] variants
    /// with a descriptive message about what parsing operation failed.
    ///
    /// # Parameters
    ///
    /// * `msg` - A message describing the parsing error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::Error;
    ///
    /// let error = Error::parse("Invalid manga ID format");
    /// let error = Error::parse(format!("Expected {} chapters, found {}", 10, 5));
    /// ```
    pub fn parse(msg: impl Into<String>) -> Self {
        Error::Parse(msg.into())
    }

    /// Creates a source-specific error with source ID and message.
    ///
    /// This is a convenience method for creating [`Error::Source`] variants
    /// with both the source identifier and a descriptive error message.
    ///
    /// # Parameters
    ///
    /// * `src` - The identifier of the source that encountered the error
    /// * `msg` - A message describing what went wrong
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::Error;
    ///
    /// let error = Error::source("mangadex", "API endpoint not found");
    /// let error = Error::source("madara-site", "Invalid response format");
    /// ```
    pub fn source(src: impl Into<String>, msg: impl Into<String>) -> Self {
        Error::Source {
            src: src.into(),
            message: msg.into(),
        }
    }

    /// Creates a not found error with the given message.
    ///
    /// This is a convenience method for creating [`Error::NotFound`] variants
    /// with a descriptive message about what resource was not found.
    ///
    /// # Parameters
    ///
    /// * `msg` - A message describing what was not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::Error;
    ///
    /// let error = Error::not_found("Manga with ID 'abc123'");
    /// let error = Error::not_found("Chapter 999 for manga 'one-piece'");
    /// ```
    pub fn not_found(msg: impl Into<String>) -> Self {
        Error::NotFound(msg.into())
    }

    /// Creates a rate limit error with optional retry-after time.
    ///
    /// This is a convenience method for creating [`Error::RateLimit`] variants.
    /// The retry-after parameter typically comes from the `Retry-After` HTTP header.
    ///
    /// # Parameters
    ///
    /// * `retry_after` - Optional number of seconds to wait before retrying
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::Error;
    ///
    /// // Rate limited with specific retry time
    /// let error = Error::rate_limit(Some(60));
    ///
    /// // Rate limited without specific retry time
    /// let error = Error::rate_limit(None);
    /// ```
    pub fn rate_limit(retry_after: Option<u64>) -> Self {
        Error::RateLimit { retry_after }
    }
}
