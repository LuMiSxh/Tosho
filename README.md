# Tosho

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Tosho** is a high-performance, async manga aggregation library that provides a unified interface for searching and retrieving manga content from multiple sources. Built with Rust's async/await and designed for speed, reliability, and ease of use.

> **Note**: This project is currently in development and not yet ready for production use.

## Features

- **High Performance**: Built on tokio with parallel processing using rayon
- **Async/Await**: Full async support for concurrent operations
- **Unified API**: Search across multiple manga sources with a single interface
- **Fluent Builder**: Chain search parameters and execution strategies elegantly
- **Rate Limiting**: Per-source rate limiting to respect website policies
- **Robust Error Handling**: Comprehensive error types with detailed context
- **Result Processing**: Built-in deduplication, sorting, and filtering capabilities
- **Extensible**: Easy to add new manga sources

## Quick Start

Add Tosho to your `Cargo.toml`:

```toml
[dependencies]
tosho = { path = "../tosho" }  # or git = "https://github.com/lumisxh/tosho"
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
```

### Basic Usage

## Quick Start

```rust
use tosho::prelude::*;
use tosho::sources::{kissmanga::KissMangaSource, mangadex::MangaDexSource};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize sources
    let mut sources = Sources::new();
    sources.add(KissMangaSource::new());
    sources.add(MangaDxSource::new());

    // Search across all sources
    let results = sources
        .search("one piece")
        .limit(20)
        .sort_by(SortOrder::UpdatedAt)
        .flatten()
        .await?
        .dedupe_by_title()
        .sort_by_relevance();

    println!("Found {} unique results", results.len());

    // Download a chapter
    if let Some(manga) = results.first() {
        let source = sources.get(&manga.source_id).unwrap();
        let chapters = source.get_chapters(&manga.id).await?;

        if let Some(chapter) = chapters.first() {
            let pages = source.get_pages(&chapter.id).await?;
            let download_pages = tosho::download::pages_from_urls(pages);

            let config = DownloadConfigBuilder::default()
                .concurrency(3usize)
                .throttle_ms(1000u64)
                .build()
                .unwrap();

            let manager = DownloadManager::new(config);

            let stats = manager.download_chapter(
                &manga.title,
                Some(chapter.number),
                &download_pages,
                |progress| {
                    tokio::spawn(async move {
                        let stats = progress.stats().await;
                        println!("Progress: {:.1}%", stats.percentage());
                    });
                }
            ).await?;

            println!("Downloaded {} pages", stats.completed_pages);
        }
    }

    Ok(())
}
```

### Advanced Search

```rust
use tosho::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let sources = Sources::new();

    // Advanced search with filtering and processing
    let results = sources
        .search("shounen manga")
        .limit(50)
        .include_tags(vec!["Action".to_string(), "Adventure".to_string()])
        .exclude_tags(vec!["Ecchi".to_string()])
        .sort_by(SortOrder::UpdatedAt)
        .flatten()
        .await?
        .dedupe_by_title()        // Remove duplicates
        .sort_by_relevance();     // Sort by relevance

    println!("Found {} unique results", results.len());
    Ok(())
}
```

## API Overview

### Search Strategies

Tosho supports multiple search execution strategies:

```rust
// Flatten results from all sources
let all_results = sources.search("naruto").flatten().await?;

// Group results by source (useful for debugging)
let grouped = sources.search("naruto").group().await;
for (source_id, result) in grouped {
    match result {
        Ok(manga) => println!("{}: {} results", source_id, manga.len()),
        Err(e) => println!("{}: Error - {}", source_id, e),
    }
}

// Search specific source only
let specific = sources.search("naruto").from_source("mangadx").await?;
```

### Source Management

```rust
let mut sources = Sources::new();

// Add sources
sources.add(MangaDxSource::new())
       .add(MadaraSource::new("https://example.com"));

// Get source information
println!("Available sources: {:?}", sources.list_ids());
println!("Total sources: {}", sources.len());

// Access specific source
if let Some(source) = sources.get("mangadx") {
    let chapters = source.get_chapters("manga_id").await?;
}
```

### Result Processing

Built-in methods for processing search results:

