//! TUI utilities and shared types for the Tosho terminal user interface.
//!
//! This module provides utilities, types, and functions that are shared between
//! the TUI binary and can be used by other applications that want to build
//! TUI interfaces on top of Tosho.
//!
//! # Features
//!
//! This module is only available when the `tui` feature is enabled.
//!
//! # Examples
//!
//! ```rust,no_run
//! use tosho::tui::{format_manga_title, AppState};
//! use tosho::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manga = Manga {
//!         id: "123".to_string(),
//!         title: "One Piece".to_string(),
//!         authors: vec!["Oda Eiichiro".to_string()],
//!         source_id: "mangadex".to_string(),
//!         cover_url: None,
//!         description: None,
//!         tags: vec!["Action".to_string()],
//!         #[cfg(feature = "sqlx")]
//!         created_at: None,
//!         #[cfg(feature = "sqlx")]
//!         updated_at: None,
//!     };
//!
//!     println!("{}", format_manga_title(&manga));
//!     Ok(())
//! }
//! ```

#[cfg(feature = "tui")]
use crate::types::{Chapter, Manga};
#[cfg(feature = "tui")]
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
#[cfg(all(feature = "tui", feature = "conversion"))]
use std::path::PathBuf;

/// Application state for TUI operations.
///
/// This struct provides utilities for managing application state
/// and formatting data for display in the terminal user interface.
#[cfg(feature = "tui")]
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_page: usize,
    pub items_per_page: usize,
    pub total_items: usize,
}

#[cfg(feature = "tui")]
impl AppState {
    /// Creates a new application state.
    pub fn new() -> Self {
        Self {
            current_page: 0,
            items_per_page: 20,
            total_items: 0,
        }
    }

    /// Updates the total number of items.
    pub fn set_total_items(&mut self, total: usize) {
        self.total_items = total;
        self.current_page = 0;
    }

    /// Gets the current page range for pagination.
    pub fn current_page_range(&self) -> (usize, usize) {
        let start = self.current_page * self.items_per_page;
        let end = ((start + self.items_per_page).min(self.total_items)).max(start);
        (start, end)
    }

    /// Moves to the next page if possible.
    pub fn next_page(&mut self) -> bool {
        let max_page = (self.total_items + self.items_per_page - 1) / self.items_per_page;
        if self.current_page + 1 < max_page {
            self.current_page += 1;
            true
        } else {
            false
        }
    }

    /// Moves to the previous page if possible.
    pub fn previous_page(&mut self) -> bool {
        if self.current_page > 0 {
            self.current_page -= 1;
            true
        } else {
            false
        }
    }

    /// Gets the total number of pages.
    pub fn total_pages(&self) -> usize {
        if self.total_items == 0 {
            1
        } else {
            (self.total_items + self.items_per_page - 1) / self.items_per_page
        }
    }
}

#[cfg(feature = "tui")]
impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Formats a manga title with styling for TUI display.
///
/// This function takes a manga object and returns a formatted Line
/// with colors and styling suitable for terminal user interface display.
///
/// # Examples
///
/// ```rust,no_run
/// use tosho::tui::format_manga_title;
/// use tosho::types::Manga;
///
/// let manga = Manga {
///     id: "123".to_string(),
///     title: "One Piece".to_string(),
///     authors: vec!["Oda Eiichiro".to_string()],
///     source_id: "mangadex".to_string(),
///     cover_url: None,
///     description: None,
///     tags: vec![],
///     #[cfg(feature = "sqlx")]
///     created_at: None,
///     #[cfg(feature = "sqlx")]
///     updated_at: None,
/// };
///
/// let formatted = format_manga_title(&manga);
/// ```
#[cfg(feature = "tui")]
pub fn format_manga_title(manga: &Manga) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            manga.title.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("({})", manga.source_id),
            Style::default().fg(Color::Green),
        ),
    ];

    if !manga.authors.is_empty() {
        spans.push(Span::raw(" by "));
        spans.push(Span::styled(
            manga.authors.join(", "),
            Style::default().fg(Color::Yellow),
        ));
    }

    Line::from(spans)
}

/// Formats a chapter title with styling for TUI display.
///
/// # Examples
///
/// ```rust,no_run
/// use tosho::tui::format_chapter_title;
/// use tosho::types::Chapter;
///
/// let chapter = Chapter {
///     id: "ch1".to_string(),
///     number: 1.0,
///     title: "Romance Dawn".to_string(),
///     pages: vec![],
///     manga_id: "one-piece".to_string(),
///     source_id: "mangadex".to_string(),
///     #[cfg(feature = "sqlx")]
///     created_at: None,
/// };
///
/// let formatted = format_chapter_title(&chapter);
/// ```
#[cfg(feature = "tui")]
pub fn format_chapter_title(chapter: &Chapter) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("Ch. {}", chapter.number),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled(chapter.title.clone(), Style::default().fg(Color::White)),
    ])
}

