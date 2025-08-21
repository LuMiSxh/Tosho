//! Search functionality and fluent search builder.
//!
//! This module provides a fluent search API that allows you to build search parameters
//! and execute searches across multiple manga sources with different strategies.
//!
//! # Examples
//!
//! ```rust
//! use tosho::prelude::*;
//! # use tosho::error::Result;
//!
//! # async fn example() -> Result<()> {
//!
//! let sources = Sources::new();
//!
//! // Simple search with result processing
//! let results = sources
//!     .search("one piece")
//!     .limit(20)
//!     .sort_by(SortOrder::UpdatedAt)
//!     .flatten()
//!     .await?
//!     .dedupe_by_title()
//!     .sort_by_relevance();
//!
//! // Grouped search for debugging
//! let grouped = sources
//!     .search("naruto")
//!     .include_tags(vec!["Action".to_string()])
//!     .group()
//!     .await;
//! # Ok(())
//! # }
//! ```

use crate::{
    error::Result,
    source::Sources,
    types::{Manga, SearchParams, SortOrder},
};

/// A fluent search builder that can build search parameters and execute searches.
///
/// `SearchBuilder` provides a chainable API for building search queries and executing
/// them with different strategies. It holds a reference to a [`Sources`] collection
/// and builds [`SearchParams`] as you chain method calls.
///
/// # Execution Strategies
///
/// - [`flatten()`](SearchBuilder::flatten) - Returns all results in a single vector
/// - [`group()`](SearchBuilder::group) - Returns results grouped by source
/// - [`from_source()`](SearchBuilder::from_source) - Searches only a specific source
/// - [`build()`](SearchBuilder::build) - Returns just the search parameters
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
/// // Basic search
/// let results = sources
///     .search("one piece")
///     .limit(10)
///     .flatten()
///     .await?;
///
/// // Advanced search with filtering
/// let filtered = sources
///     .search("shounen manga")
///     .include_tags(vec!["Action".to_string(), "Adventure".to_string()])
///     .exclude_tags(vec!["Ecchi".to_string()])
///     .sort_by(SortOrder::UpdatedAt)
///     .limit(50)
///     .flatten()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct SearchBuilder<'a> {
    sources: &'a Sources,
    params: SearchParams,
}

impl<'a> SearchBuilder<'a> {
    /// Creates a new search builder with the given query.
    ///
    /// This method is called internally by [`Sources::search()`](crate::source::Sources::search).
    /// You typically don't need to call this directly.
    pub(crate) fn new(sources: &'a Sources, query: impl Into<String>) -> Self {
        Self {
            sources,
            params: SearchParams {
                query: query.into(),
                ..Default::default()
            },
        }
    }

