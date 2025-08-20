//! Network utilities for HTTP requests, rate limiting, and content parsing.
//!
//! This module provides the networking infrastructure for Tosho, including:
//!
//! - **HTTP Client**: A global, configured HTTP client with connection pooling
//! - **Rate Limiting**: Per-source rate limiting to respect website policies
//! - **Retry Logic**: Automatic retries with exponential backoff
//! - **Content Parsing**: HTML and JSON parsing utilities
//!
//! # Examples
//!
//! ```rust
//! use tosho::net::HttpClient;
//!
//! # async fn example() -> tosho::Result<()> {
//! let client = HttpClient::new("my_source")
//!     .with_rate_limit(500)  // 500ms between requests
//!     .with_max_retries(3);
//!
//! let html = client.get_text("https://example.com").await?;
//! let json: serde_json::Value = client.get_json("https://api.example.com").await?;
//! # Ok(())
//! # }
//! ```

use bytes::Bytes;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use reqwest::{Client, header::HeaderMap};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod html;
pub mod json;

/// Global HTTP client instance with optimized configuration.
///
/// This client is configured with:
/// - 30-second timeout
/// - Connection pooling (10 idle connections per host)
/// - Compression support (gzip, brotli)
/// - Custom User-Agent header
///
/// The client is created lazily on first use and reused across all HTTP operations.
static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Tosho/0.1.0")
        .pool_max_idle_per_host(10)
        .gzip(true)
        .brotli(true)
        .build()
        .expect("Failed to build HTTP client")
});

/// Per-source rate limiter to prevent overwhelming manga websites.
///
/// The rate limiter tracks the last request time for each source and enforces
/// a minimum delay between requests. This helps respect website policies and
/// prevents getting rate-limited or banned.
///
/// # Thread Safety
///
/// The rate limiter uses a `Mutex` internally and is safe to use across multiple
/// threads and async tasks.
#[derive(Debug)]
pub struct RateLimiter {
    last_request: Mutex<HashMap<String, Instant>>,
    default_delay: Duration,
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            last_request: Mutex::new(HashMap::new()),
            default_delay: self.default_delay,
        }
    }
}

impl RateLimiter {
    /// Creates a new rate limiter with the specified default delay.
    ///
    /// # Parameters
    ///
    /// * `delay_ms` - Minimum delay between requests in milliseconds
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::RateLimiter;
    ///
    /// // Create a rate limiter with 500ms delay
    /// let limiter = RateLimiter::new(500);
    /// ```
    pub fn new(delay_ms: u64) -> Self {
        Self {
            last_request: Mutex::new(HashMap::new()),
            default_delay: Duration::from_millis(delay_ms),
        }
    }