/// Formats a list of tags with colors for TUI display.
///
/// # Examples
///
/// ```rust,no_run
/// use tosho::tui::format_tags;
///
/// let tags = vec!["Action".to_string(), "Adventure".to_string()];
/// let formatted = format_tags(&tags);
/// ```
#[cfg(feature = "tui")]
pub fn format_tags(tags: &[String]) -> Vec<Line<'static>> {
    if tags.is_empty() {
        vec![Line::from(Span::styled(
            "None",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        tags.iter()
            .map(|tag| {
                Line::from(vec![
                    Span::raw("• "),
                    Span::styled(tag.clone(), Style::default().fg(Color::Magenta)),
                ])
            })
            .collect()
    }
}

/// Formats a description with proper wrapping for TUI display.
///
/// This function formats descriptions for display in the TUI with proper
/// text wrapping and styling.
///
/// # Parameters
///
/// * `description` - The description to format
/// * `width` - Maximum width for text wrapping
///
/// # Examples
///
/// ```rust,no_run
/// use tosho::tui::format_description;
///
/// let desc = Some("This is a description...".to_string());
/// let formatted = format_description(&desc, 50);
/// ```
#[cfg(feature = "tui")]
pub fn format_description(description: &Option<String>, width: usize) -> Vec<Line<'static>> {
    match description {
        Some(desc) => {
            // Simple word wrapping
            let words: Vec<&str> = desc.split_whitespace().collect();
            let mut lines = Vec::new();
            let mut current_line = String::new();

            for word in words {
                if current_line.len() + word.len() + 1 > width {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                }
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            }

            if !current_line.is_empty() {
                lines.push(Line::from(current_line));
            }

            lines
        }
        None => vec![Line::from(Span::styled(
            "No description available",
            Style::default().fg(Color::DarkGray),
        ))],
    }
}

/// Creates a styled status message for TUI display.
///
/// # Examples
///
/// ```rust,no_run
/// use tosho::tui::create_status_message;
/// use ratatui::style::Color;
///
/// let message = create_status_message("Success", "Download completed!", Color::Green);
/// ```
#[cfg(feature = "tui")]
pub fn create_status_message(prefix: &str, message: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{}:", prefix),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(message.to_string(), Style::default().fg(color)),
    ])
}

/// Creates a success message for TUI display.
#[cfg(feature = "tui")]
pub fn success_message(message: &str) -> Line<'static> {
    create_status_message("Success", message, Color::Green)
}

/// Creates a warning message for TUI display.
#[cfg(feature = "tui")]
pub fn warning_message(message: &str) -> Line<'static> {
    create_status_message("Warning", message, Color::Yellow)
}

/// Creates an error message for TUI display.
#[cfg(feature = "tui")]
pub fn error_message(message: &str) -> Line<'static> {
    create_status_message("Error", message, Color::Red)
}

/// Creates an info message for TUI display.
#[cfg(feature = "tui")]
pub fn info_message(message: &str) -> Line<'static> {
    create_status_message("Info", message, Color::Blue)
}

/// Validates and normalizes a chapter range specification.
///
/// Parses chapter specifications like "1,2,5-10" and returns a sorted list
/// of chapter numbers.
///
/// # Parameters
///
/// * `spec` - The chapter specification string
///
/// # Returns
///
/// A vector of chapter numbers in ascending order.
///
/// # Errors
///
/// Returns an error if the specification format is invalid.
///
/// # Examples
///
/// ```rust,no_run
/// use tosho::tui::parse_chapter_range;
///
/// let chapters = parse_chapter_range("1,3,5-8").unwrap();
/// assert_eq!(chapters, vec![1.0, 3.0, 5.0, 6.0, 7.0, 8.0]);
/// ```
#[cfg(feature = "tui")]
pub fn parse_chapter_range(spec: &str) -> Result<Vec<f64>, String> {
    let mut chapters = Vec::new();

    for part in spec.split(',') {
        let part = part.trim();
        if let Some((start, end)) = part.split_once('-') {
            let start_num: f64 = start
                .parse()
                .map_err(|_| format!("Invalid chapter number: {}", start))?;
            let end_num: f64 = end
                .parse()
                .map_err(|_| format!("Invalid chapter number: {}", end))?;

            if start_num > end_num {
                return Err(format!("Invalid range: {} > {}", start_num, end_num));
            }

            let mut current = start_num;
            while current <= end_num {
                chapters.push(current);
                current += 1.0;
            }
        } else {
            let num: f64 = part
                .parse()
                .map_err(|_| format!("Invalid chapter number: {}", part))?;
            chapters.push(num);
        }
    }

    chapters.sort_by(|a, b| a.partial_cmp(b).unwrap());
    chapters.dedup();

    Ok(chapters)
}