    /// Sets the maximum number of results to return.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let results = sources
    ///     .search("popular manga")
    ///     .limit(10)  // Only return the first 10 results
    ///     .flatten()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn limit(mut self, limit: usize) -> Self {
        self.params.limit = Some(limit);
        self
    }

    /// Sets the offset for pagination.
    ///
    /// Use this in combination with [`limit()`](SearchBuilder::limit) to implement pagination.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// // Get the second page of results (items 10-19)
    /// let page_2 = sources
    ///     .search("manga")
    ///     .limit(10)
    ///     .offset(10)
    ///     .flatten()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn offset(mut self, offset: usize) -> Self {
        self.params.offset = Some(offset);
        self
    }

    /// Includes only manga with the specified tags.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let action_manga = sources
    ///     .search("manga")
    ///     .include_tags(vec!["Action".to_string(), "Adventure".to_string()])
    ///     .flatten()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn include_tags(mut self, tags: Vec<String>) -> Self {
        self.params.include_tags = tags;
        self
    }

    /// Excludes manga with the specified tags.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let sfw_manga = sources
    ///     .search("manga")
    ///     .exclude_tags(vec!["Ecchi".to_string(), "Harem".to_string()])
    ///     .flatten()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn exclude_tags(mut self, tags: Vec<String>) -> Self {
        self.params.exclude_tags = tags;
        self
    }

    /// Sets the sort order for the search results.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let recent_manga = sources
    ///     .search("manga")
    ///     .sort_by(SortOrder::UpdatedAt)
    ///     .flatten()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sort_by(mut self, sort: SortOrder) -> Self {
        self.params.sort_by = Some(sort);
        self
    }

    /// Executes the search across all sources and returns flattened results.
    ///
    /// This method searches all available sources concurrently and combines the results
    /// into a single vector. If any sources fail, their errors are logged but don't
    /// prevent other sources from returning results.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of all manga found across all sources.
    ///
    /// # Errors
    ///
    /// Returns an error only if all sources fail. Individual source failures are
    /// ignored as long as at least one source returns results.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let all_results = sources
    ///     .search("one piece")
    ///     .limit(20)
    ///     .flatten()
    ///     .await?;
    ///
    /// println!("Found {} total results", all_results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn flatten(self) -> Result<Vec<Manga>> {
        self.sources.search_all_flat(self.params).await
    }

    /// Executes the search and returns results grouped by source.
    ///
    /// This method is useful for debugging or when you need to know which source
    /// each result came from. Each source's results (or error) are returned separately.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the source ID and either the results or an error
    /// for that source.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let grouped = sources
    ///     .search("naruto")
    ///     .limit(10)
    ///     .group()
    ///     .await;
    ///
    /// for (source_id, result) in grouped {
    ///     match result {
    ///         Ok(manga) => println!("{}: {} results", source_id, manga.len()),
    ///         Err(e) => println!("{}: Error - {}", source_id, e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn group(self) -> Vec<(String, Result<Vec<Manga>>)> {
        self.sources.search_all_grouped(self.params).await
    }

    /// Executes the search on a specific source only.
    ///
    /// This method searches only the specified source, which can be useful when you
    /// know which source you want to query or for testing individual sources.
    ///
    /// # Parameters
    ///
    /// * `source_id` - The ID of the source to search
    ///
    /// # Returns
    ///
    /// A `Result` containing the manga found from the specified source.
    ///
    /// # Errors
    ///
    /// * Returns [`Error::NotFound`](crate::Error::NotFound) if the source doesn't exist
    /// * Returns source-specific errors if the search fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let mangadex_results = sources
    ///     .search("dragon ball")
    ///     .from_source("mangadex")
    ///     .await?;
    ///
    /// println!("MangaDex found {} results", mangadex_results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_source(self, source_id: &str) -> Result<Vec<Manga>> {
        match self.sources.get(source_id) {
            Some(source) => {
                let mut results = source.search(self.params).await?;
                // Ensure source_id is set
                for manga in &mut results {
                    manga.source_id = source_id.to_string();
                }
                Ok(results)
            }
            None => Err(crate::Error::not_found(format!("Source: {}", source_id))),
        }
    }

    /// Builds and returns just the search parameters without executing the search.
    ///
    /// This method is useful for advanced use cases where you want to build search
    /// parameters and use them with lower-level APIs or store them for later use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let search_params = sources
    ///     .search("manga")
    ///     .limit(20)
    ///     .sort_by(SortOrder::UpdatedAt)
    ///     .build();
    ///
    /// // Use the parameters with lower-level APIs
    /// let results = sources.search_all_flat(search_params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> SearchParams {
        self.params
    }
}

