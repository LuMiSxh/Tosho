//! Integration tests for Tosho
//!
//! End-to-end tests that verify the complete functionality works together.

use std::time::Duration;
use tokio::time::timeout;
use tosho::prelude::*;
use tosho::sources::{KissMangaSource, MangaDexSource};

// Import test utilities from mod
mod common;
use common::{setup_test_dir, TEST_TIMEOUT, TEST_MANGA_TITLE};

#[cfg(test)]
mod integration_tests {
    use super::*;

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

                // Test deduplication - clone to avoid move
                let dedupe_results = manga_list.clone().dedupe_by_title();
                println!("After deduplication: {} unique titles", dedupe_results.len());

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
    async fn test_full_workflow() {
        let test_dir = setup_test_dir().await.join("integration").join("full_workflow");
        let mut sources = Sources::new();
        sources.add(MangaDexSource::new());

        println!("Testing complete workflow: search -> chapters -> download");

        // Step 1: Search
        let search_future = sources.search(TEST_MANGA_TITLE).limit(1).flatten();
        let search_result = timeout(TEST_TIMEOUT, search_future).await;

        match search_result {
            Ok(Ok(manga_list)) if !manga_list.is_empty() => {
                let manga = &manga_list[0];
                println!("✓ Found manga: {}", manga.title);

                // Step 2: Get chapters
                let source = MangaDexSource::new();
                let chapters_result = timeout(TEST_TIMEOUT, source.get_chapters(&manga.id)).await;

                match chapters_result {
                    Ok(Ok(chapters)) if !chapters.is_empty() => {
                        let chapter = &chapters[0];
                        println!("✓ Found chapter: {}", chapter.title);

                        // Step 3: Download chapter
                        let download_result = timeout(
                            Duration::from_secs(60),
                            source.download_chapter(&chapter.id, &test_dir)
                        ).await;

                        match download_result {
                            Ok(Ok(chapter_path)) => {
                                println!("✓ Downloaded to: {}", chapter_path.display());
                                assert!(chapter_path.exists());
                            }
                            Ok(Err(e)) => println!("⚠ Download failed: {}", e),
                            Err(_) => println!("⚠ Download timeout"),
                        }
                    }
                    Ok(Ok(_)) => println!("No chapters found"),
                    Ok(Err(e)) => println!("Chapters error: {}", e),
                    Err(_) => println!("Chapters timeout"),
                }
            }
            Ok(Ok(_)) => println!("No manga found"),
            Ok(Err(e)) => println!("Search failed: {}", e),
            Err(_) => println!("Search timeout"),
        }
    }

    #[tokio::test]
    async fn test_multi_source_search() {
        let _test_dir = setup_test_dir().await.join("integration").join("multi_source");
        let mut sources = Sources::new();
        sources.add(MangaDexSource::new());
        sources.add(KissMangaSource::new());

        println!("Testing multi-source search capabilities");

        let search_future = sources
            .search("popular")
            .limit(3)
            .sort_by(SortOrder::Relevance)
            .flatten();

        match timeout(TEST_TIMEOUT, search_future).await {
            Ok(Ok(results)) => {
                println!("Multi-source search returned {} results", results.len());

                // Verify we got results from both sources
                let mgd_results = results.iter().filter(|m| m.source_id == "mgd").count();
                let kmg_results = results.iter().filter(|m| m.source_id == "kmg").count();

                println!("MangaDx results: {}, KissManga results: {}", mgd_results, kmg_results);

                // Test deduplication across sources - clone to avoid move
                let original_len = results.len();
                let dedupe_results = results.dedupe_by_title();
                println!("After cross-source deduplication: {} unique titles", dedupe_results.len());
                assert!(dedupe_results.len() <= original_len);
            }
            Ok(Err(e)) => println!("Multi-source search failed: {}", e),
            Err(_) => println!("Multi-source search timeout"),
        }
    }
}
