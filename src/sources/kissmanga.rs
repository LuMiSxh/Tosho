use crate::{
    error::Result,
    source::Source,
    types::{Chapter, Manga, SearchParams},
};
use async_trait::async_trait;
use std::collections::HashMap;

use super::madara_configurable::{ConfigurableMadaraSource, MadaraConfig, MadaraSelectors};

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
