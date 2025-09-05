use crate::{
    error::Result,
    net::HttpClient,
    source::Source,
    types::{Chapter, Manga, SearchParams, SortOrder},
};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

/// MangaDex API search response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MangaDexSearchResponse {
    data: Vec<MangaDexMangaData>,
    total: u32,
    limit: u32,
    offset: u32,
}

/// MangaDex API manga response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MangaDexMangaResponse {
    data: MangaDexMangaData,
}

/// MangaDex manga data structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MangaDexMangaData {
    id: String,
    #[serde(rename = "type")]
    data_type: String,
    attributes: MangaDexMangaAttributes,
    relationships: Vec<MangaDexRelationship>,
}

/// MangaDex manga attributes
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MangaDexMangaAttributes {
    title: HashMap<String, String>,
    #[serde(rename = "altTitles")]
    alt_titles: Vec<HashMap<String, String>>,
    description: HashMap<String, String>,
    status: String,
    tags: Vec<MangaDexTag>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<String>,
}

/// MangaDex tag structure
#[derive(Debug, Deserialize)]
struct MangaDexTag {
    attributes: MangaDexTagAttributes,
}

/// MangaDex tag attributes
#[derive(Debug, Deserialize)]
struct MangaDexTagAttributes {
    name: HashMap<String, String>,
}

/// MangaDex relationship structure
#[derive(Debug, Deserialize)]
struct MangaDexRelationship {
    #[serde(rename = "type")]
    rel_type: String,
    attributes: Option<MangaDexRelationshipAttributes>,
}

/// MangaDex relationship attributes
#[derive(Debug, Deserialize)]
struct MangaDexRelationshipAttributes {
    name: Option<String>,
    #[serde(rename = "fileName")]
    file_name: Option<String>,
}

/// MangaDex chapter list response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MangaDexChapterListResponse {
    data: Vec<MangaDexChapterData>,
    total: u32,
    limit: u32,
    offset: u32,
}

/// MangaDex single chapter response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MangaDexChapterResponse {
    data: MangaDexChapterData,
}

/// MangaDex chapter data structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MangaDexChapterData {
    id: String,
    attributes: MangaDexChapterAttributes,
    relationships: Vec<MangaDexRelationship>,
}

/// MangaDex chapter attributes
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MangaDexChapterAttributes {
    title: Option<String>,
    chapter: Option<String>,
    volume: Option<String>,
    #[serde(rename = "publishAt")]
    publish_at: Option<String>,
    #[serde(rename = "translatedLanguage")]
    translated_language: String,
}

/// MangaDex pages response (at-home server)
#[derive(Debug, Deserialize)]
struct MangaDexPagesResponse {
    #[serde(rename = "baseUrl")]
    base_url: String,
    chapter: MangaDexChapterPages,
}

/// MangaDex chapter pages structure
#[derive(Debug, Deserialize)]
struct MangaDexChapterPages {
    hash: String,
    data: Vec<String>,
    #[serde(rename = "dataSaver")]
    data_saver: Vec<String>,
}

/// MangaDex source implementation for accessing manga from MangaDex.org.
///
/// This source provides access to the MangaDex API, supporting search functionality,
/// chapter retrieval, and image downloads. MangaDex is one of the largest open-source
/// manga platforms with extensive multilingual support.
///
/// # Features
///
/// - Full-text manga search with filtering support
/// - Multi-language title support (prioritizes English, then Japanese)
/// - Chapter listing and metadata retrieval
/// - High-quality image downloads with fallback support
/// - Built-in rate limiting (1 request per second)
/// - Automatic retry on failed requests
///
/// # Rate Limiting
///
/// This implementation respects MangaDex's API rate limits by enforcing
/// a 1-second delay between requests. The API allows up to 5 requests
/// per second, but we use a conservative limit to avoid issues.
///
/// # Examples
///
/// ```rust
/// use tosho::sources::MangaDexSource;
/// use tosho::prelude::*;
///
/// # async fn example() -> tosho::Result<()> {
/// let source = MangaDexSource::new();
///
/// // Search for manga
/// let results = source.search(SearchParams {
///     query: "one piece".to_string(),
///     limit: Some(10),
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
pub struct MangaDexSource {
    client: HttpClient,
    api_base: String,
}

impl MangaDexSource {
    /// Create a new MangaDex source
    pub fn new() -> Self {
        Self {
            client: HttpClient::new("mangadex")
                .with_rate_limit(1000) // 1 second between requests (5 req/sec limit)
                .with_max_retries(3),
            api_base: "https://api.mangadex.org".to_string(),
        }
    }

