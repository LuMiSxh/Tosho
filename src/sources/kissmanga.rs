use crate::{
    error::Result,
    source::Source,
    types::{Chapter, Manga, SearchParams},
};
use async_trait::async_trait;
use std::collections::HashMap;

use super::madara_configurable::{ConfigurableMadaraSource, MadaraConfig, MadaraSelectors};

/// KissManga source implementation for accessing manga from KissManga.in.
///
/// This source provides access to KissManga, a popular manga reading website
/// that uses the Madara WordPress theme. It leverages the configurable Madara
/// base implementation with KissManga-specific settings and selectors.
///
/// # Features
///
/// - Manga search with query support
/// - Chapter listing and metadata retrieval
/// - Image downloads from KissManga's hosting
/// - Built-in rate limiting and retry logic
/// - Custom headers for improved compatibility
///
/// # Implementation Details
///
/// This source is built on top of the [`ConfigurableMadaraSource`] which handles
/// the common Madara theme patterns. KissManga-specific configuration includes:
/// - Custom User-Agent and headers for better site compatibility
/// - Optimized CSS selectors for KissManga's layout
/// - Proper referrer handling
///
/// # Examples
///
/// ```rust
/// use tosho::sources::KissMangaSource;
/// use tosho::prelude::*;
///
/// # async fn example() -> tosho::Result<()> {
/// let source = KissMangaSource::new();
///
/// // Search for manga
/// let results = source.search(SearchParams {
///     query: "naruto".to_string(),
///     limit: Some(5),
///     ..Default::default()
/// }).await?;
///
/// // Get chapters for a manga
/// if let Some(manga) = results.first() {
///     let chapters = source.get_chapters(&manga.id).await?;
/// }
/// # Ok(())
/// # }
/// ```
pub struct KissMangaSource {
    inner: ConfigurableMadaraSource,
}

impl KissMangaSource {
    pub fn new() -> Self {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".into());
        headers.insert(
            "Accept".into(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".into(),
        );
        headers.insert("Accept-Language".into(), "en-US,en;q=0.9".into());
        headers.insert("Cache-Control".into(), "no-cache".into());
        headers.insert("Referer".into(), "https://kissmanga.in/".into());

        let config = MadaraConfig {
            id: "kmg",
            name: "KissManga",
            base_url: "https://kissmanga.in",
            headers: Some(headers),
            selectors: MadaraSelectors {
                manga_item: ".c-tabs-item__content .post-title h3 a".to_string(),
                chapter_links: ".wp-manga-chapter a".to_string(),
                chapter_titles: ".wp-manga-chapter a".to_string(),
                chapter_pages: ".reading-content .page-break img".to_string(),
            },
        };
        Self {
            inner: ConfigurableMadaraSource::new(config),
        }
    }
}

// Delegate all Source trait methods to the inner ConfigurableMadaraSource
#[async_trait]
impl Source for KissMangaSource {
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn base_url(&self) -> &str {
        self.inner.base_url()
    }

    async fn search(&self, params: SearchParams) -> Result<Vec<Manga>> {
        self.inner.search(params).await
    }

    async fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>> {
        self.inner.get_chapters(manga_id).await
    }

    async fn get_pages(&self, chapter_id: &str) -> Result<Vec<String>> {
        self.inner.get_pages(chapter_id).await
    }
}
