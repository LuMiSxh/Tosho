# Tosho - Manga Aggregation Library

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/lumisxh/tosho/workflows/Release%20and%20Documentation/badge.svg)](https://github.com/lumisxh/tosho/actions)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://lumisxh.github.io/tosho/)

**Tosho** is a high-performance, async manga aggregation library that provides a unified interface for searching, downloading, and converting manga content from multiple sources. Built with Rust's async/await and designed for speed, reliability, and ease of use.

> **Note**: This project is currently in development and not yet ready for production use.

## Library Usage

Add Tosho to your `Cargo.toml`:

```toml
[dependencies]
tosho = { git = "https://github.com/lumisxh/tosho", tag = "vX.X.X" }  # Replace `vX.X.X` with the version you want to use

# With SQLx compatibility for database storage
tosho = { git = "https://github.com/lumisxh/tosho", tag = "vX.X.X", features = ["sqlx"] }

# With Specta support for type-safe APIs
tosho = { git = "https://github.com/lumisxh/tosho", tag = "vX.X.X", features = ["specta"] }

# Minimal build with only MangaDex
tosho = { git = "https://github.com/lumisxh/tosho", tag = "vX.X.X", default-features = false, features = ["source-mangadex"] }
```

### Available Features

- `sqlx` - Adds SQLx derive traits for database compatibility
- `specta` - Adds Specta derive traits for type-safe APIs
- `source-mangadex` - MangaDex source support
- `source-kissmanga` - KissManga source support
- `all-sources` - All available sources (default)

## Quick Example

```rust
use tosho::prelude::*;
use tosho::error::Result;
use tosho::sources::MangaDexSource;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize sources
    let mut sources = Sources::new();
    sources.add(MangaDexSource::new());

    // Search and process results
    let results = sources
        .search("one piece")
        .limit(20)
        .sort_by(SortOrder::UpdatedAt)
        .flatten()
        .await?
        .dedupe_by_title()
        .sort_by_relevance();

    println!("Found {} unique results", results.len());

    // Download and optionally convert a chapter
    if let Some(manga) = results.first() {
        let source = sources.get(&manga.source_id).unwrap();
        let chapters = source.get_chapters(&manga.id).await?;

        if let Some(chapter) = chapters.first() {
            let download_dir = std::path::Path::new("./downloads");
            let chapter_path = source.download_chapter(&chapter.id, &download_dir).await?;
            println!("Downloaded to: {}", chapter_path.display());
        }

        }
    }

    Ok(())
}
```

### With TUI Conversion Features

```rust
use tosho::prelude::*;
use std::path::PathBuf;

#[cfg(feature = "conversion")]
use tosho::tui::{convert_manga_with_metadata, ConversionConfig, EbookFormat};

#[tokio::main]
async fn main() -> Result<()> {
    let mut sources = Sources::new();
    sources.add(tosho::sources::MangaDexSource::new());

    let results = sources
        .search("one piece")
        .limit(20)
        .sort_by(SortOrder::UpdatedAt)
        .flatten()
        .await?
        .dedupe_by_title()
        .sort_by_relevance();

    println!("Found {} unique results", results.len());

    // Download and convert to ebook format (requires "tui" feature)
    #[cfg(feature = "tui")]
    if let Some(manga) = results.first() {
        let source = sources.get(&manga.source_id).unwrap();
        let chapters = source.get_chapters(&manga.id).await?;

        if let Some(chapter) = chapters.first() {
            let download_dir = std::path::Path::new("./downloads");
            let chapter_path = source.download_chapter(&chapter.id, &download_dir).await?;

            println!("Downloaded to: {}", chapter_path.display());

            // Convert to ebook format
            let config = ConversionConfig {
                output_format: EbookFormat::Epub,
                output_path: PathBuf::from("./converted"),
                ..Default::default()
            };

            let output_path = convert_manga_with_metadata(
                manga,
                &[chapter.clone()],
                chapter_path,
                config
            ).await?;

            println!("Converted to: {}", output_path.display());
        }
    }

    Ok(())
}
```

## Documentation

### API Documentation

Comprehensive API documentation is automatically generated and available at:
**[https://lumisxh.github.io/tosho/](https://lumisxh.github.io/Tosho/)**

The documentation includes:

- Complete API reference with examples
- Search strategies and result processing
- Source implementation guide
- HTTP client and rate limiting details
- Database integration patterns
- Download utilities and file handling

### Available Sources

- **MangaDex** - Feature: `source-mangadex`
- **KissManga** - Feature: `source-kissmanga`

## Development Status

This library is actively developed with automated testing and security auditing. Check the [Actions page](https://github.com/lumisxh/tosho/actions) for current build status and release information.

## Contributing

Contributions are welcome! Please see the API documentation for development guidelines, architecture details, and instructions for implementing new manga sources.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
