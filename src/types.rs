//! Core data types for manga, chapters, and search parameters.
//!
//! This module defines the fundamental data structures used throughout Tosho:
//!
//! - [`Manga`] - Represents a manga/comic series with metadata
//! - [`Chapter`] - Represents a single chapter with pages
//! - [`SearchParams`] - Parameters for searching manga
//! - [`SortOrder`] - Sorting options for search results
//!
//! # Examples
//!
//! ```rust
//! use tosho::types::*;
//!
//! let manga = Manga {
//!     id: "one-piece".to_string(),
//!     title: "One Piece".to_string(),
//!     authors: vec!["Oda Eiichiro".to_string()],
//!     source_id: "mangadex".to_string(),
//!     cover_url: Some("https://example.com/cover.jpg".to_string()),
//!     description: Some("Epic pirate adventure".to_string()),
//!     tags: vec!["Action".to_string(), "Adventure".to_string()],
//!     #[cfg(feature = "sqlx")]
//!     created_at: None,
//!     #[cfg(feature = "sqlx")]
//!     updated_at: None,
//! };
//! ```

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[cfg(feature = "sqlx")]
use chrono::NaiveDateTime;
#[cfg(feature = "sqlx")]
use sqlx::FromRow;

/// Represents a manga/comic series with all its metadata.
///
/// This is the core data structure for manga information across all sources.
/// Each manga has a unique ID within its source and contains essential metadata
/// like title, authors, description, and tags.
///
/// # Fields
///
/// * `id` - Unique identifier within the source (used for fetching chapters)
/// * `title` - The main title of the manga
/// * `cover_url` - Optional URL to the cover image
/// * `authors` - List of author names
/// * `description` - Optional plot summary or description
/// * `tags` - Genre tags and categories
/// * `source_id` - Identifier of the source this manga came from
///
/// # Examples
///
/// ```rust
/// use tosho::types::Manga;
///
/// let manga = Manga {
///     id: "123".to_string(),
///     title: "One Piece".to_string(),
///     authors: vec!["Oda Eiichiro".to_string()],
///     source_id: "mangadex".to_string(),
///     cover_url: Some("https://example.com/cover.jpg".to_string()),
///     description: Some("A story about pirates".to_string()),
///     tags: vec!["Action".to_string(), "Adventure".to_string()],
///     #[cfg(feature = "sqlx")]
///     created_at: None,
///     #[cfg(feature = "sqlx")]
///     updated_at: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(FromRow))]
#[cfg_attr(feature = "sqlx", sqlx(rename_all = "snake_case"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Manga {
    /// Unique identifier within the source
    pub id: String,

    /// Main title
    pub title: String,

    /// Cover image URL
    pub cover_url: Option<String>,

    /// List of authors
    #[cfg_attr(feature = "sqlx", sqlx(skip))]
    #[serde(default)]
    pub authors: Vec<String>,

    /// Description/summary
    pub description: Option<String>,

    /// Tags/genres
    #[cfg_attr(feature = "sqlx", sqlx(skip))]
    #[serde(default)]
    pub tags: Vec<String>,

    /// Source identifier this manga came from
    pub source_id: String,

    /// Creation timestamp (for database users)
    #[cfg(feature = "sqlx")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<NaiveDateTime>,

    /// Last update timestamp (for database users)
    #[cfg(feature = "sqlx")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<NaiveDateTime>,
}

