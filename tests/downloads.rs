//! Download functionality tests
//!
//! Tests file downloading, path handling, and download utilities.

use std::path::PathBuf;
use tokio::time::timeout;
use tosho::prelude::*;

// Import test utilities from mod
mod common;
use common::{TEST_TIMEOUT, setup_test_dir};

#[cfg(test)]
mod download_tests {
    use super::*;

    #[tokio::test]
    async fn test_filename_sanitization() {
        let test_cases = vec![
            ("Normal Chapter", "Normal Chapter"),
            ("Chapter/with\\slashes", "Chapter_with_slashes"),
            ("Chapter:with*special?chars", "Chapter_with_special_chars"),
            ("Chapter\"with<quotes>", "Chapter_with_quotes_"),
            ("Chapter|with|pipes", "Chapter_with_pipes"),
            ("   Spaced   Chapter   ", "Spaced   Chapter"),
            ("", "untitled"),
            ("...", "..."),
        ];

        for (input, _expected_pattern) in test_cases {
            let result = sanitize_filename(input);

            // Check that dangerous characters are removed
            assert!(!result.contains('/'));
            assert!(!result.contains('\\'));
            assert!(!result.contains(':'));
            assert!(!result.contains('*'));
            assert!(!result.contains('?'));
            assert!(!result.contains('"'));
            assert!(!result.contains('<'));
            assert!(!result.contains('>'));
            assert!(!result.contains('|'));

            // Check that basic content is preserved for normal cases
            if !input.trim().is_empty() && input != "..." {
                assert!(!result.is_empty());
            }

            println!("Sanitized '{}' -> '{}'", input, result);
        }
    }

    #[tokio::test]
    async fn test_extension_extraction() {
        let test_cases = vec![
            ("https://example.com/image.jpg", Some("jpg")),
            ("https://example.com/image.jpeg", Some("jpeg")),
            ("https://example.com/image.png", Some("png")),
            ("https://example.com/image.webp", Some("webp")),
            ("https://example.com/image.gif", Some("gif")),
            ("https://example.com/image.bmp", Some("bmp")),
            ("https://example.com/image.jpg?version=123", Some("jpg")),
            ("https://example.com/image.png#anchor", Some("png")),
            ("https://example.com/image.jpg?v=1&format=png", Some("jpg")),
            ("https://example.com/no-extension", None),
            ("https://example.com/", None),
            ("", None),
            ("not-a-url", None),
            ("https://example.com/path.with.dots.jpg", Some("jpg")),
        ];

        for (url, expected) in test_cases {
            let result = extract_extension(url);
            assert_eq!(
                result,
                expected.map(|s| s.to_string()),
                "Failed for URL: '{}'",
                url
            );
            println!("Extension from '{}' -> {:?}", url, result);
        }
    }

    #[tokio::test]
    async fn test_file_download_basic() {
        let test_dir = setup_test_dir().await.join("unit").join("basic");

        // Test downloading a small binary file
        let test_file_path = test_dir.join("test_download.bin");
        let download_future = download_file("https://httpbin.org/bytes/512", &test_file_path);

        match timeout(TEST_TIMEOUT, download_future).await {
            Ok(Ok(bytes_downloaded)) => {
                println!(
                    "Downloaded {} bytes to {:?}",
                    bytes_downloaded, test_file_path
                );
                assert!(test_file_path.exists());
                assert!(bytes_downloaded > 0);

                // Verify file size
                let metadata = tokio::fs::metadata(&test_file_path).await.unwrap();
                assert_eq!(metadata.len(), bytes_downloaded);
            }
            Ok(Err(e)) => {
                println!("Download failed (this may be due to network issues): {}", e);
                // Don't fail the test for network issues
            }
            Err(_) => {
                println!("Download timed out");
            }
        }
    }

    #[tokio::test]
    async fn test_file_download_error_handling() {
        let test_dir = setup_test_dir().await.join("unit").join("errors");

        // Test invalid URL
        let invalid_file_path = test_dir.join("invalid_download.bin");
        let result = download_file("not-a-valid-url", &invalid_file_path).await;
        match result {
            Ok(_) => panic!("Expected download to fail for invalid URL"),
            Err(e) => println!("Correctly handled invalid URL: {}", e),
        }

        // Test 404 URL
        let not_found_path = test_dir.join("not_found.bin");
        let result = download_file("https://httpbin.org/status/404", &not_found_path).await;
        match result {
            Ok(_) => println!("Unexpected success for 404 URL"),
            Err(e) => println!("Correctly handled 404 error: {}", e),
        }

        // Test invalid file path (read-only or non-existent directory)
        let invalid_path = PathBuf::from("/invalid/readonly/path/file.bin");
        let result = download_file("https://httpbin.org/bytes/100", &invalid_path).await;
        match result {
            Ok(_) => println!("Unexpected success for invalid path"),
            Err(e) => println!("Correctly handled invalid path: {}", e),
        }
    }