    /// Waits if necessary before allowing a request for the specified source.
    ///
    /// This method checks the last request time for the source and sleeps if
    /// insufficient time has passed since the last request.
    ///
    /// # Parameters
    ///
    /// * `source_id` - The identifier of the source making the request
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::RateLimiter;
    ///
    /// # async fn example() {
    /// let limiter = RateLimiter::new(1000); // 1 second delay
    /// limiter.wait("mangadex").await; // Will wait if needed
    /// # }
    /// ```
    pub async fn wait(&self, source_id: &str) {
        let now = Instant::now();
        let wait_duration = {
            let last_map = self.last_request.lock();
            if let Some(&last) = last_map.get(source_id) {
                let elapsed = now.duration_since(last);
                if elapsed < self.default_delay {
                    Some(self.default_delay - elapsed)
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(duration) = wait_duration {
            tokio::time::sleep(duration).await;
        }

        self.last_request
            .lock()
            .insert(source_id.to_string(), Instant::now());
    }

    /// Waits with a custom delay for a specific source.
    ///
    /// This method allows overriding the default delay for a specific request,
    /// useful when a source has special rate limiting requirements.
    ///
    /// # Parameters
    ///
    /// * `source_id` - The identifier of the source making the request
    /// * `delay` - Custom delay duration for this source
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::RateLimiter;
    /// use std::time::Duration;
    ///
    /// # async fn example() {
    /// let limiter = RateLimiter::new(500);
    /// // Use a longer delay for a specific source
    /// limiter.wait_custom("slow_source", Duration::from_secs(2)).await;
    /// # }
    /// ```
    pub async fn wait_custom(&self, source_id: &str, delay: Duration) {
        let now = Instant::now();
        let wait_duration = {
            let last_map = self.last_request.lock();
            if let Some(&last) = last_map.get(source_id) {
                let elapsed = now.duration_since(last);
                if elapsed < delay {
                    Some(delay - elapsed)
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(duration) = wait_duration {
            tokio::time::sleep(duration).await;
        }

        self.last_request
            .lock()
            .insert(source_id.to_string(), Instant::now());
    }
}

/// HTTP client wrapper with built-in rate limiting and retry logic.
///
/// `HttpClient` provides a high-level interface for making HTTP requests with
/// automatic rate limiting, retries, and error handling. Each client is associated
/// with a specific source and applies rate limiting per-source.
///
/// # Features
///
/// - **Rate Limiting**: Automatic delays between requests
/// - **Retry Logic**: Exponential backoff for failed requests
/// - **Error Handling**: Comprehensive error types with context
/// - **Content Types**: Built-in support for text and JSON responses
///
/// # Examples
///
/// ```rust
/// use tosho::net::HttpClient;
///
/// # async fn example() -> tosho::Result<()> {
/// let client = HttpClient::new("mangadex")
///     .with_rate_limit(1000)  // 1 second between requests
///     .with_max_retries(5);   // Retry up to 5 times
///
/// let html = client.get_text("https://mangadex.org/title/123").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct HttpClient {
    source_id: String,
    rate_limiter: RateLimiter,
    max_retries: u32,
    headers: HeaderMap,
}

impl HttpClient {
    /// Creates a new HTTP client for the specified source.
    ///
    /// The client is initialized with sensible defaults:
    /// - 200ms rate limit delay
    /// - 3 maximum retries
    ///
    /// # Parameters
    ///
    /// * `source_id` - Identifier for the source (used for rate limiting)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::HttpClient;
    ///
    /// let client = HttpClient::new("my_manga_source");
    /// ```
    pub fn new(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            rate_limiter: RateLimiter::new(200), // 200ms default
            max_retries: 3,
            headers: HeaderMap::new(),
        }
    }

    /// Sets the rate limit delay for this client.
    ///
    /// # Parameters
    ///
    /// * `delay_ms` - Minimum delay between requests in milliseconds
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::HttpClient;
    ///
    /// let client = HttpClient::new("source")
    ///     .with_rate_limit(1000); // 1 second between requests
    /// ```
    pub fn with_rate_limit(mut self, delay_ms: u64) -> Self {
        self.rate_limiter = RateLimiter::new(delay_ms);
        self
    }

    /// Sets the maximum number of retries for failed requests.
    ///
    /// # Parameters
    ///
    /// * `retries` - Maximum number of retry attempts
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::HttpClient;
    ///
    /// let client = HttpClient::new("source")
    ///     .with_max_retries(5); // Retry up to 5 times
    /// ```
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Adds a custom header to all requests made by this client.
    ///
    /// # Parameters
    ///
    /// * `name` - Header name
    /// * `value` - Header value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::HttpClient;
    ///
    /// let client = HttpClient::new("source")
    ///     .with_header("User-Agent", "MyBot/1.0")
    ///     .with_header("Referer", "https://example.com");
    /// ```
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        if let (Ok(name), Ok(value)) = (
            name.parse::<reqwest::header::HeaderName>(),
            value.parse::<reqwest::header::HeaderValue>(),
        ) {
            self.headers.insert(name, value);
        }
        self
    }

    /// Performs a GET request with automatic retry logic and rate limiting.
    ///
    /// This method applies rate limiting, handles HTTP errors, and retries failed
    /// requests with exponential backoff. It handles 429 (Too Many Requests) responses
    /// specially by respecting the `Retry-After` header.
    ///
    /// # Parameters
    ///
    /// * `url` - The URL to request
    ///
    /// # Returns
    ///
    /// The response body as `Bytes` on success.
    ///
    /// # Errors
    ///
    /// * [`Error::RateLimit`](crate::Error::RateLimit) - If rate limited after retries
    /// * [`Error::Source`](crate::Error::Source) - For HTTP errors (4xx, 5xx)
    /// * [`Error::Network`](crate::Error::Network) - For network/connection errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::HttpClient;
    ///
    /// # async fn example() -> tosho::Result<()> {
    /// let client = HttpClient::new("source");
    /// let response = client.get("https://example.com/api/manga/123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, url: &str) -> crate::Result<Bytes> {
        let mut attempts = 0;

        loop {
            // Apply rate limiting
            self.rate_limiter.wait(&self.source_id).await;

            match CLIENT.get(url).headers(self.headers.clone()).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response.bytes().await?);
                    }

                    // Handle rate limiting
                    if response.status() == 429 {
                        if attempts < self.max_retries {
                            attempts += 1;
                            let delay = Duration::from_secs(2_u64.pow(attempts));
                            tokio::time::sleep(delay).await;
                            continue;
                        }

                        let retry_after = response
                            .headers()
                            .get("retry-after")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.parse::<u64>().ok());

                        return Err(crate::Error::rate_limit(retry_after));
                    }

                    // Other HTTP errors
                    return Err(crate::Error::source(
                        &self.source_id,
                        format!("HTTP {}", response.status()),
                    ));
                }
                Err(e) => {
                    if attempts < self.max_retries {
                        attempts += 1;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    return Err(e.into());
                }
            }
        }
    }

    /// Performs a GET request and returns the response as a UTF-8 string.
    ///
    /// This is a convenience method that calls [`get()`](HttpClient::get) and converts
    /// the response bytes to a string.
    ///
    /// # Parameters
    ///
    /// * `url` - The URL to request
    ///
    /// # Returns
    ///
    /// The response body as a `String` on success.
    ///
    /// # Errors
    ///
    /// * All errors from [`get()`](HttpClient::get)
    /// * [`Error::Parse`](crate::Error::Parse) - If the response is not valid UTF-8
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::HttpClient;
    ///
    /// # async fn example() -> tosho::Result<()> {
    /// let client = HttpClient::new("source");
    /// let html = client.get_text("https://example.com/manga/123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_text(&self, url: &str) -> crate::Result<String> {
        let bytes = self.get(url).await?;
        String::from_utf8(bytes.to_vec())
            .map_err(|e| crate::Error::parse(format!("Invalid UTF-8: {}", e)))
    }

    /// Performs a GET request and deserializes the response as JSON.
    ///
    /// This is a convenience method that calls [`get()`](HttpClient::get) and
    /// deserializes the response bytes as JSON using serde.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize the JSON into
    ///
    /// # Parameters
    ///
    /// * `url` - The URL to request
    ///
    /// # Returns
    ///
    /// The deserialized JSON data as type `T` on success.
    ///
    /// # Errors
    ///
    /// * All errors from [`get()`](HttpClient::get)
    /// * [`Error::Json`](crate::Error::Json) - If JSON parsing fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::net::HttpClient;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct ApiResponse {
    ///     title: String,
    ///     chapters: Vec<String>,
    /// }
    ///
    /// # async fn example() -> tosho::Result<()> {
    /// let client = HttpClient::new("source");
    /// let data: ApiResponse = client.get_json("https://api.example.com/manga/123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_json<T>(&self, url: &str) -> crate::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let bytes = self.get(url).await?;
        serde_json::from_slice(&bytes).map_err(Into::into)
    }
}