/// Creates a progress indicator for TUI display.
///
/// # Parameters
///
/// * `current` - Current progress value
/// * `total` - Total progress value
/// * `width` - Width of the progress bar
///
/// # Examples
///
/// ```rust,no_run
/// use tosho::tui::create_progress_bar;
///
/// let progress = create_progress_bar(75, 100, 20);
/// ```
#[cfg(feature = "tui")]
pub fn create_progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "█".repeat(width);
    }

    let progress = (current * width) / total;
    let completed = "█".repeat(progress);
    let remaining = "░".repeat(width.saturating_sub(progress));
    format!("{}{}", completed, remaining)
}

/// Truncates text to fit within a specified width.
///
/// # Parameters
///
/// * `text` - The text to truncate
/// * `width` - Maximum width
///
/// # Examples
///
/// ```rust,no_run
/// use tosho::tui::truncate_text;
///
/// let truncated = truncate_text("This is a very long text", 10);
/// assert_eq!(truncated, "This is...");
/// ```
#[cfg(feature = "tui")]
pub fn truncate_text(text: &str, width: usize) -> String {
    if text.len() <= width {
        text.to_string()
    } else if width > 3 {
        format!("{}...", &text[..width - 3])
    } else {
        text.chars().take(width).collect()
    }
}

/// Configuration for ebook conversion
#[cfg(all(feature = "tui", feature = "conversion"))]
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    pub output_format: EbookFormat,
    pub output_path: PathBuf,
    pub volume_grouping: VolumeGrouping,
    pub metadata: Option<ConversionMetadata>,
}

/// Ebook output formats supported by the conversion system
#[cfg(all(feature = "tui", feature = "conversion"))]
#[derive(Debug, Clone, PartialEq)]
pub enum EbookFormat {
    Cbz,
    Epub,
}

/// Volume grouping strategies for conversion
#[cfg(all(feature = "tui", feature = "conversion"))]
#[derive(Debug, Clone, PartialEq)]
pub enum VolumeGrouping {
    Name,
    ImageAnalysis,
    Manual(usize),
    Flat,
}

/// Metadata for ebook conversion
#[cfg(all(feature = "tui", feature = "conversion"))]
#[derive(Debug, Clone)]
pub struct ConversionMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub genre: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[cfg(all(feature = "tui", feature = "conversion"))]
impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            output_format: EbookFormat::Cbz,
            output_path: PathBuf::from("./converted"),
            volume_grouping: VolumeGrouping::Name,
            metadata: None,
        }
    }
}

#[cfg(all(feature = "tui", feature = "conversion"))]
impl Default for ConversionMetadata {
    fn default() -> Self {
        Self {
            title: "Untitled Manga".to_string(),
            authors: vec!["Unknown Author".to_string()],
            genre: None,
            publisher: None,
            description: None,
            tags: vec![],
        }
    }
}

/// Converts a source directory to ebook format using hozon
#[cfg(all(feature = "tui", feature = "conversion"))]
pub async fn convert_directory(
    source_path: PathBuf,
    config: ConversionConfig,
) -> Result<PathBuf, String> {
    use hozon::prelude::*;

    let file_format = match config.output_format {
        EbookFormat::Cbz => FileFormat::Cbz,
        EbookFormat::Epub => FileFormat::Epub,
    };

    let volume_strategy = match config.volume_grouping {
        VolumeGrouping::Name => VolumeGroupingStrategy::Name,
        VolumeGrouping::ImageAnalysis => VolumeGroupingStrategy::ImageAnalysis,
        VolumeGrouping::Manual(_size) => VolumeGroupingStrategy::Manual,
        VolumeGrouping::Flat => VolumeGroupingStrategy::Flat,
    };

    let metadata = if let Some(meta) = config.metadata {
        EbookMetadata {
            title: meta.title,
            authors: meta.authors,
            genre: meta.genre,
            publisher: meta.publisher,
            description: meta.description,
            tags: meta.tags,
            ..Default::default()
        }
    } else {
        EbookMetadata::default()
    };

    let hozon_config = HozonConfig::builder()
        .metadata(metadata)
        .source_path(source_path)
        .target_path(config.output_path.clone())
        .output_format(file_format)
        .volume_grouping_strategy(volume_strategy)
        .build()
        .map_err(|e| format!("Failed to build hozon config: {}", e))?;

    hozon_config
        .convert_from_source()
        .await
        .map_err(|e| format!("Conversion failed: {}", e))?;

    Ok(config.output_path)
}