    /// Extract the best title from a multi-language title map
    fn extract_best_title(title_map: &HashMap<String, String>) -> String {
        // Priority order for title languages
        let priority_langs = ["en", "en-us", "ja", "ja-ro"];

        for lang in &priority_langs {
            if let Some(title) = title_map.get(*lang) {
                if !title.trim().is_empty() {
                    return title.trim().to_string();
                }
            }
        }

        // If no priority language found, take the first available
        title_map
            .values()
            .find(|title| !title.trim().is_empty())
            .map(|title| title.trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string())
    }

    /// Format search query parameters
    fn format_search_query(&self, query: &str, params: &SearchParams) -> String {
        let mut query_parts = vec![
            format!("title={}", urlencoding::encode(query)),
            format!("limit={}", params.limit.unwrap_or(20)),
            "includes[]=cover_art".to_string(),
        ];

        // Add order parameters
        match params.sort_by {
            Some(SortOrder::UpdatedAt) => {
                query_parts.push("order[updatedAt]=desc".to_string());
            }
            Some(SortOrder::CreatedAt) => {
                query_parts.push("order[createdAt]=desc".to_string());
            }
            Some(SortOrder::Title) => {
                query_parts.push("order[title]=asc".to_string());
            }
            _ => {
                query_parts.push("order[relevance]=desc".to_string());
            }
        }

        // Add content ratings
        let content_ratings = ["safe", "suggestive", "erotica", "pornographic"];
        for rating in &content_ratings {
            query_parts.push(format!("contentRating[]={}", rating));
        }

        // Add offset if specified
        if let Some(offset) = params.offset {
            query_parts.push(format!("offset={}", offset));
        }

        query_parts.join("&")
    }

