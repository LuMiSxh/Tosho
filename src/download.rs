//! Simplified download functionality for manga chapters.
//!
//! This module provides basic utilities for downloading manga content with minimal configuration.
//! Downloads are handled directly by the sources using their `download_chapter` method.

use crate::error::{Error, Result};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Downloads a single file from a URL to a local path.
///
/// This is a simple utility function for downloading individual files.
/// Used internally by sources for downloading manga pages.
///
/// # Parameters
///
/// * `url` - The URL to download from
/// * `output_path` - Where to save the downloaded file
///
/// # Returns
///
/// The number of bytes downloaded.
///
/// # Examples
///
/// ```rust,no_run
/// use tosho::download::download_file;
/// use std::path::Path;
///
/// # async fn example() -> tosho::Result<()> {
/// let bytes = download_file(
///     "https://example.com/image.jpg",
///     Path::new("./image.jpg")
/// ).await?;
/// println!("Downloaded {} bytes", bytes);
/// # Ok(())
/// # }
/// ```
pub async fn download_file(url: &str, output_path: &Path) -> Result<u64> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| Error::parse(format!("Failed to download {}: {}", url, e)))?;

    if !response.status().is_success() {
        return Err(Error::parse(format!(
            "Failed to download {}: HTTP {}",
            url,
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| Error::parse(format!("Failed to read data from {}: {}", url, e)))?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| Error::source("download", format!("Failed to create directory: {}", e)))?;
    }

    let mut file = fs::File::create(output_path)
        .await
        .map_err(|e| Error::source("download", format!("Failed to create file: {}", e)))?;

    file.write_all(&bytes)
        .await
        .map_err(|e| Error::source("download", format!("Failed to write file: {}", e)))?;

    Ok(bytes.len() as u64)
}

/// Sanitizes a filename by replacing invalid characters.
///
/// This function removes or replaces characters that are not allowed in filenames
/// on most operating systems.
///
/// # Parameters
///
/// * `name` - The filename to sanitize
///
/// # Returns
///
/// A sanitized filename safe for use on most filesystems.
///
/// # Examples
///
/// ```rust
/// use tosho::download::sanitize_filename;
///
/// let clean = sanitize_filename("Chapter: 1 - The Beginning!");
/// assert_eq!(clean, "Chapter_ 1 - The Beginning!");
/// ```
pub fn sanitize_filename(name: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut sanitized = name.to_string();

    for &ch in &invalid_chars {
        sanitized = sanitized.replace(ch, "_");
    }

    // Trim whitespace and limit length
    sanitized = sanitized.trim().to_string();
    if sanitized.len() > 200 {
        sanitized.truncate(200);
    }

    // Ensure we have a valid filename
    if sanitized.is_empty() {
        sanitized = "untitled".to_string();
    }

    sanitized
}

/// Extracts file extension from a URL.
///
/// This function attempts to determine the file extension from a URL,
/// ignoring query parameters and fragments.
///
/// # Parameters
///
/// * `url` - The URL to extract extension from
///
/// # Returns
///
/// The file extension (without the dot) if found, None otherwise.
///
/// # Examples
///
/// ```rust
/// use tosho::download::extract_extension;
///
/// assert_eq!(extract_extension("https://example.com/image.jpg"), Some("jpg".to_string()));
/// assert_eq!(extract_extension("https://example.com/image.png?v=123"), Some("png".to_string()));
/// assert_eq!(extract_extension("https://example.com/image"), None);
/// ```
pub fn extract_extension(url: &str) -> Option<String> {
    // Remove query parameters and fragments
    let clean_url = url.split('?').next()?.split('#').next()?;

    // Get the path part
    let path = clean_url.split('/').last()?;

    // Extract extension
    if let Some(dot_pos) = path.rfind('.') {
        let ext = &path[dot_pos + 1..];
        if !ext.is_empty() && ext.len() <= 10 {
            return Some(ext.to_lowercase());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal_file.txt"), "normal_file.txt");
        assert_eq!(
            sanitize_filename("file/with\\bad:chars"),
            "file_with_bad_chars"
        );
        assert_eq!(sanitize_filename(""), "untitled");

        // Test length limiting
        let long_name = "a".repeat(250);
        let sanitized = sanitize_filename(&long_name);
        assert!(sanitized.len() <= 200);
    }

    #[test]
    fn test_extract_extension() {
        assert_eq!(
            extract_extension("https://example.com/image.jpg"),
            Some("jpg".to_string())
        );
        assert_eq!(
            extract_extension("https://example.com/image.PNG"),
            Some("png".to_string())
        );
        assert_eq!(
            extract_extension("https://example.com/image.jpg?v=123"),
            Some("jpg".to_string())
        );
        assert_eq!(extract_extension("https://example.com/image"), None);
        assert_eq!(extract_extension("https://example.com/image."), None);
    }
}
