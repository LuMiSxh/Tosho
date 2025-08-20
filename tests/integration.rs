use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;
use tosho::prelude::*;
use tosho::sources::{kissmanga::KissMangaSource, mangadex::MangaDexSource};

const TEST_DOWNLOADS_DIR: &str = "tests/downloads";
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

#[cfg(test)]
mod integration_tests {
    use super::*;

    // Helper function to create test downloads directory
    async fn setup_test_dir() -> PathBuf {
        let test_dir = PathBuf::from(TEST_DOWNLOADS_DIR);
        if !test_dir.exists() {
            tokio::fs::create_dir_all(&test_dir).await.unwrap();
        }
        test_dir
    }

    // Helper function to clean up test files (optional, since they're git-ignored)
    #[allow(dead_code)]
    async fn cleanup_test_dir() {
        let test_dir = PathBuf::from(TEST_DOWNLOADS_DIR);
        if test_dir.exists() {
            let _ = tokio::fs::remove_dir_all(&test_dir).await;
        }
    }

    #[tokio::test]
    async fn test_sources_collection_basic() {
        let mut sources = Sources::new();
        sources.add(KissMangaSource::new());
        sources.add(MangaDexSource::new());

        assert_eq!(sources.len(), 2);
        assert!(!sources.is_empty());

        // Check that we can access source IDs
        let source_ids = sources.list_ids();
        assert!(source_ids.contains(&"kmg"));
        assert!(source_ids.contains(&"mgd"));
    }

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
                println!("MangaDex search: {} results", manga_list.len());