/// Converts manga from manga/chapter data with downloaded images
#[cfg(all(feature = "tui", feature = "conversion"))]
pub async fn convert_manga_with_metadata(
    manga: &Manga,
    _chapters: &[Chapter],
    source_path: PathBuf,
    config: ConversionConfig,
) -> Result<PathBuf, String> {
    let metadata = ConversionMetadata {
        title: manga.title.clone(),
        authors: manga.authors.clone(),
        genre: if manga.tags.is_empty() {
            None
        } else {
            Some(manga.tags.join(", "))
        },
        publisher: Some(format!("Tosho ({})", manga.source_id)),
        description: manga.description.clone(),
        tags: manga.tags.clone(),
    };

    let updated_config = ConversionConfig {
        metadata: Some(metadata),
        ..config
    };

    convert_directory(source_path, updated_config).await
}

/// Creates a status message for conversion operations
#[cfg(all(feature = "tui", feature = "conversion"))]
pub fn conversion_status_message(message: &str) -> Line<'static> {
    create_status_message("Convert", message, Color::Cyan)
}

/// Creates a conversion progress indicator
#[cfg(all(feature = "tui", feature = "conversion"))]
pub fn create_conversion_progress(stage: &str, progress: Option<(usize, usize)>) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            "Converting:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(stage.to_string(), Style::default().fg(Color::White)),
    ];

    if let Some((current, total)) = progress {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("({}/{})", current, total),
            Style::default().fg(Color::Yellow),
        ));
    }

    Line::from(spans)
}

/// Formats conversion configuration for display
#[cfg(all(feature = "tui", feature = "conversion"))]
pub fn format_conversion_config(config: &ConversionConfig) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Format: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:?}", config.output_format),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Output: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                config.output_path.display().to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Grouping: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:?}", config.volume_grouping),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    // Add metadata information
    if let Some(ref metadata) = config.metadata {
        let title_text = if metadata.title.is_empty() {
            "Not set".to_string()
        } else {
            metadata.title.clone()
        };
        let title_color = if metadata.title.is_empty() {
            Color::Red
        } else {
            Color::White
        };

        lines.push(Line::from(vec![
            Span::styled("Title: ", Style::default().fg(Color::Yellow)),
            Span::styled(title_text, Style::default().fg(title_color)),
        ]));

        let authors_text = if metadata.authors.is_empty() {
            "Not set".to_string()
        } else {
            metadata.authors.join(", ")
        };
        let authors_color = if metadata.authors.is_empty() {
            Color::Red
        } else {
            Color::White
        };

        lines.push(Line::from(vec![
            Span::styled("Authors: ", Style::default().fg(Color::Yellow)),
            Span::styled(authors_text, Style::default().fg(authors_color)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("Metadata: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Not configured".to_string(),
                Style::default().fg(Color::Red),
            ),
        ]));
    }

    lines
}

#[cfg(test)]
#[cfg(feature = "tui")]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chapter_range() {
        assert_eq!(parse_chapter_range("1").unwrap(), vec![1.0]);
        assert_eq!(parse_chapter_range("1,3,5").unwrap(), vec![1.0, 3.0, 5.0]);
        assert_eq!(parse_chapter_range("1-3").unwrap(), vec![1.0, 2.0, 3.0]);
        assert_eq!(
            parse_chapter_range("1,3-5,7").unwrap(),
            vec![1.0, 3.0, 4.0, 5.0, 7.0]
        );

        assert!(parse_chapter_range("invalid").is_err());
        assert!(parse_chapter_range("5-3").is_err());
    }

    #[test]
    fn test_app_state_pagination() {
        let mut state = AppState::new();
        state.set_total_items(100);
        state.items_per_page = 10;

        assert_eq!(state.total_pages(), 10);
        assert_eq!(state.current_page_range(), (0, 10));

        assert!(state.next_page());
        assert_eq!(state.current_page_range(), (10, 20));

        assert!(state.previous_page());
        assert_eq!(state.current_page_range(), (0, 10));
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("Hello World", 5), "He...");
        assert_eq!(truncate_text("Hi", 10), "Hi");
        assert_eq!(truncate_text("Test", 3), "Tes");
    }

    #[test]
    fn test_progress_bar() {
        let progress = create_progress_bar(50, 100, 10);
        assert_eq!(progress.chars().count(), 10);
        assert!(progress.contains("█"));
        assert!(progress.contains("░"));
    }
}
