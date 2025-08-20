# Tosho

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Tosho** is a high-performance, async manga aggregation library that provides a unified interface for searching and downloading manga content from multiple sources. Built with Rust's async/await and designed for speed, reliability, and ease of use.

> **Note**: This project is currently in development and not yet ready for production use.

## Features

- **High Performance**: Built on tokio with parallel processing using rayon
- **Async/Await**: Full async support for concurrent operations
- **Unified API**: Search across multiple manga sources with a single interface
- **Fluent Builder**: Chain search parameters and execution strategies elegantly
- **Integrated Downloads**: Direct chapter downloading through source implementations
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

```rust
use tosho::prelude::*;
use tosho::sources::mangadex::MangaDexSource;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize sources
    let mut sources = Sources::new();
    sources.add(MangaDexSource::new());

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
            let download_dir = std::path::Path::new("./downloads");
            let chapter_path = source.download_chapter(&chapter.id, &download_dir).await?;

            println!("Downloaded to: {}", chapter_path.display());
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
    let mut sources = Sources::new();
    sources.add(tosho::sources::mangadex::MangaDexSource::new());
    sources.add(tosho::sources::kissmanga::KissMangaSource::new());

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
sources.add(tosho::sources::mangadex::MangaDexSource::new());
sources.add(tosho::sources::kissmanga::KissMangaSource::new());

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

### Downloads

Each source provides integrated download functionality:

```rust
use std::path::Path;

// Get a source
let source = tosho::sources::mangadex::MangaDexSource::new();

// Search for manga
let manga_list = source.search("oneshot".into()).await?;
let manga = &manga_list[0];

// Get chapters
let chapters = source.get_chapters(&manga.id).await?;
let chapter = &chapters[0];

// Download chapter - creates a directory with all pages
let download_dir = Path::new("./downloads");
let chapter_path = source.download_chapter(&chapter.id, &download_dir).await?;

println!("Downloaded to: {}", chapter_path.display());
```

## Architecture

The library is organized into several key modules:

- [`source`]: Core trait and collection for manga sources
- [`search`]: Fluent search builder and result processing
- [`types`]: Core data structures for manga, chapters, and search parameters
- [`net`]: HTTP client, rate limiting, and parsing utilities
- [`error`]: Comprehensive error handling
- [`download`]: Simple download utilities for individual files
- [`sources`]: Built-in source implementations (MangaDx, KissManga, Madara)

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

## Available Sources

Tosho currently supports the following manga sources:

- **MangaDx**: High-quality manga aggregator
- **KissManga**: Popular manga reading site
- **Madara**: Framework for manga sites (configurable base URL)

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

    // Optional: Override default download behavior
    async fn download_chapter(
        &self,
        chapter_id: &str,
        output_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf> {
        // Custom download implementation
        // Default implementation downloads all pages to a directory
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

## Download Utilities

The library includes simple utilities for file downloads:

```rust
use tosho::download::{download_file, sanitize_filename, extract_extension};
use std::path::Path;

// Download a single file
let bytes = download_file("https://example.com/image.jpg", Path::new("./image.jpg")).await?;

// Sanitize filenames for safe filesystem use
let safe_name = sanitize_filename("Chapter: 1 - The Beginning!");

// Extract file extension from URL
let ext = extract_extension("https://example.com/image.jpg?v=123"); // Some("jpg")
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