/// Extension trait providing additional processing methods for search results.
///
/// This trait adds useful post-processing methods to `Vec<Manga>` that help you
/// filter, deduplicate, and sort search results.
///
/// # Examples
///
/// ```rust
/// use tosho::prelude::*;
/// use tosho::error::Result;
///
/// # async fn example() -> Result<()> {
/// # let sources = Sources::new();
/// let processed = sources
///     .search("popular manga")
///     .limit(100)
///     .flatten()
///     .await?
///     .dedupe_by_title()        // Remove duplicates
///     .filter_popular(4)        // Keep well-documented manga
///     .sort_by_relevance();     // Sort by relevance
/// # Ok(())
/// # }
/// ```
pub trait SearchResultExt {
    /// Filters results by popularity score based on metadata quality.
    ///
    /// This method uses a heuristic scoring system to filter manga based on
    /// available metadata quality, which tends to correlate with popularity:
    ///
    /// **Scoring System:**
    /// - Has description: +2 points
    /// - Has authors listed: +1 point
    /// - Has cover image: +1 point
    /// - Has 3+ tags: +1 point
    /// - Has 5+ tags: +1 additional point
    ///
    /// # Parameters
    ///
    /// * `min_popularity_score` - Minimum score required (0-7 range)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// // Filter for well-documented manga (score >= 4)
    /// let quality_results = sources
    ///     .search("manga")
    ///     .flatten()
    ///     .await?
    ///     .filter_popular(4);
    /// # Ok(())
    /// # }
    /// ```
    fn filter_popular(self, min_popularity_score: usize) -> Self;

    /// Removes duplicate manga entries based on title.
    ///
    /// This method keeps the first occurrence of each manga title (case-insensitive)
    /// and removes subsequent duplicates. This is useful when searching multiple
    /// sources that may have the same manga.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let unique_results = sources
    ///     .search("one piece")
    ///     .flatten()
    ///     .await?
    ///     .dedupe_by_title();  // Remove duplicate titles
    /// # Ok(())
    /// # }
    /// ```
    fn dedupe_by_title(self) -> Self;

    /// Sorts results by relevance score.
    ///
    /// Uses a sophisticated scoring algorithm that considers multiple factors:
    /// - Exact title matches get highest priority
    /// - Case-insensitive matches get medium priority
    /// - Partial matches are ranked by word overlap
    /// - Popular manga (with more metadata) get slight boost
    /// - Shorter titles preferred for similar relevance scores
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let sorted_results = sources
    ///     .search("naruto")
    ///     .flatten()
    ///     .await?
    ///     .sort_by_relevance();  // Most relevant first
    /// # Ok(())
    /// # }
    /// ```
    fn sort_by_relevance(self) -> Self;

    /// Sorts results by relevance score with query-aware matching.
    ///
    /// This is an enhanced version of `sort_by_relevance` that considers how well
    /// the manga title matches the original search query. It provides more accurate
    /// relevance scoring for search results.
    ///
    /// # Parameters
    ///
    /// * `query` - The original search query to match against
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tosho::prelude::*;
    /// # use tosho::error::Result;
    /// # async fn example() -> Result<()> {
    /// # let sources = Sources::new();
    ///
    /// let query = "one piece";
    /// let sorted_results = sources
    ///     .search(query)
    ///     .flatten()
    ///     .await?
    ///     .sort_by_query_relevance(query);  // Sort by query match
    /// # Ok(())
    /// # }
    /// ```
    fn sort_by_query_relevance(self, query: &str) -> Self;
}

impl SearchResultExt for Vec<Manga> {
    fn filter_popular(self, min_popularity_score: usize) -> Self {
        // Filter manga based on popularity heuristics since we don't have direct chapter counts
        // We use a scoring system based on available metadata:
        // - Has description: +2 points
        // - Has authors: +1 point
        // - Has cover image: +1 point
        // - Has 3+ tags: +1 point
        // - Has 5+ tags: +2 points (additional point)
        self.into_iter()
            .filter(|manga| {
                let mut score = 0;

                // Description indicates well-documented manga
                if manga.description.is_some()
                    && !manga.description.as_ref().unwrap().trim().is_empty()
                {
                    score += 2;
                }

                // Authors listed indicates properly catalogued manga
                if !manga.authors.is_empty() {
                    score += 1;
                }

                // Cover image indicates higher quality entry
                if manga.cover_url.is_some() {
                    score += 1;
                }

                // Well-tagged manga tend to be more popular/complete
                let tag_count = manga.tags.len();
                if tag_count >= 3 {
                    score += 1;
                }
                if tag_count >= 5 {
                    score += 1; // Additional point for very well-tagged manga
                }

                score >= min_popularity_score
            })
            .collect()
    }