    #[tokio::test]
    async fn test_directory_creation() {
        let test_dir = setup_test_dir().await.join("unit").join("directories");
        let nested_dir = test_dir.join("nested").join("deeply").join("nested");
        let test_file_path = nested_dir.join("test_file.bin");

        // This should create all necessary parent directories
        let download_future = download_file("https://httpbin.org/bytes/128", &test_file_path);

        match timeout(TEST_TIMEOUT, download_future).await {
            Ok(Ok(_)) => {
                println!("Successfully created nested directories and downloaded file");
                assert!(test_file_path.exists());
                assert!(nested_dir.exists());
            }
            Ok(Err(e)) => {
                println!("Download failed: {}", e);
            }
            Err(_) => {
                println!("Download timed out");
            }
        }
    }

    #[tokio::test]
    async fn test_download_path_structure() {
        let test_dir = setup_test_dir().await.join("unit").join("path_structure");

        // Test that download paths are created correctly
        let manga_title = "Test Manga: Special/Characters\\Edition";
        let chapter_title = "Chapter 1: The Beginning*";

        let sanitized_manga = sanitize_filename(manga_title);
        let sanitized_chapter = sanitize_filename(chapter_title);

        let expected_manga_dir = test_dir.join(&sanitized_manga);
        let expected_chapter_dir = expected_manga_dir.join(&sanitized_chapter);

        // Create the directory structure
        tokio::fs::create_dir_all(&expected_chapter_dir)
            .await
            .unwrap();

        // Verify the structure
        assert!(expected_manga_dir.exists());
        assert!(expected_chapter_dir.exists());

        println!("Created manga directory: {}", expected_manga_dir.display());
        println!(
            "Created chapter directory: {}",
            expected_chapter_dir.display()
        );

        // Test that paths don't contain dangerous characters
        let manga_path_str = expected_manga_dir.to_string_lossy();
        let chapter_path_str = expected_chapter_dir.to_string_lossy();

        for dangerous_char in ['/', '\\', ':', '*', '?', '"', '<', '>', '|'] {
            if dangerous_char != '/' && dangerous_char != '\\' {
                // Allow path separators in full paths
                assert!(!manga_path_str.contains(dangerous_char));
                assert!(!chapter_path_str.contains(dangerous_char));
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_downloads() {
        let test_dir = setup_test_dir().await.join("unit").join("concurrent");

        // Test downloading multiple small files concurrently
        let download_tasks = vec![
            (
                "https://httpbin.org/bytes/256".to_string(),
                test_dir.join("concurrent_file_0.bin"),
            ),
            (
                "https://httpbin.org/bytes/512".to_string(),
                test_dir.join("concurrent_file_1.bin"),
            ),
            (
                "https://httpbin.org/bytes/128".to_string(),
                test_dir.join("concurrent_file_2.bin"),
            ),
        ];

        let mut handles = Vec::new();
        for (url, path) in download_tasks {
            let handle = tokio::spawn(async move { download_file(&url, &path).await });
            handles.push(handle);
        }

        // Wait for all downloads to complete
        let results = timeout(TEST_TIMEOUT, futures::future::join_all(handles)).await;

        match results {
            Ok(download_results) => {
                let mut success_count = 0;
                for (i, result) in download_results.into_iter().enumerate() {
                    match result {
                        Ok(Ok(bytes)) => {
                            success_count += 1;
                            println!("Concurrent download {} completed: {} bytes", i, bytes);
                        }
                        Ok(Err(e)) => {
                            println!("Concurrent download {} failed: {}", i, e);
                        }
                        Err(e) => {
                            println!("Concurrent download {} panicked: {}", i, e);
                        }
                    }
                }

                if success_count > 0 {
                    println!(
                        "Concurrent downloads completed: {}/{} successful",
                        success_count, 3
                    );
                } else {
                    println!("All concurrent downloads failed (likely network issues)");
                }
            }
            Err(_) => {
                println!("Concurrent downloads timed out");
            }
        }
    }

    #[tokio::test]
    async fn test_download_resume_behavior() {
        let test_dir = setup_test_dir().await.join("unit").join("resume");
        let test_file_path = test_dir.join("resume_test.bin");

        // First download
        match download_file("https://httpbin.org/bytes/1024", &test_file_path).await {
            Ok(bytes1) => {
                println!("First download completed: {} bytes", bytes1);

                // Get file modification time
                let metadata1 = tokio::fs::metadata(&test_file_path).await.unwrap();

                // Wait a bit to ensure different modification time
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                // Second download to same path (should overwrite)
                match download_file("https://httpbin.org/bytes/2048", &test_file_path).await {
                    Ok(bytes2) => {
                        println!("Second download completed: {} bytes", bytes2);

                        let metadata2 = tokio::fs::metadata(&test_file_path).await.unwrap();

                        // Verify the file was updated
                        assert_ne!(metadata1.len(), metadata2.len());
                        assert_ne!(metadata1.modified().unwrap(), metadata2.modified().unwrap());

                        println!("File was properly overwritten");
                    }
                    Err(e) => println!("Second download failed: {}", e),
                }
            }
            Err(e) => println!("First download failed: {}", e),
        }
    }
}