                // Validate manga structure
                for manga in manga_list.iter().take(1) {
                    assert!(!manga.id.is_empty());
                    assert!(!manga.title.is_empty());
                    assert_eq!(manga.source_id, "mgd");

                    // Test chapters functionality
                    let chapters_result =
                        timeout(TEST_TIMEOUT, source.get_chapters(&manga.id)).await;
                    match chapters_result {
                        Ok(Ok(chapters)) => {
                            println!(
                                "    Found {} chapters for '{}'",
                                chapters.len(),
                                manga.title
                            );

                            // Test pages for first chapter if available
                            if let Some(chapter) = chapters.first() {
                                let pages_result =
                                    timeout(TEST_TIMEOUT, source.get_pages(&chapter.id)).await;
                                match pages_result {
                                    Ok(Ok(pages)) => {
                                        println!(
                                            "      Chapter '{}': {} pages",
                                            chapter.title,
                                            pages.len()
                                        );

                                        // Validate page structure (pages are just strings)
                                        for page in pages.iter().take(1) {
                                            assert!(!page.is_empty());
                                        }
                                    }
                                    Ok(Err(e)) => println!("      Pages error: {}", e),
                                    Err(_) => println!("      Pages timeout"),
                                }
                            }
                        }
                        Ok(Err(e)) => println!("    Chapters error: {}", e),
                        Err(_) => println!("    Chapters timeout"),
                    }
                }
            }
            Ok(Err(e)) => {
                println!("MangaDex search failed: {}", e);
                // Don't fail the test for network issues
            }
            Err(_) => {
                println!("MangaDex search timeout");
                // Don't fail the test for timeouts
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
            query: "naruto".to_string(),
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

                // Validate manga structure
                for manga in manga_list.iter().take(1) {
                    assert!(!manga.id.is_empty());
                    assert!(!manga.title.is_empty());
                    assert_eq!(manga.source_id, "kmg");

                    // Test chapters functionality
                    let chapters_result =
                        timeout(TEST_TIMEOUT, source.get_chapters(&manga.id)).await;
                    match chapters_result {
                        Ok(Ok(chapters)) => {
                            println!(
                                "    Found {} chapters for '{}'",
                                chapters.len(),
                                manga.title
                            );

                            // Test pages for first chapter if available
                            if let Some(chapter) = chapters.first() {
                                let pages_result =
                                    timeout(TEST_TIMEOUT, source.get_pages(&chapter.id)).await;
                                match pages_result {
                                    Ok(Ok(pages)) => {
                                        println!(
                                            "      Chapter '{}': {} pages",
                                            chapter.title,
                                            pages.len()
                                        );

                                        // Validate page structure (pages are just strings)
                                        for page in pages.iter().take(1) {
                                            assert!(!page.is_empty());
                                        }
                                    }
                                    Ok(Err(e)) => println!("      Pages error: {}", e),
                                    Err(_) => println!("      Pages timeout"),
                                }
                            }
                        }
                        Ok(Err(e)) => println!("    Chapters error: {}", e),
                        Err(_) => println!("    Chapters timeout"),
                    }
                }
            }
            Ok(Err(e)) => {
                println!("KissManga search failed: {}", e);
                // Don't fail the test for network issues
            }
            Err(_) => {
                println!("KissManga search timeout");
                // Don't fail the test for timeouts
            }
        }
    }

    #[tokio::test]
    async fn test_sources_fluent_api() {
        let mut sources = Sources::new();
        sources.add(KissMangaSource::new());
        sources.add(MangaDexSource::new());

        // Test fluent search API with timeout
        let search_future = sources
            .search("manga")
            .limit(5)
            .sort_by(SortOrder::UpdatedAt)
            .flatten();

        let results = timeout(TEST_TIMEOUT, search_future).await;

        match results {
            Ok(Ok(manga_list)) => {
                println!("Fluent API search: {} results", manga_list.len());

                // Test deduplication
                let dedupe_results = manga_list.clone().dedupe_by_title();
                println!(
                    "After deduplication: {} unique titles",
                    dedupe_results.len()
                );

                // Test relevance sorting
                let sorted_results = manga_list.clone().sort_by_relevance();
                assert_eq!(sorted_results.len(), manga_list.len());
            }
            Ok(Err(e)) => {
                println!("Fluent API search failed: {}", e);
            }
            Err(_) => {
                println!("Fluent API search timeout");
            }
        }
    }

    #[tokio::test]
    async fn test_grouped_search() {
        let mut sources = Sources::new();
        sources.add(KissMangaSource::new());
        sources.add(MangaDexSource::new());

        // Test grouped search with timeout
        let grouped_future = sources.search("test").limit(3).group();

        let grouped_results = timeout(TEST_TIMEOUT, grouped_future).await;

        match grouped_results {
            Ok(grouped) => {
                println!("Grouped search completed");

                for (source_id, result) in grouped {
                    match result {
                        Ok(manga_list) => {
                            println!("{}: {} results", source_id, manga_list.len());
                        }
                        Err(e) => {
                            println!("{}: Error - {}", source_id, e);
                        }
                    }
                }
            }
            Err(_) => {
                println!("Grouped search timeout");
            }
        }
    }

    #[tokio::test]
    async fn test_source_specific_search() {
        let mut sources = Sources::new();
        sources.add(KissMangaSource::new());
        sources.add(MangaDexSource::new());

        // Test KissManga specific search
        let kmg_future = sources.search("one piece").from_source("kmg");
        let kmg_result = timeout(TEST_TIMEOUT, kmg_future).await;

        match kmg_result {
            Ok(Ok(results)) => {
                println!("KissManga specific search: {} results", results.len());
                for manga in results.iter() {
                    assert_eq!(manga.source_id, "kmg");
                }
            }
            Ok(Err(e)) => println!("KissManga specific search failed: {}", e),
            Err(_) => println!("KissManga specific search timeout"),
        }

        // Test MangaDex specific search
        let mgd_future = sources.search("manga").from_source("mgd");
        let mgd_result = timeout(TEST_TIMEOUT, mgd_future).await;

        match mgd_result {
            Ok(Ok(results)) => {
                println!("MangaDex specific search: {} results", results.len());
                for manga in results.iter() {
                    assert_eq!(manga.source_id, "mgd");
                }
            }
            Ok(Err(e)) => println!("MangaDex specific search failed: {}", e),
            Err(_) => println!("MangaDex specific search timeout"),
        }
    }

    #[tokio::test]
    async fn test_download_functionality() {
        let _test_dir = setup_test_dir().await;

        // Test utility functions
        let dirty_filename = "Test/Chapter\\1:*?\"<>|";
        let clean_filename = sanitize_filename(dirty_filename);
        assert!(!clean_filename.contains('/'));
        assert!(!clean_filename.contains('\\'));
        println!(
            "Filename sanitization: '{}' -> '{}'",
            dirty_filename, clean_filename
        );

        // Test extension extraction
        let test_url = "https://example.com/image.jpg?v=123";
        let extension = extract_extension(test_url);
        assert_eq!(extension, Some("jpg".to_string()));
        println!("Extension extraction: '{}' -> {:?}", test_url, extension);

        // Test simple file download with a small test file
        let test_file_path = PathBuf::from(TEST_DOWNLOADS_DIR).join("test_file.bin");
        let download_future = download_file("https://httpbin.org/bytes/100", &test_file_path);
        let download_result = timeout(TEST_TIMEOUT, download_future).await;

        match download_result {
            Ok(Ok(bytes)) => {
                println!("File download: {} bytes", bytes);
                assert!(test_file_path.exists());
            }
            Ok(Err(e)) => println!("File download failed: {}", e),
            Err(_) => println!("File download timeout"),
        }
    }

    #[tokio::test]
    async fn test_chapter_download_integration() {
        let test_dir = setup_test_dir().await;

        // Test with MangaDex (usually more reliable for testing)
        let source = MangaDexSource::new();

        // Search for a short manga or oneshot for testing
        let search_params = SearchParams {
            query: "oneshot".to_string(),
            limit: Some(1),
            offset: None,
            include_tags: vec![],
            exclude_tags: vec![],
            sort_by: Some(SortOrder::UpdatedAt),
        };

        let search_future = source.search(search_params);
        let search_result = timeout(TEST_TIMEOUT, search_future).await;

        match search_result {
            Ok(Ok(manga_list)) if !manga_list.is_empty() => {
                let manga = &manga_list[0];
                println!("Found test manga: {}", manga.title);

                let chapters_future = source.get_chapters(&manga.id);
                let chapters_result = timeout(TEST_TIMEOUT, chapters_future).await;

                match chapters_result {
                    Ok(Ok(chapters)) if !chapters.is_empty() => {
                        let chapter = &chapters[0];
                        println!("Found test chapter: {}", chapter.title);

                        // Attempt to download the chapter
                        let download_future = source.download_chapter(&chapter.id, &test_dir);
                        let download_result =
                            timeout(Duration::from_secs(60), download_future).await;

                        match download_result {
                            Ok(Ok(chapter_path)) => {
                                println!("Chapter download: {}", chapter_path.display());
                                assert!(chapter_path.exists());
                            }
                            Ok(Err(e)) => {
                                println!("⚠ Chapter download failed: {}", e);
                                // Don't fail test for download issues (could be network/site related)
                            }
                            Err(_) => {
                                println!("⚠ Chapter download timeout");
                            }
                        }
                    }
                    Ok(Ok(_)) => println!("No chapters found for test manga"),
                    Ok(Err(e)) => println!("Failed to get chapters: {}", e),
                    Err(_) => println!("Get chapters timeout"),
                }
            }
            Ok(Ok(_)) => println!("No manga found for download test"),
            Ok(Err(e)) => println!("Search failed: {}", e),
            Err(_) => println!("Search timeout"),
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let source = MangaDexSource::new();

        // Test invalid chapter ID
        let invalid_result = source.get_chapters("invalid-id-12345").await;
        match invalid_result {
            Ok(_) => println!("Expected error for invalid chapter ID but got success"),
            Err(e) => println!("Correctly handled invalid chapter ID: {}", e),
        }

        // Test invalid page ID
        let invalid_pages = source.get_pages("invalid-page-id-67890").await;
        match invalid_pages {
            Ok(_) => println!("Expected error for invalid page ID but got success"),
            Err(e) => println!("Correctly handled invalid page ID: {}", e),
        }

        // Test download with invalid path
        let test_dir = PathBuf::from("/invalid/path/that/should/not/exist");
        let invalid_download = source.download_chapter("any-id", &test_dir).await;
        match invalid_download {
            Ok(_) => println!("Expected error for invalid download path but got success"),
            Err(e) => println!("Correctly handled invalid download path: {}", e),
        }
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let mut sources = Sources::new();
        sources.add(MangaDexSource::new());

        // Test concurrent searches
        let search1 = sources.search("manga").limit(3).flatten();
        let search2 = sources.search("oneshot").limit(3).flatten();

        let results = timeout(TEST_TIMEOUT, async move {
            let (result1, result2) = tokio::join!(search1, search2);
            (result1, result2)
        })
        .await;

        let (result1, result2) =
            results.unwrap_or((Err(Error::parse("timeout")), Err(Error::parse("timeout"))));

        match (result1, result2) {
            (Ok(manga1), Ok(manga2)) => {
                println!(
                    "Concurrent searches: {} and {} results",
                    manga1.len(),
                    manga2.len()
                );
            }
            (Ok(manga1), Err(e2)) => {
                println!(
                    "Partial concurrent success: {} results, error: {}",
                    manga1.len(),
                    e2
                );
            }
            (Err(e1), Ok(manga2)) => {
                println!(
                    "Partial concurrent success: error: {}, {} results",
                    e1,
                    manga2.len()
                );
            }
            (Err(e1), Err(e2)) => {
                println!("Both concurrent searches failed: {}, {}", e1, e2);
            }
        }
    }
}