/// Represents a single chapter of a manga.
///
/// Chapters contain the actual readable content in the form of page URLs.
/// The chapter number can be decimal to support special chapters like "Chapter 5.5".
///
/// # Fields
///
/// * `id` - Unique identifier within the source
/// * `number` - Chapter number (supports decimals for special chapters)
/// * `title` - Chapter title or name
/// * `pages` - URLs to individual pages of the chapter
/// * `manga_id` - ID of the manga this chapter belongs to
/// * `source_id` - Identifier of the source
///
/// # Examples
///
/// ```rust
/// use tosho::types::Chapter;
///
/// let chapter = Chapter {
///     id: "ch1".to_string(),
///     number: 1.0,
///     title: "Romance Dawn".to_string(),
///     pages: vec![
///         "https://example.com/page1.jpg".to_string(),
///         "https://example.com/page2.jpg".to_string(),
///     ],
///     manga_id: "one-piece".to_string(),
///     source_id: "mangadex".to_string(),
///     #[cfg(feature = "sqlx")]
///     created_at: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(FromRow))]
#[cfg_attr(feature = "sqlx", sqlx(rename_all = "snake_case"))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Chapter {
    /// Unique identifier within the source
    pub id: String,

    /// Chapter number (can be decimal for .5 chapters)
    pub number: f64,

    /// Chapter title
    pub title: String,

    /// Page URLs for this chapter
    #[cfg_attr(feature = "sqlx", sqlx(skip))]
    #[serde(default)]
    pub pages: Vec<String>,

    /// Associated manga ID
    pub manga_id: String,

    /// Source identifier
    pub source_id: String,

    /// Creation timestamp (for database users)
    #[cfg(feature = "sqlx")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<NaiveDateTime>,
}

/// Search parameters for querying manga across sources.
///
/// This struct contains all the parameters that can be used to search for manga.
/// It uses the builder pattern (via `derive_builder`) to provide a fluent API
/// for constructing search queries.
///
/// # Builder Usage
///
/// The `derive_builder` crate automatically generates a `SearchParamsBuilder`
/// that can be used for constructing search parameters:
///
/// ```rust
/// use tosho::types::{SearchParamsBuilder, SortOrder};
///
/// let params = SearchParamsBuilder::default()
///     .query("one piece".to_string())
///     .limit(Some(20))
///     .sort_by(Some(SortOrder::UpdatedAt))
///     .build()
///     .unwrap();
/// ```
///
/// # Fields
///
/// * `query` - The search query string
/// * `limit` - Maximum number of results to return
/// * `offset` - Offset for pagination
/// * `include_tags` - Only include manga with these tags
/// * `exclude_tags` - Exclude manga with these tags
/// * `sort_by` - How to sort the results
#[derive(Debug, Clone, Default, Builder)]
#[builder(setter(into))]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SearchParams {
    pub query: String,
    #[builder(default)]
    pub limit: Option<usize>,
    #[builder(default)]
    pub offset: Option<usize>,
    #[builder(default)]
    pub include_tags: Vec<String>,
    #[builder(default)]
    pub exclude_tags: Vec<String>,
    #[builder(default)]
    pub sort_by: Option<SortOrder>,
}

/// Defines how search results should be sorted.
///
/// Different sources may support different sorting options. Not all sources
/// are guaranteed to support all sorting methods.
///
/// # Variants
///
/// * `Relevance` - Sort by search relevance (default for most searches)
/// * `UpdatedAt` - Sort by when the manga was last updated (newest first)
/// * `CreatedAt` - Sort by when the manga was first published
/// * `Title` - Sort alphabetically by title
///
/// # Examples
///
/// ```rust
/// use tosho::types::SortOrder;
///
/// // Most recent updates first
/// let sort = SortOrder::UpdatedAt;
///
/// // Alphabetical order
/// let sort = SortOrder::Title;
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SortOrder {
    Relevance,
    UpdatedAt,
    CreatedAt,
    Title,
}

impl From<String> for SearchParams {
    /// Creates search parameters from a query string.
    ///
    /// This is a convenience method for creating basic search parameters
    /// with just a query and default values for all other fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::types::SearchParams;
    ///
    /// let params: SearchParams = "one piece".to_string().into();
    /// assert_eq!(params.query, "one piece");
    /// assert_eq!(params.limit, None);
    /// ```
    fn from(query: String) -> Self {
        SearchParams {
            query,
            ..Default::default()
        }
    }
}

impl From<&str> for SearchParams {
    /// Creates search parameters from a string slice.
    ///
    /// This is a convenience method for creating basic search parameters
    /// with just a query and default values for all other fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tosho::types::SearchParams;
    ///
    /// let params: SearchParams = "naruto".into();
    /// assert_eq!(params.query, "naruto");
    /// assert_eq!(params.limit, None);
    /// ```
    fn from(query: &str) -> Self {
        SearchParams {
            query: query.to_string(),
            ..Default::default()
        }
    }
}
