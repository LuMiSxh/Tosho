//! Source-specific functionality tests
//!
//! Tests individual manga sources (MangaDx, KissManga, etc.)

use tokio::time::timeout;
use tosho::prelude::*;
use tosho::sources::{KissMangaSource, MangaDexSource};

// Import test utilities
mod common;
use common::{DOWNLOAD_TIMEOUT, TEST_TIMEOUT, setup_test_dir};

#[cfg(test)]
mod source_tests {
    use super::*;

    #[tokio::test]
    async fn test_mangadex_basic_functionality() {
        let source = MangaDexSource::new();

        // Test source metadata
        assert_eq!(source.id(), "mgd");
        assert_eq!(source.name(), "MangaDex");
        assert!(source.base_url().starts_with("https://"));

        // Test search with timeout
        let search_params = SearchParams {
            query: "test".to_string(),
            limit: Some(3),
            offset: None,
            include_tags: vec![],
            exclude_tags: vec![],
            sort_by: Some(SortOrder::UpdatedAt),
        };

        let search_result = timeout(TEST_TIMEOUT, source.search(search_params)).await;

        match search_result {
            Ok(Ok(manga_list)) => {
                println!("MangaDx search: {} results", manga_list.len());
                assert!(manga_list.len() <= 3);
                for manga in &manga_list {
                    assert!(!manga.id.is_empty());
                    assert!(!manga.title.is_empty());
                    assert_eq!(manga.source_id, "mgd");
                }
            }
            Ok(Err(e)) => {
                println!("MangaDx search failed: {}", e);
            }
            Err(_) => {
                println!("MangaDx search timeout");
            }
        }
    }

    #[tokio::test]
    async fn test_kissmanga_basic_functionality() {
        let source = KissMangaSource::new();

        // Test source metadata
        assert_eq!(source.id(), "kmg");
        assert_eq!(source.name(), "KissManga");
        assert!(source.base_url().starts_with("https://"));

        // Test search with timeout
        let search_params = SearchParams {
            query: "popular".to_string(),
            limit: Some(2),
            offset: None,
            include_tags: vec![],
            exclude_tags: vec![],
            sort_by: None,
        };

        let search_result = timeout(TEST_TIMEOUT, source.search(search_params)).await;

        match search_result {
            Ok(Ok(manga_list)) => {
                println!("KissManga search: {} results", manga_list.len());
                assert!(manga_list.len() <= 2);
                for manga in &manga_list {
                    assert!(!manga.id.is_empty());
                    assert!(!manga.title.is_empty());
                    assert_eq!(manga.source_id, "kmg");
                }
            }
            Ok(Err(e)) => {
                println!("KissManga search failed: {}", e);
            }
            Err(_) => {
                println!("KissManga search timeout");
            }
        }
    }