    /// Format chapter query parameters
    fn format_chapters_query(&self, offset: u32, limit: u32) -> String {
        let params = vec![
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
            ("order[volume]", "asc".to_string()),
            ("order[chapter]", "asc".to_string()),
            ("translatedLanguage[]", "en".to_string()),
            ("contentRating[]", "safe".to_string()),
            ("contentRating[]", "suggestive".to_string()),
            ("contentRating[]", "erotica".to_string()),
            ("contentRating[]", "pornographic".to_string()),
        ];

        params
            .iter()
            .map(|(key, value)| format!("{}={}", key, urlencoding::encode(value)))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Fetch all chapters for a manga (handles pagination)
    async fn fetch_all_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>> {
        let mut all_chapters = Vec::new();
        let mut offset = 0;
        const LIMIT: u32 = 500; // Max limit for this endpoint

        loop {
            let query_params = self.format_chapters_query(offset, LIMIT);
            let url = format!("{}/manga/{}/feed?{}", self.api_base, manga_id, query_params);

            let response: MangaDexChapterListResponse = self.client.get_json(&url).await?;

            // Map chapters
            for chapter_data in response.data {
                if let Some(chapter) = self.map_chapter_data_to_chapter(&chapter_data, manga_id) {
                    all_chapters.push(chapter);
                }
            }

            // Check if we've fetched all chapters
            if response.total <= offset + response.limit {
                break;
            }

            offset += response.limit;
        }

        // Sort chapters by number
        all_chapters.sort_by(|a, b| {
            a.number
                .partial_cmp(&b.number)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(all_chapters)
    }

    /// Map MangaDex chapter data to internal Chapter structure
    fn map_chapter_data_to_chapter(
        &self,
        data: &MangaDexChapterData,
        manga_id: &str,
    ) -> Option<Chapter> {
        let chapter_num = data
            .attributes
            .chapter
            .as_ref()
            .and_then(|ch| ch.parse::<f64>().ok())
            .unwrap_or(0.0);

        let title = data
            .attributes
            .title
            .clone()
            .unwrap_or_else(|| format!("Chapter {}", chapter_num));

        Some(Chapter {
            id: data.id.clone(),
            number: chapter_num,
            title,
            pages: vec![], // Pages are fetched separately
            manga_id: manga_id.to_string(),
            source_id: self.id().to_string(),
            #[cfg(feature = "sqlx")]
            created_at: None,
        })
    }

    /// Extract cover filename from relationship data
    fn extract_cover_filename(&self, data: &MangaDexMangaData) -> Option<String> {
        data.relationships
            .iter()
            .find(|rel| rel.rel_type == "cover_art")
            .and_then(|rel| {
                rel.attributes
                    .as_ref()
                    .and_then(|attr| attr.file_name.as_ref())
                    .map(|filename| filename.clone())
            })
    }

    /// Map MangaDx manga data to internal Manga structure
    fn map_manga_data_to_manga(&self, data: &MangaDexMangaData) -> Manga {
        let title = Self::extract_best_title(&data.attributes.title);
        let description = Self::extract_best_title(&data.attributes.description);

        // Extract authors from relationships
        let authors: Vec<String> = data
            .relationships
            .iter()
            .filter(|rel| rel.rel_type == "author" || rel.rel_type == "artist")
            .filter_map(|rel| {
                rel.attributes
                    .as_ref()
                    .and_then(|attr| attr.name.as_ref())
                    .map(|name| name.clone())
            })
            .collect();

        // Extract tags
        let tags: Vec<String> = data
            .attributes
            .tags
            .iter()
            .map(|tag| Self::extract_best_title(&tag.attributes.name))
            .collect();

        // Try to find cover art URL from relationships using reference expansion
        let cover_url = if let Some(filename) = self.extract_cover_filename(data) {
            let url = format!(
                "https://uploads.mangadex.org/covers/{}/{}",
                data.id, filename
            );
            Some(url)
        } else {
            None
        };

        Manga {
            id: data.id.clone(),
            title,
            cover_url,
            authors,
            description: if description.is_empty() || description == "Unknown Title" {
                None
            } else {
                Some(description)
            },
            tags,
            source_id: self.id().to_string(),
            #[cfg(feature = "sqlx")]
            created_at: None,
            #[cfg(feature = "sqlx")]
            updated_at: None,
        }
    }
}

impl Default for MangaDexSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Source for MangaDexSource {
    fn id(&self) -> &'static str {
        "mgd"
    }

    fn name(&self) -> &'static str {
        "MangaDex"
    }

    fn base_url(&self) -> &str {
        "https://mangadex.org"
    }

    async fn search(&self, params: SearchParams) -> Result<Vec<Manga>> {
        let query_params = self.format_search_query(&params.query, &params);
        let search_url = format!("{}/manga?{}", self.api_base, query_params);

        let response: MangaDexSearchResponse = self.client.get_json(&search_url).await?;

        let manga_list: Vec<Manga> = response
            .data
            .iter()
            .map(|manga_data| self.map_manga_data_to_manga(manga_data))
            .collect();

        Ok(manga_list)
    }

    async fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>> {
        self.fetch_all_chapters(manga_id).await
    }

    async fn get_pages(&self, chapter_id: &str) -> Result<Vec<String>> {
        // First, fetch chapter info to get manga ID
        let chapter_info_url = format!("{}/chapter/{}", self.api_base, chapter_id);

        let _chapter_info: MangaDexChapterResponse =
            self.client.get_json(&chapter_info_url).await?;

        // Then fetch page URLs from at-home server
        let pages_url = format!("{}/at-home/server/{}", self.api_base, chapter_id);
        let pages_response: MangaDexPagesResponse = self.client.get_json(&pages_url).await?;

        // Validate that we have the necessary data
        if pages_response.chapter.hash.is_empty() {
            return Err(crate::Error::parse("Chapter hash is empty".to_string()));
        }

        if pages_response.base_url.is_empty() {
            return Err(crate::Error::parse("Base URL is empty".to_string()));
        }

        // Construct full page URLs
        let page_urls: Vec<String> = if !pages_response.chapter.data.is_empty() {
            pages_response
                .chapter
                .data
                .iter()
                .map(|filename| {
                    format!(
                        "{}/data/{}/{}",
                        pages_response.base_url.trim_end_matches('/'),
                        pages_response.chapter.hash,
                        filename
                    )
                })
                .collect()
        } else if !pages_response.chapter.data_saver.is_empty() {
            pages_response
                .chapter
                .data_saver
                .iter()
                .map(|filename| {
                    format!(
                        "{}/data-saver/{}/{}",
                        pages_response.base_url.trim_end_matches('/'),
                        pages_response.chapter.hash,
                        filename
                    )
                })
                .collect()
        } else {
            Vec::new()
        };

        if page_urls.is_empty() {
            return Err(crate::Error::not_found(format!(
                "No pages found for chapter {}",
                chapter_id
            )));
        }
        Ok(page_urls)
    }
}
