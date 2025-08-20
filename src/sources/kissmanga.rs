use crate::{
    error::Result,
    net::HttpClient,
    source::Source,
    types::{Chapter, Manga, SearchParams},
};
use async_trait::async_trait;
use scraper::{Html, Selector};

/// KissManga source implementation using Madara WordPress theme
pub struct KissMangaSource {
    client: HttpClient,
}

impl KissMangaSource {
    /// Create a new KissManga source
    pub fn new() -> Self {
        let mut client = HttpClient::new("kissmanga")
            .with_rate_limit(2000) // 2 seconds between requests
            .with_max_retries(3);

        // Add headers based on Go implementation
        client = client
            .with_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .with_header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .with_header("Accept-Language", "en-US,en;q=0.9")
            .with_header("Cache-Control", "no-cache")
            .with_header("Referer", "https://kissmanga.in/");

        Self { client }
    }
}

impl Default for KissMangaSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Source for KissMangaSource {
    fn id(&self) -> &'static str {
        "kmg"
    }

    fn name(&self) -> &'static str {
        "KissManga"
    }

    fn base_url(&self) -> &str {
        "https://kissmanga.in"
    }

    async fn search(&self, params: SearchParams) -> Result<Vec<Manga>> {
        let search_url = format!(
            "{}/?s={}&post_type=wp-manga",
            self.base_url(),
            urlencoding::encode(&params.query)
        );

        let html_str = self.client.get_text(&search_url).await?;
        let document = Html::parse_document(&html_str);

        // Try multiple selectors for search results
        let selectors = [
            "div.post-title h3 a",
            "div.post-title h5 a",
            ".post-title a",
        ];

        let mut manga_list = Vec::new();

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if let Some(href) = element.value().attr("href") {
                        let text = element.text().collect::<String>().trim().to_string();
                        if !text.is_empty() {
                            // Extract manga ID from URL
                            let id = href
                                .trim_end_matches('/')
                                .split('/')
                                .last()
                                .unwrap_or("unknown")
                                .to_string();

                            // Try to get cover image
                            let cover_url = self.extract_cover_from_search(&document, &id);

                            manga_list.push(Manga {
                                id,
                                title: text,
                                cover_url,
                                authors: vec![],
                                description: None,
                                tags: vec![],
                                source_id: self.id().to_string(),
                            });
                        }
                    }
                }

                // If we found results with this selector, break
                if !manga_list.is_empty() {
                    break;
                }
            }
        }

        // Apply limit if specified
        if let Some(limit) = params.limit {
            manga_list.truncate(limit);
        }

        Ok(manga_list)
    }

    async fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>> {
        let manga_url = format!("{}/manga/{}/", self.base_url(), manga_id);
        let html_str = self.client.get_text(&manga_url).await?;
        let document = Html::parse_document(&html_str);

        let mut chapters = Vec::new();

        // Try multiple selectors for chapters based on Go implementation
        let selectors = [
            "li.wp-manga-chapter > a", // Primary from Go
            ".chapter-link",           // Secondary from Go
            ".wp-manga-chapter a",
            "ul.main li a", // Additional common selectors
            ".chapter-list a",
            ".manga-chapters a",
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for (index, element) in document.select(&selector).enumerate() {
                    if let Some(href) = element.value().attr("href") {
                        let title = element.text().collect::<String>().trim().to_string();

                        // Use the full URL path as chapter ID since KissManga has complex URL structure
                        let chapter_id = if href.starts_with("http") {
                            // Extract the path part from full URL
                            href.strip_prefix("https://kissmanga.in")
                                .or_else(|| href.strip_prefix("http://kissmanga.in"))
                                .unwrap_or(href)
                                .to_string()
                        } else if href.starts_with('/') {
                            // Already a path
                            href.to_string()
                        } else {
                            // Relative path, add leading slash
                            format!("/{}", href)
                        };

                        // Try to extract chapter number from title or ID
                        let number = self
                            .extract_chapter_number(&title, &chapter_id)
                            .unwrap_or((index + 1) as f64);

                        chapters.push(Chapter {
                            id: chapter_id,
                            number,
                            title: if title.is_empty() {
                                format!("Chapter {}", number)
                            } else {
                                title
                            },
                            pages: vec![],
                            manga_id: manga_id.to_string(),
                            source_id: self.id().to_string(),
                        });
                    }
                }

                // If we found chapters with this selector, break
                if !chapters.is_empty() {
                    break;
                }
            }
        }

        // Sort chapters by number (ascending)
        chapters.sort_by(|a, b| {
            a.number
                .partial_cmp(&b.number)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(chapters)
    }

    async fn get_pages(&self, chapter_id: &str) -> Result<Vec<String>> {
        // Based on the Go implementation, try different URL patterns for KissManga chapters
        // Since we now store the full path as chapter_id, construct the full URL
        let possible_urls = [
            // Primary: Use the stored path directly (this should work for KissManga)
            format!("{}{}", self.base_url(), chapter_id),
            // Fallback: Try without trailing slash if present
            format!("{}{}", self.base_url(), chapter_id.trim_end_matches('/')),
            // Legacy fallbacks for other possible formats
            format!(
                "{}/manga/{}/",
                self.base_url(),
                chapter_id.trim_start_matches('/')
            ),
            format!(
                "{}/chapter/{}/",
                self.base_url(),
                chapter_id.trim_start_matches('/')
            ),
        ];

        for (_i, chapter_url) in possible_urls.iter().enumerate() {
            match self.client.get_text(chapter_url).await {
                Ok(html_str) => {
                    let document = Html::parse_document(&html_str);

                    // Enhanced selectors based on Go implementation
                    let selectors = [
                        "div.page-break img",   // Primary selector from Go
                        ".reading-content img", // Secondary selector from Go
                        ".wp-manga-chapter-img img",
                        "#readerarea img",
                        ".entry-content img",
                        ".chapter-content img",
                        "img[data-src]", // For lazy-loaded images
                    ];

                    for selector_str in &selectors {
                        if let Ok(selector) = Selector::parse(selector_str) {
                            let pages: Vec<String> = document
                                .select(&selector)
                                .filter_map(|img| {
                                    // Try multiple src attributes as images might be lazy-loaded
                                    let src = img
                                        .value()
                                        .attr("src")
                                        .or_else(|| img.value().attr("data-src"))
                                        .or_else(|| img.value().attr("data-lazy-src"))
                                        .or_else(|| img.value().attr("data-original"));

                                    if let Some(src) = src {
                                        // Clean up whitespace and line breaks from the URL
                                        let cleaned_src =
                                            src.trim().replace('\n', "").replace('\t', "");

                                        // Skip placeholder images and invalid URLs
                                        if cleaned_src.contains("placeholder")
                                            || cleaned_src.contains("loading")
                                            || cleaned_src.len() < 10
                                        {
                                            return None;
                                        }

                                        // Convert relative URLs to absolute
                                        let absolute_url = if cleaned_src.starts_with("http") {
                                            cleaned_src
                                        } else if cleaned_src.starts_with("//") {
                                            format!("https:{}", cleaned_src)
                                        } else if cleaned_src.starts_with('/') {
                                            format!("{}{}", self.base_url(), cleaned_src)
                                        } else {
                                            format!("{}/{}", self.base_url(), cleaned_src)
                                        };

                                        Some(absolute_url)
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            if !pages.is_empty() {
                                println!(
                                    "Debug: Found {} pages for chapter {}",
                                    pages.len(),
                                    chapter_id
                                );
                                for (i, page_url) in pages.iter().enumerate() {
                                    println!("Debug: Page {}: {}", i + 1, page_url);
                                }
                                return Ok(pages);
                            }
                        }
                    }
                }
                Err(_) => {
                    continue; // Try next URL format
                }
            }
        }

        Err(crate::Error::not_found(format!(
            "No pages found for chapter {} after trying all URL formats",
            chapter_id
        )))
    }

    async fn download_chapter(
        &self,
        chapter_id: &str,
        output_dir: &std::path::Path,
    ) -> crate::Result<std::path::PathBuf> {
        self.download_chapter_with_headers(chapter_id, output_dir)
            .await
    }
}

impl KissMangaSource {
    /// Custom download implementation for KissManga with proper headers
    async fn download_chapter_with_headers(
        &self,
        chapter_id: &str,
        output_dir: &std::path::Path,
    ) -> crate::Result<std::path::PathBuf> {
        use tokio::fs;
        use tokio::io::AsyncWriteExt;

        let pages = self.get_pages(chapter_id).await?;
        if pages.is_empty() {
            return Err(crate::Error::source(
                self.id(),
                "No pages found for chapter",
            ));
        }

        println!("Debug: About to download {} pages", pages.len());
        for (i, page_url) in pages.iter().enumerate() {
            println!("Debug: Page {}: {}", i + 1, page_url);
        }

        // Create chapter directory
        let chapter_dir = output_dir.join(format!("chapter_{}", chapter_id.replace('/', "_")));
        fs::create_dir_all(&chapter_dir).await.map_err(|e| {
            crate::Error::source(self.id(), format!("Failed to create directory: {}", e))
        })?;

        // Download each page with KissManga-specific headers
        let client = reqwest::Client::new();
        for (i, page_url) in pages.iter().enumerate() {
            println!("Debug: Downloading page {} from: {}", i + 1, page_url);

            let response = client
                .get(page_url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .header("Accept", "image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
                .header("Accept-Language", "en-US,en;q=0.9")
                .header("Cache-Control", "no-cache")
                .header("Referer", "https://kissmanga.in/")
                .send()
                .await
                .map_err(|e| {
                    crate::Error::parse(format!("Failed to download page {}: {}", i + 1, e))
                })?;

            println!(
                "Debug: Response status for page {}: {}",
                i + 1,
                response.status()
            );

            if !response.status().is_success() {
                println!(
                    "Debug: Failed to download page {} with status: {}",
                    i + 1,
                    response.status()
                );
                continue; // Skip this page instead of failing the whole chapter
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

            println!(
                "Debug: Successfully downloaded page {} ({} bytes)",
                i + 1,
                bytes.len()
            );
        }

        println!(
            "Downloaded {} pages to {}",
            pages.len(),
            chapter_dir.display()
        );
        Ok(chapter_dir)
    }
    /// Extract cover image URL from search results
    fn extract_cover_from_search(&self, document: &Html, _manga_id: &str) -> Option<String> {
        // Try to find cover image in the search results
        let selectors = [".post-thumb img", ".tab-thumb img", ".manga-cover img"];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(img) = document.select(&selector).next() {
                    if let Some(src) = img
                        .value()
                        .attr("src")
                        .or_else(|| img.value().attr("data-src"))
                        .or_else(|| img.value().attr("data-lazy-src"))
                    {
                        return Some(if src.starts_with("http") {
                            src.to_string()
                        } else if src.starts_with("//") {
                            format!("https:{}", src)
                        } else {
                            format!("{}{}", self.base_url(), src)
                        });
                    }
                }
            }
        }
        None
    }

    /// Extract chapter number from title or ID
    fn extract_chapter_number(&self, title: &str, chapter_id: &str) -> Option<f64> {
        // Try to extract number from title first
        let title_lower = title.to_lowercase();

        // Look for patterns like "Chapter 123" or "Ch. 123" or "123"
        if let Some(captures) = regex::Regex::new(r"(?i)(?:chapter|ch\.?)\s*(\d+(?:\.\d+)?)")
            .ok()?
            .captures(&title_lower)
        {
            return captures.get(1)?.as_str().parse().ok();
        }

        // Look for standalone numbers
        if let Some(captures) = regex::Regex::new(r"(\d+(?:\.\d+)?)")
            .ok()?
            .captures(&title_lower)
        {
            return captures.get(1)?.as_str().parse().ok();
        }

        // Try to extract from chapter ID
        if let Some(captures) = regex::Regex::new(r"(?i)(?:chapter|ch)-?(\d+(?:\.\d+)?)")
            .ok()?
            .captures(chapter_id)
        {
            return captures.get(1)?.as_str().parse().ok();
        }

        // Last resort: look for any number in the ID
        if let Some(captures) = regex::Regex::new(r"(\d+(?:\.\d+)?)")
            .ok()?
            .captures(chapter_id)
        {
            return captures.get(1)?.as_str().parse().ok();
        }

        None
    }
}