```rust
let processed = sources
    .search("popular manga")
    .limit(100)
    .flatten()
    .await?
    .dedupe_by_title()        // Remove duplicate titles
    .filter_popular(4)        // Filter well-documented manga (score >= 4)
    .sort_by_relevance();     // Sort by relevance score
```

## Architecture

The library is organized into several key modules:

- [`source`]: Core trait and collection for manga sources
- [`search`]: Fluent search builder and result processing
- [`types`]: Core data structures for manga, chapters, and search parameters
- [`net`]: HTTP client, rate limiting, and parsing utilities
- [`error`]: Comprehensive error handling
- [`download`]: Download manager with concurrent processing and progress tracking
- [`sources`]: Built-in source implementations (KissManga, MangaDx)

### Core Types

```rust
// Manga metadata
pub struct Manga {
    pub id: String,
    pub title: String,
    pub cover_url: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub source_id: String,
}

// Chapter information
pub struct Chapter {
    pub id: String,
    pub number: f64,
    pub title: String,
    pub pages: Vec<String>,
    pub manga_id: String,
    pub source_id: String,
}
```

## Implementing a Source

To add support for a new manga website, implement the `Source` trait:

```rust
use tosho::prelude::*;
use async_trait::async_trait;

struct MyMangaSource {
    base_url: String,
    client: tosho::net::HttpClient,
}

#[async_trait]
impl Source for MyMangaSource {
    fn id(&self) -> &'static str { "my_source" }
    fn name(&self) -> &'static str { "My Manga Source" }
    fn base_url(&self) -> &str { &self.base_url }

    async fn search(&self, params: SearchParams) -> Result<Vec<Manga>> {
        // Implement search logic using self.client
        // Parse HTML/JSON responses
        // Return manga results
        todo!()
    }

    async fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>> {
        // Implement chapter fetching
        todo!()
    }

    async fn get_pages(&self, chapter_id: &str) -> Result<Vec<String>> {
        // Implement page URL extraction
        todo!()
    }
}
```

### HTTP Client Usage

The library provides a robust HTTP client with rate limiting:

```rust
use tosho::net::HttpClient;

let client = HttpClient::new("my_source")
    .with_rate_limit(1000)  // 1 second between requests
    .with_max_retries(3);   // Retry up to 3 times

// GET requests with automatic retries and rate limiting
let html = client.get_text("https://example.com/manga/123").await?;
let json: ApiResponse = client.get_json("https://api.example.com/manga").await?;
```

### HTML Parsing

Convenient HTML parsing utilities:

```rust
use tosho::net::html;

let document = html::parse(&html_content);

// Extract data using CSS selectors
let title = html::select_text(&document, ".manga-title");
let cover_url = html::select_attr(&document, ".cover img", "src");
let tags = html::select_all_text(&document, ".tag");

// Parse manga items in parallel
let manga_list = html::parse_manga_items(&document, ".manga-item", |element| {
    // Extract manga data from each element
    Some(manga)
});
```

## Error Handling

Tosho provides comprehensive error types:

```rust
use tosho::{Result, Error};

match sources.search("query").flatten().await {
    Ok(results) => println!("Found {} results", results.len()),
    Err(Error::Network(e)) => println!("Network error: {}", e),
    Err(Error::RateLimit { retry_after }) => {
        println!("Rate limited, retry after: {:?}", retry_after);
    }
    Err(Error::Source { src, message }) => {
        println!("Source {} error: {}", src, message);
    }
    Err(e) => println!("Other error: {}", e),
}
```

## Examples

Check out the [examples](examples/) directory for more usage examples:

- `search_api.rs` - Comprehensive search API showcase
- More examples coming soon!

## Documentation

- Full API documentation (run `cargo doc --open`)
- [Examples](examples/) - Usage examples and tutorials

## Status

✅ **Complete Implementation**

- KissManga and MangaDx source providers
- Unified search API with filtering and sorting
- Integrated download manager with progress tracking
- Comprehensive error handling and rate limiting
- Full async/await support with tokio

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This library is for educational purposes. Please respect the terms of service of the manga websites you interact with and consider supporting official sources.
