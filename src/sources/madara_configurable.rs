use crate::{
    error::Result,
    net::{self, HttpClient},
    source::Source,
    types::{Chapter, Manga, SearchParams},
};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MadaraSelectors {
    pub manga_item: String,
    pub chapter_links: String,
    pub chapter_titles: String,
    pub chapter_pages: String,
    pub cover_image: String,
}

#[derive(Debug, Clone)]
pub struct MadaraConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub headers: Option<HashMap<String, String>>,
    pub selectors: MadaraSelectors,
}

pub struct ConfigurableMadaraSource {
    config: MadaraConfig,
    client: HttpClient,
}

impl ConfigurableMadaraSource {
    pub fn new(config: MadaraConfig) -> Self {
        let mut client_builder = HttpClient::new(config.id)
            .with_rate_limit(2000)
            .with_max_retries(3);

        // Apply custom headers if provided
        if let Some(headers) = &config.headers {
            for (key, value) in headers {
                client_builder = client_builder.with_header(key, value);
            }
        }

        Self {
            config,
            client: client_builder,
        }
    }

    // Helper function to resolve relative URLs to absolute ones
    fn full_url(&self, path: &str) -> String {
        let trimmed_base = self.config.base_url.trim_end_matches('/');
        let trimmed_path = path.trim_start_matches('/');
        format!("{}/{}", trimmed_base, trimmed_path)
    }
}

#[async_trait]
impl Source for ConfigurableMadaraSource {
    fn id(&self) -> &'static str {
        &self.config.id
    }

    fn name(&self) -> &'static str {
        &self.config.name
    }

    fn base_url(&self) -> &str {
        &self.config.base_url
    }

    async fn search(&self, params: SearchParams) -> Result<Vec<Manga>> {
        let url = format!(
            "{}/?s={}&post_type=wp-manga",
            self.config.base_url,
            urlencoding::encode(&params.query)
        );

        let html_str = self.client.get_text(&url).await?;

        let html = net::html::parse(&html_str);
        let links = net::html::select_all_attr(&html, &self.config.selectors.manga_item, "href");
        let titles = net::html::select_all_text(&html, &self.config.selectors.manga_item);
        let cover_images =
            net::html::select_all_attr(&html, &self.config.selectors.cover_image, "src");

        let mut manga = Vec::new();

        for ((href, title), cover_img) in links.into_iter().zip(titles).zip(
            cover_images
                .into_iter()
                .chain(std::iter::repeat(String::new())),
        ) {
            if title.trim().is_empty() || href.trim().is_empty() {
                continue;
            }

            // Extract manga ID from URL path
            let id = if href.contains("/kissmanga/") {
                if let Some(id_part) = href.split("/kissmanga/").nth(1) {
                    id_part.trim_end_matches('/').to_string()
                } else {
                    continue;
                }
            } else {
                if let Some(id_part) = href.split('/').filter(|s| !s.is_empty()).last() {
                    id_part.to_string()
                } else {
                    continue;
                }
            };

            let cover_url = if !cover_img.trim().is_empty() {
                if cover_img.starts_with("http") {
                    Some(cover_img)
                } else {
                    Some(self.full_url(&cover_img))
                }
            } else {
                None
            };

            manga.push(Manga {
                id,
                title: title.trim().to_string(),
                cover_url,
                authors: vec![],
                description: None,
                tags: vec![],
                source_id: self.id().to_string(),
                #[cfg(feature = "sqlx")]
                created_at: None,
                #[cfg(feature = "sqlx")]
                updated_at: None,
            });
        }

        // Apply limit if specified
        let manga = if let Some(limit) = params.limit {
            manga.into_iter().take(limit).collect()
        } else {
            manga
        };

        Ok(manga)
    }

    async fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>> {
        let url = if manga_id.starts_with("http") {
            manga_id.to_string()
        } else {
            self.full_url(&format!("kissmanga/{}", manga_id))
        };

        let html_str = self.client.get_text(&url).await?;
        let html = net::html::parse(&html_str);

        // Try to get chapter links and titles
        let links = net::html::select_all_attr(&html, &self.config.selectors.chapter_links, "href");
        let titles = net::html::select_all_text(&html, &self.config.selectors.chapter_titles);

        let chapters: Vec<Chapter> = links
            .into_iter()
            .zip(titles)
            .enumerate()
            .filter_map(|(i, (href, title))| {
                if href.trim().is_empty() {
                    return None;
                }

                // Extract chapter ID from URL
                let id = if href.contains("/kissmanga/") {
                    href.split("/kissmanga/")
                        .nth(1)?
                        .trim_end_matches('/')
                        .to_string()
                } else {
                    href.split('/')
                        .filter(|s| !s.is_empty())
                        .last()?
                        .to_string()
                };

                Some(Chapter {
                    id,
                    number: (i + 1) as f64,
                    title: title.trim().to_string(),
                    pages: vec![],
                    manga_id: manga_id.to_string(),
                    source_id: self.id().to_string(),
                    #[cfg(feature = "sqlx")]
                    created_at: None,
                })
            })
            .collect();

        Ok(chapters)
    }

    async fn get_pages(&self, chapter_id: &str) -> Result<Vec<String>> {
        let url = if chapter_id.starts_with("http") {
            chapter_id.to_string()
        } else {
            self.full_url(&format!("kissmanga/{}", chapter_id))
        };

        let html_str = self.client.get_text(&url).await?;
        let html = net::html::parse(&html_str);

        // Try to get page images
        let pages = net::html::select_all_attr(&html, &self.config.selectors.chapter_pages, "src");

        if pages.is_empty() {
            return Err(crate::Error::not_found("No pages found"));
        }

        // Filter out small images (likely ads or icons)
        let pages: Vec<String> = pages
            .into_iter()
            .filter(|url| {
                // Filter out tiny images and common ad patterns
                !url.contains("loading") &&
                !url.contains("advertisement") &&
                !url.contains("banner") &&
                !url.contains("favicon") &&
                !url.ends_with(".gif") &&
                url.len() > 10 &&
                // Make sure it's a valid image URL
                (url.contains(".jpg") || url.contains(".png") || url.contains(".jpeg") || url.contains(".webp"))
            })
            .collect();

        if pages.is_empty() {
            return Err(crate::Error::not_found(
                "No valid pages found after filtering",
            ));
        }

        Ok(pages)
    }
}
