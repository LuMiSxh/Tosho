//! # Tosho - High-performance manga aggregation and download library
//!
//! Tosho is a high-performance, async manga aggregation library that provides a unified interface
//! for searching, retrieving, and downloading manga content from multiple sources. It features a
//! fluent search API, integrated download system, intelligent rate limiting, parallel processing,
//! and robust error handling.
//!
//! ## Features
//!
//! - **Unified Search API**: Search across multiple manga sources with a single interface
//! - **Integrated Downloads**: Download chapters directly through source implementations
//! - **Fluent Builder Pattern**: Chain search parameters and execution strategies elegantly
//! - **Async/Await Support**: Built on tokio for high-performance concurrent operations
//! - **Rate Limiting**: Per-source rate limiting to respect website policies
//! - **Parallel Processing**: Uses rayon for CPU-intensive parsing operations
//! - **Robust Error Handling**: Comprehensive error types with detailed context
//! - **Result Processing**: Built-in deduplication, sorting, and filtering capabilities
//!
//! ## Quick Start
//!
//! ### Searching for Manga
//!
//! ```rust
//! use tosho::prelude::*;
//! use tosho::error::Result;
//! #[cfg(feature = "source-mangadex")]
//! use tosho::sources::MangaDexSource;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut sources = Sources::new();
//!     #[cfg(feature = "source-mangadex")]
//!     sources.add(MangaDexSource::new());
//!
//!     // Simple search with flattened results
//!     let results = sources
//!         .search("one piece")
//!         .limit(20)
//!         .sort_by(SortOrder::UpdatedAt)
//!         .flatten()
//!         .await?;
//!
//!     println!("Found {} results", results.len());
//!     Ok(())
//! }
//! ```
//!
//! ### Downloading Chapters
//!
//! ```rust,no_run
//! use tosho::prelude::*;
//! use tosho::error::Result;
//! #[cfg(feature = "source-mangadex")]
//! use tosho::sources::MangaDexSource;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     #[cfg(feature = "source-mangadex")]
//!     let source = MangaDexSource::new();
//!
//!     // Search for manga
//!     let manga_list = source.search("oneshot".into()).await?;
//!     let manga = &manga_list[0];
//!
//!     // Get chapters
//!     let chapters = source.get_chapters(&manga.id).await?;
//!     let chapter = &chapters[0];
//!
//!     // Download chapter
//!     let download_dir = PathBuf::from("./downloads");
//!     let chapter_path = source.download_chapter(&chapter.id, &download_dir).await?;
//!
//!     println!("Downloaded to: {}", chapter_path.display());
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The library is organized into several key modules:
//!
//! - [`source`]: Core trait and collection for manga sources
//! - [`search`]: Fluent search builder and result processing
//! - [`types`]: Core data structures for manga, chapters, and search parameters
//! - [`net`]: HTTP client, rate limiting, and parsing utilities
//! - [`error`]: Comprehensive error handling
//!
//! ## Search Strategies
//!
//! Tosho supports multiple search execution strategies:
//!
//! ```rust
//! # use tosho::prelude::*;
//! # use tosho::error::Result;
//!
//! # async fn example() -> Result<()> {
//! # let sources = Sources::new();
//! // Flatten results from all sources
//! let all_results = sources.search("naruto").flatten().await?;
//!
//! // Group results by source (useful for debugging)
//! let grouped = sources.search("naruto").group().await;
//!
//! // Search specific source only
//! let specific = sources.search("naruto").from_source("mangadex").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Result Processing
//!
//! Built-in result processing methods help you work with search results:
//!
//! ```rust
//! # use tosho::prelude::*;
//! # use tosho::error::Result;
//!
//! # async fn example() -> Result<()> {
//! # let sources = Sources::new();
//! let processed = sources
//!     .search("popular manga")
//!     .limit(50)
//!     .flatten()
//!     .await?
//!     .dedupe_by_title()        // Remove duplicates
//!     .sort_by_relevance();     // Sort by relevance
//! # Ok(())
//! # }
//! ```

pub mod download;
pub mod error;
pub mod net;
pub mod search;
pub mod source;
pub mod sources;
pub mod types;

#[cfg(feature = "tui")]
pub mod tui;

/// Prelude module for convenient imports.
///
/// This module re-exports the most commonly used types and traits, allowing you to
/// import everything you need with a single `use tosho::prelude::*;` statement.
///
/// # Example
///
/// ```rust
/// use tosho::prelude::*;
///
/// // Now you have access to:
/// // - Sources, Source trait
/// // - SearchBuilder, SearchResultExt
/// // - Manga, Chapter, SearchParams, SortOrder
/// // - Download utilities
/// ```
pub mod prelude {
    pub use crate::{
        download::{download_file, extract_extension, sanitize_filename},
        search::{SearchBuilder, SearchResultExt},
        source::{Source, Sources},
        types::{Chapter, Manga, SearchParams, SortOrder},
    };

    #[cfg(feature = "tui")]
    pub use crate::tui::*;
}

// Re-export main types at crate root for direct access
pub use download::{download_file, extract_extension, sanitize_filename};
pub use error::{Error, Result};
pub use search::{SearchBuilder, SearchResultExt};
pub use source::{Source, Sources};
pub use types::{Chapter, Manga, SearchParams, SortOrder};
