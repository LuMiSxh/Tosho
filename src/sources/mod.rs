//! Manga source implementations with conditional compilation support.
//!
//! This module contains all manga source implementations, with individual sources
//! protected by feature flags to allow for minimal builds that only include the
//! sources you need.
//!
//! # Feature Flags
//!
//! Each source is behind its own feature flag:
//! - `source-mangadex` - Enables the MangaDex source
//! - `source-kissmanga` - Enables the KissManga source
//! - `all-sources` - Enables all sources (default)
//!
//! # Examples
//!
//! Build with only MangaDex support:
//! ```bash
//! cargo build --no-default-features --features source-mangadex
//! ```
//!
//! Build with only KissManga support:
//! ```bash
//! cargo build --no-default-features --features source-kissmanga
//! ```
//!
//! Build with specific sources:
//! ```bash
//! cargo build --no-default-features --features "source-mangadex,source-kissmanga"
//! ```
//!
//! # Available Sources
//!
//! - [`madara_configurable`] - Base implementation for Madara theme sites (always available)
//! - [`MangaDexSource`] - MangaDex.org source (requires `source-mangadex` feature)
//! - [`KissMangaSource`] - KissManga.in source (requires `source-kissmanga` feature)

// Always include the configurable madara base
pub mod madara_configurable;

// Individual sources behind feature flags
#[cfg(feature = "source-mangadex")]
pub mod mangadex;

#[cfg(feature = "source-kissmanga")]
pub mod kissmanga;

// Re-export sources only when their features are enabled
#[cfg(feature = "source-mangadex")]
pub use mangadex::MangaDexSource;

#[cfg(feature = "source-kissmanga")]
pub use kissmanga::KissMangaSource;
