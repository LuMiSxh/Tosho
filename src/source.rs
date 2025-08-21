//! Source trait and collection for managing manga sources.
//!
//! This module defines the core [`Source`] trait that all manga sources must implement,
//! and the [`Sources`] collection for managing multiple sources. It provides both
//! individual source operations and aggregated operations across all sources.
//!
//! # Examples
//!
//! ```rust
//! use tosho::prelude::*;
//! use tosho::error::Result;
//!
//! # async fn example() -> Result<()> {
//! let mut sources = Sources::new();
//! // sources.add(MangaDexSource::new());
//! // sources.add(MadaraSource::new("https://example.com"));
//!
//! // Search across all sources
//! let results = sources.search("one piece").limit(10).flatten().await?;
//!
//! // Get chapters from a specific source
//! if let Some(source) = sources.get("mangadex") {
//!     let chapters = source.get_chapters("manga_id").await?;
//! }
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use futures::future;
use std::collections::HashMap;

use crate::{
    error::Result,
    search::SearchBuilder,
    types::{Chapter, Manga, SearchParams},
};

/// Trait that all manga sources must implement.
///
/// The `Source` trait defines the interface for manga sources, providing methods
/// for searching manga, retrieving chapters, and getting page URLs. Each source
/// implementation handles the specifics of communicating with its respective
/// manga website or API.
///
/// # Required Methods
///
/// * [`id()`](Source::id) - Unique identifier for the source
/// * [`name()`](Source::name) - Human-readable name
/// * [`base_url()`](Source::base_url) - Base URL of the source
/// * [`search()`](Source::search) - Search for manga
/// * [`get_chapters()`](Source::get_chapters) - Get chapters for a manga
/// * [`get_pages()`](Source::get_pages) - Get page URLs for a chapter
///
/// # Implementation Guidelines
///
/// - Use the [`net::HttpClient`](crate::net::HttpClient) for HTTP requests
/// - Implement proper rate limiting to respect website policies
/// - Return detailed errors using the [`Error`](crate::Error) types
/// - Ensure all returned manga have the correct `source_id` set
///
/// # Examples
///
/// ```rust
/// use tosho::prelude::*;
/// use tosho::error::Result;
/// use async_trait::async_trait;
///
/// struct MyMangaSource {
///     base_url: String,
///     client: tosho::net::HttpClient,
/// }
///
/// #[async_trait]
/// impl Source for MyMangaSource {
///     fn id(&self) -> &'static str { "my_source" }
///     fn name(&self) -> &'static str { "My Manga Source" }
///     fn base_url(&self) -> &str { &self.base_url }
///
///     async fn search(&self, params: SearchParams) -> Result<Vec<Manga>> {
///         // Implementation here
/// #       Ok(vec![])
///     }
///
///     async fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>> {
///         // Implementation here
/// #       Ok(vec![])
///     }
///
///     async fn get_pages(&self, chapter_id: &str) -> Result<Vec<String>> {
///         // Implementation here
/// #       Ok(vec![])
///     }
/// }
/// ```
#[async_trait]
pub trait Source: Send + Sync {
    /// Returns the unique identifier for this source.
    ///
    /// The ID should be a lowercase, hyphen-separated string that uniquely
    /// identifies this source. It's used for source selection and internal
    /// mapping.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # struct MySource;
    /// # impl MySource { fn new() -> Self { Self } }
    /// # #[async_trait::async_trait]
    /// # impl Source for MySource {
    /// fn id(&self) -> &'static str {
    ///     "mangadex"  // or "madara-site", "my-custom-source", etc.
    /// }
    /// #   fn name(&self) -> &'static str { "MangaDex" }
    /// #   fn base_url(&self) -> &str { "https://mangadex.org" }
    /// #   async fn search(&self, params: tosho::SearchParams) -> tosho::Result<Vec<tosho::Manga>> { Ok(vec![]) }
    /// #   async fn get_chapters(&self, manga_id: &str) -> tosho::Result<Vec<tosho::Chapter>> { Ok(vec![]) }
    /// #   async fn get_pages(&self, chapter_id: &str) -> tosho::Result<Vec<String>> { Ok(vec![]) }
    /// # }
    /// ```
    fn id(&self) -> &'static str;

