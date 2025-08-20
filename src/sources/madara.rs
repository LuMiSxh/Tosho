use crate::{
    error::Result,
    net::{self, HttpClient},
    source::Source,
    types::{Chapter, Manga, SearchParams},
};
use async_trait::async_trait;

/// Source implementation for Madara WordPress theme sites
pub struct MadaraSource {
    base_url: String,
    client: HttpClient,
}

impl MadaraSource {
    /// Create a new Madara source
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        Self {
            client: HttpClient::new("madara").with_rate_limit(500),
            base_url,
        }
    }
}

#[async_trait]
impl Source for MadaraSource {
    fn id(&self) -> &'static str {
        "madara"
    }

    fn name(&self) -> &'static str {
        "Madara Site"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn search(&self, params: SearchParams) -> Result<Vec<Manga>> {
        let url = format!(
            "{}/?s={}&post_type=wp-manga",
            self.base_url,
            urlencoding::encode(&params.query)
        );

        let html_str = self.client.get_text(&url).await?;
        let html = net::html::parse(&html_str);

        // Parse manga items in parallel
        let manga = net::html::parse_manga_items(&html, ".post-title a", |element| {
            let title = element.text().collect::<String>().trim().to_string();
            let href = element.value().attr("href")?;
            let id = href
                .split('/')
                .filter(|s| !s.is_empty())
                .last()?
                .to_string();

            Some(Manga {
                id,
                title,
                cover_url: None,
                authors: vec![],
                description: None,
                tags: vec![],
                source_id: self.id().to_string(),
            })
        });

        // Apply limit if specified
        let manga = if let Some(limit) = params.limit {
            manga.into_iter().take(limit).collect()
        } else {
            manga
        };

        Ok(manga)
    }

    async fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>> {
        let url = format!("{}/manga/{}", self.base_url, manga_id);
        let html_str = self.client.get_text(&url).await?;
        let html = net::html::parse(&html_str);

        let chapter_links = net::html::select_all_attr(&html, "li.wp-manga-chapter a", "href");
        let chapter_titles = net::html::select_all_text(&html, "li.wp-manga-chapter a");

        let chapters: Vec<Chapter> = chapter_links
            .into_iter()
            .zip(chapter_titles)
            .enumerate()
            .map(|(i, (href, title))| {
                let id = href
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .last()
                    .unwrap_or("unknown")
                    .to_string();

                Chapter {
                    id,
                    number: (i + 1) as f64,
                    title,
                    pages: vec![],
                    manga_id: manga_id.to_string(),
                    source_id: self.id().to_string(),
                }
            })
            .collect();

        Ok(chapters)
    }

    async fn get_pages(&self, chapter_id: &str) -> Result<Vec<String>> {
        let url = format!("{}/manga/{}", self.base_url, chapter_id);
        let html_str = self.client.get_text(&url).await?;
        let html = net::html::parse(&html_str);

        let pages = net::html::select_all_attr(&html, ".page-break img", "src");

        if pages.is_empty() {
            return Err(crate::Error::not_found("No pages found"));
        }

        Ok(pages)
    }
}