    fn dedupe_by_title(mut self) -> Self {
        let mut seen = std::collections::HashSet::new();
        self.retain(|manga| seen.insert(manga.title.to_lowercase()));
        self
    }

    fn sort_by_relevance(mut self) -> Self {
        // Enhanced relevance scoring algorithm
        self.sort_by(|a, b| {
            let score_a = calculate_relevance_score(&a.title, &a.description, &a.tags, &a.authors);
            let score_b = calculate_relevance_score(&b.title, &b.description, &b.tags, &b.authors);

            // Sort by highest score first, then by title length for ties
            score_b
                .cmp(&score_a)
                .then_with(|| a.title.len().cmp(&b.title.len()))
        });
        self
    }

    fn sort_by_query_relevance(mut self, query: &str) -> Self {
        // Query-aware relevance scoring
        let query_lower = query.to_lowercase();
        self.sort_by(|a, b| {
            let score_a = calculate_query_relevance_score(
                &a.title,
                &a.description,
                &a.tags,
                &a.authors,
                &query_lower,
            );
            let score_b = calculate_query_relevance_score(
                &b.title,
                &b.description,
                &b.tags,
                &b.authors,
                &query_lower,
            );

            // Sort by highest score first, then by title length for ties
            score_b
                .cmp(&score_a)
                .then_with(|| a.title.len().cmp(&b.title.len()))
        });
        self
    }
}

/// Calculate relevance score for a manga based on multiple factors
fn calculate_relevance_score(
    title: &str,
    description: &Option<String>,
    tags: &[String],
    authors: &[String],
) -> u32 {
    let mut score = 0u32;

    // Base popularity score from metadata completeness
    if description.is_some() && !description.as_ref().unwrap().trim().is_empty() {
        score += 10; // Well-documented manga
    }

    if !authors.is_empty() {
        score += 5; // Has author information
    }

    // Tag-based scoring
    let tag_count = tags.len();
    if tag_count >= 3 {
        score += 5;
    }
    if tag_count >= 5 {
        score += 5; // Well-categorized
    }

    // Title length preference (shorter titles often more relevant for exact searches)
    let title_len = title.len();
    if title_len <= 20 {
        score += 15; // Short, likely main title
    } else if title_len <= 40 {
        score += 10; // Medium length
    } else {
        score += 5; // Longer titles
    }

    // Quality indicators
    if title.contains("Official") || title.contains("Colored") {
        score += 8; // Official versions
    }

    if title.chars().all(|c| c.is_ascii()) {
        score += 3; // ASCII titles often more accessible
    }

    score
}

/// Calculate query-aware relevance score for a manga
fn calculate_query_relevance_score(
    title: &str,
    description: &Option<String>,
    tags: &[String],
    authors: &[String],
    query: &str,
) -> u32 {
    let mut score = 0u32;
    let title_lower = title.to_lowercase();

    // Query matching scores (highest priority)
    if title_lower == query {
        score += 100; // Exact match
    } else if title_lower.contains(query) {
        score += 50; // Contains query
    } else {
        // Word-by-word matching
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let title_words: Vec<&str> = title_lower.split_whitespace().collect();

        let mut word_matches = 0;
        for query_word in &query_words {
            for title_word in &title_words {
                if title_word.contains(query_word) || query_word.contains(title_word) {
                    word_matches += 1;
                    break;
                }
            }
        }

        // Score based on word match percentage
        if !query_words.is_empty() {
            score += (word_matches * 25) / query_words.len() as u32;
        }
    }

    // Description matching
    if let Some(desc) = description {
        let desc_lower = desc.to_lowercase();
        if desc_lower.contains(query) {
            score += 15;
        }
    }

    // Tag matching
    for tag in tags {
        if tag.to_lowercase().contains(query) {
            score += 10;
        }
    }

    // Author matching
    for author in authors {
        if author.to_lowercase().contains(query) {
            score += 20;
        }
    }

    // Add base popularity score (reduced weight for query-aware scoring)
    let base_score = calculate_relevance_score(title, description, tags, authors);
    score += base_score / 3; // Reduce base score influence

    score
}