    /// Returns the human-readable name of this source.
    ///
    /// This name is displayed to users and should be the official name
    /// of the manga website or service.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # struct MySource;
    /// # #[async_trait::async_trait]
    /// # impl Source for MySource {
    /// #   fn id(&self) -> &'static str { "mangadx" }
    /// fn name(&self) -> &'static str {
    ///     "MangaDex"  // or "MangaPlus", "Viz Media", etc.
    /// }
    /// #   fn base_url(&self) -> &str { "https://mangadex.org" }
    /// #   async fn search(&self, params: tosho::SearchParams) -> tosho::Result<Vec<tosho::Manga>> { Ok(vec![]) }
    /// #   async fn get_chapters(&self, manga_id: &str) -> tosho::Result<Vec<tosho::Chapter>> { Ok(vec![]) }
    /// #   async fn get_pages(&self, chapter_id: &str) -> tosho::Result<Vec<String>> { Ok(vec![]) }
    /// # }
    /// ```
    fn name(&self) -> &'static str;

    /// Returns the base URL of this source.
    ///
    /// This should be the root URL of the manga website, without any
    /// trailing slashes or specific paths.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # struct MySource;
    /// # #[async_trait::async_trait]
    /// # impl Source for MySource {
    /// #   fn id(&self) -> &'static str { "mangadx" }
    /// #   fn name(&self) -> &'static str { "MangaDex" }
    /// fn base_url(&self) -> &str {
    ///     "https://mangadx.org"  // No trailing slash
    /// }
    /// #   async fn search(&self, params: tosho::SearchParams) -> tosho::Result<Vec<tosho::Manga>> { Ok(vec![]) }
    /// #   async fn get_chapters(&self, manga_id: &str) -> tosho::Result<Vec<tosho::Chapter>> { Ok(vec![]) }
    /// #   async fn get_pages(&self, chapter_id: &str) -> tosho::Result<Vec<String>> { Ok(vec![]) }
    /// # }
    /// ```
    fn base_url(&self) -> &str;

    /// Searches for manga based on the given parameters.
    ///
    /// This method should search the source's catalog and return matching manga.
    /// The implementation should respect all search parameters when possible,
    /// but may ignore unsupported parameters.
    ///
    /// # Parameters
    ///
    /// * `params` - Search parameters including query, tags, sorting, etc.
    ///
    /// # Returns
    ///
    /// A vector of [`Manga`] objects matching the search criteria.
    ///
    /// # Errors
    ///
    /// * [`Error::Source`](crate::Error::Source) - For source-specific errors
    /// * [`Error::Network`](crate::Error::Network) - For network/connection issues
    /// * [`Error::Parse`](crate::Error::Parse) - For parsing errors
    ///
    /// # Implementation Notes
    ///
    /// - Ensure all returned manga have their `source_id` field set correctly
    /// - Apply rate limiting to avoid overwhelming the source
    /// - Handle pagination according to the `limit` and `offset` parameters
    /// - Return partial results if some data is missing rather than failing completely
    async fn search(&self, params: SearchParams) -> Result<Vec<Manga>>;

    /// Retrieves the list of chapters for a specific manga.
    ///
    /// This method fetches all available chapters for the given manga ID.
    /// Chapters should be returned in reading order (usually chronological).
    ///
    /// # Parameters
    ///
    /// * `manga_id` - The unique identifier of the manga within this source
    ///
    /// # Returns
    ///
    /// A vector of [`Chapter`] objects for the manga, typically sorted by chapter number.
    ///
    /// # Errors
    ///
    /// * [`Error::NotFound`](crate::Error::NotFound) - If the manga doesn't exist
    /// * [`Error::Source`](crate::Error::Source) - For source-specific errors
    /// * [`Error::Network`](crate::Error::Network) - For network/connection issues
    ///
    /// # Implementation Notes
    ///
    /// - Ensure all returned chapters have their `manga_id` and `source_id` fields set
    /// - Handle special chapters (like .5 chapters) using decimal numbers
    /// - Consider caching chapter lists if the source supports it
    async fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>>;