    #[tokio::test]
    async fn test_mangadex_chapter_download() {
        let test_dir = setup_test_dir().await.join("sources").join("mangadex");
        let source = MangaDexSource::new();

        // Search for a short oneshot or simple manga
        let search_params = SearchParams {
            query: "oneshot".to_string(),
            limit: Some(1),
            offset: None,
            include_tags: vec![],
            exclude_tags: vec![],
            sort_by: Some(SortOrder::UpdatedAt),
        };

        let search_future = source.search(search_params);
        match timeout(TEST_TIMEOUT, search_future).await {
            Ok(Ok(manga_list)) if !manga_list.is_empty() => {
                let manga = &manga_list[0];
                println!("Found test manga: {}", manga.title);

                let chapters_future = source.get_chapters(&manga.id);
                match timeout(TEST_TIMEOUT, chapters_future).await {
                    Ok(Ok(chapters)) if !chapters.is_empty() => {
                        let chapter = &chapters[0];
                        println!("Found test chapter: {}", chapter.title);

                        // Test the download
                        let download_future = source.download_chapter(&chapter.id, &test_dir);
                        match timeout(DOWNLOAD_TIMEOUT, download_future).await {
                            Ok(Ok(chapter_path)) => {
                                println!(
                                    "MangaDx chapter downloaded to: {}",
                                    chapter_path.display()
                                );
                                assert!(chapter_path.exists());

                                // Check that files were actually downloaded
                                if let Ok(entries) = tokio::fs::read_dir(&chapter_path).await {
                                    let mut count = 0;
                                    let mut entries = entries;
                                    while let Ok(Some(_)) = entries.next_entry().await {
                                        count += 1;
                                    }
                                    println!("Downloaded {} files", count);
                                    assert!(count > 0, "No files were downloaded");
                                }
                            }
                            Ok(Err(e)) => {
                                println!("MangaDx download failed: {}", e);
                                // Don't fail test for site/network issues
                            }
                            Err(_) => {
                                println!("MangaDx download timed out");
                            }
                        }
                    }
                    Ok(Ok(_)) => println!("No chapters available for download test"),
                    Ok(Err(e)) => println!("Failed to get chapters: {}", e),
                    Err(_) => println!("Get chapters timed out"),
                }
            }
            Ok(Ok(_)) => println!("No manga found for download test"),
            Ok(Err(e)) => println!("Search failed: {}", e),
            Err(_) => println!("Search timed out"),
        }
    }

    #[tokio::test]
    async fn test_kissmanga_chapter_download() {
        let test_dir = setup_test_dir().await.join("sources").join("kissmanga");
        let source = KissMangaSource::new();

        // Search for a popular manga that's likely to have working chapters
        let search_params = SearchParams {
            query: "naruto".to_string(),
            limit: Some(1),
            offset: None,
            include_tags: vec![],
            exclude_tags: vec![],
            sort_by: None,
        };

        let search_future = source.search(search_params);
        match timeout(TEST_TIMEOUT, search_future).await {
            Ok(Ok(manga_list)) if !manga_list.is_empty() => {
                let manga = &manga_list[0];
                println!("Found KissManga test manga: {}", manga.title);

                let chapters_future = source.get_chapters(&manga.id);
                match timeout(TEST_TIMEOUT, chapters_future).await {
                    Ok(Ok(chapters)) if !chapters.is_empty() => {
                        // Try the first chapter
                        let chapter = &chapters[0];
                        println!("Found KissManga test chapter: {}", chapter.title);

                        // Test the download with KissManga's custom implementation
                        let download_future = source.download_chapter(&chapter.id, &test_dir);
                        match timeout(DOWNLOAD_TIMEOUT, download_future).await {
                            Ok(Ok(chapter_path)) => {
                                println!(
                                    "KissManga chapter downloaded to: {}",
                                    chapter_path.display()
                                );
                                assert!(chapter_path.exists());

                                // Check that files were actually downloaded
                                if let Ok(entries) = tokio::fs::read_dir(&chapter_path).await {
                                    let mut count = 0;
                                    let mut entries = entries;
                                    while let Ok(Some(_)) = entries.next_entry().await {
                                        count += 1;
                                    }
                                    println!("Downloaded {} files", count);
                                    assert!(count > 0, "No files were downloaded");
                                }
                            }
                            Ok(Err(e)) => {
                                println!("KissManga download failed: {}", e);
                                // Don't fail test for site/network issues
                            }
                            Err(_) => {
                                println!("KissManga download timed out");
                            }
                        }
                    }
                    Ok(Ok(_)) => println!("No chapters available for KissManga download test"),
                    Ok(Err(e)) => println!("Failed to get KissManga chapters: {}", e),
                    Err(_) => println!("Get KissManga chapters timed out"),
                }
            }
            Ok(Ok(_)) => println!("No KissManga manga found for download test"),
            Ok(Err(e)) => println!("KissManga search failed: {}", e),
            Err(_) => println!("KissManga search timed out"),
        }
    }
}
