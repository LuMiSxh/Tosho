//! Common test utilities and constants
//!
//! Shared functionality used across all test modules.
// Common test utilities and constants - all must be public

use std::path::PathBuf;
use std::time::Duration;

#[allow(dead_code)]
pub const TEST_DOWNLOADS_DIR: &str = "tests/downloads";
#[allow(dead_code)]
pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);
#[allow(dead_code)]
pub const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
#[allow(dead_code)]
pub const TEST_MANGA_TITLE: &str = "The Summer You Were There";

/// Helper function to create test downloads directory
/// Creates the main directory and common subdirectories for organized testing
#[allow(dead_code)]
pub async fn setup_test_dir() -> PathBuf {
    let test_dir = PathBuf::from(TEST_DOWNLOADS_DIR);
    if !test_dir.exists() {
        tokio::fs::create_dir_all(&test_dir).await.unwrap();
    }

    // Create organized subdirectories for different test types
    let subdirs = [
        "unit/basic",
        "unit/errors",
        "unit/directories",
        "unit/path_structure",
        "unit/concurrent",
        "unit/resume",
        "sources/mangadx",
        "sources/kissmanga",
        "integration/full_workflow",
        "integration/multi_source",
    ];

    for subdir in &subdirs {
        let dir_path = test_dir.join(subdir);
        if !dir_path.exists() {
            let _ = tokio::fs::create_dir_all(&dir_path).await;
        }
    }

    test_dir
}

/// Helper function to clean up test files (optional, since they're git-ignored)
/// Removes the entire test downloads directory and all contents
#[allow(dead_code)]
pub async fn cleanup_test_dir() {
    let test_dir = PathBuf::from(TEST_DOWNLOADS_DIR);
    if test_dir.exists() {
        let _ = tokio::fs::remove_dir_all(&test_dir).await;
    }
}

/// Helper function to clean up only a specific test subdirectory
#[allow(dead_code)]
pub async fn cleanup_test_subdir(subdir: &str) {
    let test_dir = PathBuf::from(TEST_DOWNLOADS_DIR).join(subdir);
    if test_dir.exists() {
        let _ = tokio::fs::remove_dir_all(&test_dir).await;
    }
}

/// Helper function to get the size of downloaded test files
#[allow(dead_code)]
pub async fn get_test_dir_size() -> Result<u64, std::io::Error> {
    let test_dir = PathBuf::from(TEST_DOWNLOADS_DIR);
    if !test_dir.exists() {
        return Ok(0);
    }

    let mut total_size = 0u64;
    let mut entries = tokio::fs::read_dir(&test_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            // Recursively calculate directory size with Box::pin to fix recursion
            total_size += Box::pin(calculate_dir_size(&entry.path())).await?;
        }
    }

    Ok(total_size)
}

/// Recursive helper for directory size calculation
/// Uses Box::pin to handle async recursion properly
#[allow(dead_code)]
pub async fn calculate_dir_size(dir: &std::path::Path) -> Result<u64, std::io::Error> {
    let mut total_size = 0u64;
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            // Use Box::pin for async recursion
            total_size += Box::pin(calculate_dir_size(&entry.path())).await?;
        }
    }

    Ok(total_size)
}