    /// Retrieves the page URLs for a specific chapter.
    ///
    /// This method fetches the URLs of all pages in the given chapter.
    /// The URLs should be returned in reading order.
    ///
    /// # Parameters
    ///
    /// * `chapter_id` - The unique identifier of the chapter within this source
    ///
    /// # Returns
    ///
    /// A vector of strings containing the URLs to each page of the chapter.
    ///
    /// # Errors
    ///
    /// * [`Error::NotFound`](crate::Error::NotFound) - If the chapter doesn't exist
    /// * [`Error::Source`](crate::Error::Source) - For source-specific errors
    /// * [`Error::Network`](crate::Error::Network) - For network/connection issues
    ///
    /// # Implementation Notes
    ///
    /// - Return direct image URLs when possible
    /// - Handle dynamic page loading if required by the source
    /// - Consider implementing lazy loading for large chapters
    async fn get_pages(&self, chapter_id: &str) -> Result<Vec<String>>;

    /// Downloads a chapter to the specified directory.
    ///
    /// This is a convenience method that combines getting pages and downloading them.
    /// The chapter will be saved in a subdirectory named after the chapter.
    ///
    /// # Parameters
    ///
    /// * `chapter_id` - The unique identifier of the chapter
    /// * `output_dir` - Base directory where the chapter should be saved
    ///
    /// # Returns
    ///
    /// The path to the downloaded chapter directory.
    ///
    /// # Default Implementation
    ///
    /// The default implementation:
    /// 1. Gets the chapter pages using `get_pages`
    /// 2. Downloads each page to `output_dir/chapter_id/`
    /// 3. Names files as `page_001.jpg`, `page_002.jpg`, etc.
    ///
    /// Sources can override this for custom download behavior.
    async fn download_chapter(
        &self,
        chapter_id: &str,
        output_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf> {
        use tokio::fs;
        use tokio::io::AsyncWriteExt;

        let pages = self.get_pages(chapter_id).await?;
        if pages.is_empty() {
            return Err(crate::Error::source(
                self.id(),
                "No pages found for chapter",
            ));
        }

        // Create chapter directory
        let chapter_dir = output_dir.join(format!("chapter_{}", chapter_id));
        fs::create_dir_all(&chapter_dir).await.map_err(|e| {
            crate::Error::source(self.id(), format!("Failed to create directory: {}", e))
        })?;

        // Download each page
        let client = reqwest::Client::new();
        for (i, page_url) in pages.iter().enumerate() {
            let response = client.get(page_url).send().await.map_err(|e| {
                crate::Error::parse(format!("Failed to download page {}: {}", i + 1, e))
            })?;

            if !response.status().is_success() {
                return Err(crate::Error::parse(format!(
                    "Failed to download page {}: HTTP {}",
                    i + 1,
                    response.status()
                )));
            }

            let bytes = response.bytes().await.map_err(|e| {
                crate::Error::parse(format!("Failed to read page {} data: {}", i + 1, e))
            })?;

            // Determine file extension from URL or default to jpg
            let extension = page_url
                .split('?')
                .next()
                .and_then(|url| url.split('.').last())
                .filter(|ext| ext.len() <= 4)
                .unwrap_or("jpg");

            let filename = format!("page_{:03}.{}", i + 1, extension);
            let filepath = chapter_dir.join(filename);

            let mut file = fs::File::create(&filepath).await.map_err(|e| {
                crate::Error::source(self.id(), format!("Failed to create file: {}", e))
            })?;

            file.write_all(&bytes).await.map_err(|e| {
                crate::Error::source(self.id(), format!("Failed to write file: {}", e))
            })?;
        }

        println!(
            "Downloaded {} pages to {}",
            pages.len(),
            chapter_dir.display()
        );
        Ok(chapter_dir)
    }
}

/// A collection of manga sources with convenience methods for management and aggregation.
///
/// `Sources` manages multiple [`Source`] implementations and provides high-level
/// operations for searching across all sources, managing source collections,
/// and accessing individual sources.
///
/// # Features
///
/// - **Source Management**: Add, remove, and retrieve sources by ID
/// - **Aggregated Search**: Search across all sources simultaneously
/// - **Fluent API**: Chain search parameters and execution strategies
/// - **Error Handling**: Graceful handling of individual source failures
///
/// # Examples
///
/// ```rust
/// use tosho::prelude::*;
/// use tosho::error::Result;
///
/// # async fn example() -> Result<()> {
/// let mut sources = Sources::new();
/// // sources.add(MangaDexSource::new());
/// // sources.add(MadaraSource::new("https://example.com"));
///
/// // Search all sources
/// let results = sources.search("one piece").limit(10).flatten().await?;
///
/// // Search specific source
/// let mangadx_results = sources.search("naruto").from_source("mangadx").await?;
///
/// // Get source information
/// println!("Available sources: {:?}", sources.list_ids());
/// println!("Total sources: {}", sources.len());
/// # Ok(())
/// # }
/// ```
pub struct Sources {
    sources: Vec<Box<dyn Source>>,
    by_id: HashMap<String, usize>,
}

impl Sources {
    /// Creates a new empty source collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::prelude::*;
    ///
    /// let sources = Sources::new();
    /// assert_eq!(sources.len(), 0);
    /// assert!(sources.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            by_id: HashMap::new(),
        }
    }

    /// Starts a fluent search across all sources.
    ///
    /// This method returns a [`SearchBuilder`] that allows you to chain search
    /// parameters and execute the search with different strategies.
    ///
    /// # Parameters
    ///
    /// * `query` - The search query string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::prelude::*;
    /// use tosho::error::Result;
    ///
    /// # async fn example() -> Result<()> {
    /// let sources = Sources::new();
    ///
    /// // Simple search
    /// let results = sources.search("one piece").flatten().await?;
    ///
    /// // Advanced search with parameters
    /// let filtered = sources
    ///     .search("manga")
    ///     .limit(20)
    ///     .include_tags(vec!["Action".to_string()])
    ///     .sort_by(SortOrder::UpdatedAt)
    ///     .flatten()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn search(&self, query: impl Into<String>) -> SearchBuilder<'_> {
        SearchBuilder::new(self, query)
    }

    /// Adds a source to the collection.
    ///
    /// The source is added to the internal collection and indexed by its ID
    /// for fast retrieval. Returns a mutable reference to self for chaining.
    ///
    /// # Parameters
    ///
    /// * `source` - Any type implementing the [`Source`] trait
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::prelude::*;
    ///
    /// let mut sources = Sources::new();
    /// // sources.add(MangaDexSource::new())
    /// //        .add(MadaraSource::new("https://example.com"));
    ///
    /// // println!("Added {} sources", sources.len());
    /// ```
    pub fn add(&mut self, source: impl Source + 'static) -> &mut Self {
        let id = source.id().to_string();
        let index = self.sources.len();
        self.sources.push(Box::new(source));
        self.by_id.insert(id, index);
        self
    }

    /// Retrieves a source by its ID.
    ///
    /// # Parameters
    ///
    /// * `id` - The unique identifier of the source
    ///
    /// # Returns
    ///
    /// * `Some(&dyn Source)` - Reference to the source if found
    /// * `None` - If no source with the given ID exists
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::prelude::*;
    /// use tosho::error::Result;
    ///
    /// # async fn example() -> Result<()> {
    /// let sources = Sources::new();
    ///
    /// if let Some(source) = sources.get("mangadx") {
    ///     println!("Found source: {}", source.name());
    ///     let chapters = source.get_chapters("manga_id").await?;
    /// } else {
    ///     println!("Source not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, id: &str) -> Option<&dyn Source> {
        self.by_id
            .get(id)
            .and_then(|&index| self.sources.get(index))
            .map(|s| s.as_ref())
    }

    /// Returns a list of all source IDs in the collection.
    ///
    /// # Returns
    ///
    /// A vector containing the IDs of all registered sources.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::prelude::*;
    ///
    /// let mut sources = Sources::new();
    /// // sources.add(MangaDexSource::new());
    /// // sources.add(MadaraSource::new("https://example.com"));
    ///
    /// let ids = sources.list_ids();
    /// // println!("Available sources: {:?}", ids);
    /// ```
    pub fn list_ids(&self) -> Vec<&'static str> {
        self.sources.iter().map(|s| s.id()).collect()
    }

    /// Searches all sources and returns results grouped by source.
    ///
    /// This method executes the search across all registered sources concurrently
    /// and returns the results grouped by source ID. Each source's result is
    /// returned separately, allowing you to handle successes and failures individually.
    ///
    /// # Parameters
    ///
    /// * `params` - Search parameters to use for all sources
    ///
    /// # Returns
    ///
    /// A vector of tuples containing:
    /// - Source ID (String)
    /// - Search result (`Result<Vec<Manga>>`) for that source
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::prelude::*;
    /// use tosho::error::Result;
    ///
    /// # async fn example() -> Result<()> {
    /// let sources = Sources::new();
    /// let params = SearchParams::from("one piece");
    ///
    /// let grouped = sources.search_all_grouped(params).await;
    /// for (source_id, result) in grouped {
    ///     match result {
    ///         Ok(manga) => println!("{}: {} results", source_id, manga.len()),
    ///         Err(e) => println!("{}: Error - {}", source_id, e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_all_grouped(
        &self,
        params: SearchParams,
    ) -> Vec<(String, Result<Vec<Manga>>)> {
        let futures = self.sources.iter().map(|source| {
            let params = params.clone();
            async move {
                let source_id = source.id().to_string();
                let result = source.search(params).await.map(|mut manga| {
                    // Add source_id to each manga
                    for m in &mut manga {
                        m.source_id = source_id.clone();
                    }
                    manga
                });
                (source_id, result)
            }
        });

        future::join_all(futures).await
    }

    /// Searches all sources and returns flattened results.
    ///
    /// This method executes the search across all registered sources concurrently
    /// and combines all successful results into a single vector. Individual source
    /// failures are logged but don't prevent other sources from returning results.
    ///
    /// # Parameters
    ///
    /// * `params` - Search parameters to use for all sources
    ///
    /// # Returns
    ///
    /// A single vector containing all manga found across all sources.
    ///
    /// # Errors
    ///
    /// Returns an error only if all sources fail. Individual source failures
    /// are ignored as long as at least one source returns results.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::prelude::*;
    /// use tosho::error::Result;
    ///
    /// # async fn example() -> Result<()> {
    /// let sources = Sources::new();
    /// let params = SearchParams::from("one piece");
    ///
    /// let all_results = sources.search_all_flat(params).await?;
    /// println!("Found {} total results across all sources", all_results.len());
    ///
    /// // Process all results together
    /// for manga in all_results {
    ///     println!("{} from {}", manga.title, manga.source_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_all_flat(&self, params: SearchParams) -> Result<Vec<Manga>> {
        let grouped = self.search_all_grouped(params).await;

        let mut all_results = Vec::new();
        let mut errors = Vec::new();

        for (source_id, result) in grouped {
            match result {
                Ok(mut manga) => all_results.append(&mut manga),
                Err(e) => errors.push(format!("{}: {}", source_id, e)),
            }
        }

        // If all sources failed, return an error
        if all_results.is_empty() && !errors.is_empty() {
            return Err(crate::Error::Other(format!(
                "All sources failed: {}",
                errors.join(", ")
            )));
        }

        Ok(all_results)
    }

    /// Returns the number of sources in the collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::prelude::*;
    ///
    /// let mut sources = Sources::new();
    /// assert_eq!(sources.len(), 0);
    ///
    /// // sources.add(MangaDexSource::new());
    /// // assert_eq!(sources.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Returns `true` if the collection contains no sources.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::prelude::*;
    ///
    /// let mut sources = Sources::new();
    /// assert!(sources.is_empty());
    ///
    /// // sources.add(MangaDexSource::new());
    /// // assert!(!sources.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

impl Default for Sources {
    fn default() -> Self {
        Self::new()
    }
}
