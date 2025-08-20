use std::path::PathBuf;
use tosho::prelude::*;
use tosho::types::SearchParamsBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_builder() {
        let params = SearchParamsBuilder::default()
            .query("test manga".to_string())
            .limit(Some(10))
            .include_tags(vec!["Action".to_string(), "Adventure".to_string()])
            .sort_by(Some(SortOrder::UpdatedAt))
            .build()
            .unwrap();

        assert_eq!(params.query, "test manga");
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.include_tags.len(), 2);
        assert!(params.include_tags.contains(&"Action".to_string()));
        assert!(params.exclude_tags.is_empty());
        assert!(matches!(params.sort_by, Some(SortOrder::UpdatedAt)));
    }

    #[test]
    fn test_manga_struct() {
        let manga = Manga {
            id: "test-id".to_string(),
            title: "Test Manga".to_string(),
            authors: vec!["Author 1".to_string(), "Author 2".to_string()],
            description: Some("A test manga description".to_string()),
            tags: vec!["Action".to_string(), "Adventure".to_string()],
            cover_url: Some("https://example.com/cover.jpg".to_string()),
            source_id: "test".to_string(),
        };

        assert_eq!(manga.id, "test-id");
        assert_eq!(manga.title, "Test Manga");
        assert_eq!(manga.authors.len(), 2);
        assert!(manga.description.is_some());
        assert_eq!(manga.tags.len(), 2);
        assert_eq!(manga.source_id, "test");
        assert!(manga.cover_url.is_some());
    }

    #[test]
    fn test_chapter_struct() {
        let chapter = Chapter {
            id: "chapter-1".to_string(),
            title: "Chapter 1: The Beginning".to_string(),
            number: 1.0,
            pages: vec![
                "https://example.com/page1.jpg".to_string(),
                "https://example.com/page2.jpg".to_string(),
            ],
            manga_id: "test-manga".to_string(),
            source_id: "test".to_string(),
        };

        assert_eq!(chapter.id, "chapter-1");
        assert_eq!(chapter.title, "Chapter 1: The Beginning");
        assert_eq!(chapter.number, 1.0);
        assert_eq!(chapter.pages.len(), 2);
        assert_eq!(chapter.manga_id, "test-manga");
        assert_eq!(chapter.source_id, "test");
    }

    #[test]
    fn test_page_strings() {
        let pages = vec![
            "https://example.com/page1.jpg".to_string(),
            "https://example.com/page2.png".to_string(),
        ];

        assert_eq!(pages.len(), 2);
        assert!(!pages[0].is_empty());
        assert!(!pages[1].is_empty());
        assert!(pages[0].contains("page1"));
        assert!(pages[1].contains("page2"));
    }

    #[test]
    fn test_sort_order_enum() {
        let orders = vec![
            SortOrder::Relevance,
            SortOrder::UpdatedAt,
            SortOrder::CreatedAt,
            SortOrder::Title,
        ];

        assert_eq!(orders.len(), 4);
        assert!(matches!(orders[0], SortOrder::Relevance));
        assert!(matches!(orders[1], SortOrder::UpdatedAt));
    }

    #[test]
    fn test_filename_sanitization() {
        let dirty_filename = "Test/Manga\\Chapter:1*?\"<>|";
        let clean_filename = sanitize_filename(dirty_filename);

        assert!(!clean_filename.contains('/'));
        assert!(!clean_filename.contains('\\'));
        assert!(!clean_filename.contains(':'));
        assert!(!clean_filename.contains('*'));
        assert!(!clean_filename.contains('?'));
        assert!(!clean_filename.contains('"'));
        assert!(!clean_filename.contains('<'));
        assert!(!clean_filename.contains('>'));
        assert!(!clean_filename.contains('|'));

        // Should still contain the basic text
        assert!(clean_filename.contains("Test"));
        assert!(clean_filename.contains("Manga"));
        assert!(clean_filename.contains("Chapter"));
        assert!(clean_filename.contains("1"));
    }

    #[test]
    fn test_extension_extraction() {
        let test_cases = vec![
            ("https://example.com/page.jpg", Some("jpg")),
            ("https://example.com/page.png?v=123", Some("png")),
            ("https://example.com/page.webp#anchor", Some("webp")),
            ("https://example.com/page.jpeg", Some("jpeg")),
            ("https://example.com/page", None),
            ("https://example.com/", None),
            ("", None),
        ];

        for (url, expected) in test_cases {
            let result = extract_extension(url);
            assert_eq!(
                result,
                expected.map(|s| s.to_string()),
                "Extension extraction failed for URL: {}",
                url
            );
        }
    }

    #[test]
    fn test_sources_collection() {
        let mut sources = Sources::new();

        // Initially empty
        assert_eq!(sources.len(), 0);
        assert!(sources.is_empty());

        // Add a source
        sources.add(tosho::sources::mangadex::MangaDexSource::new());
        assert_eq!(sources.len(), 1);
        assert!(!sources.is_empty());

        // Check source IDs
        let ids = sources.list_ids();
        assert!(ids.contains(&"mgd"));
    }

    #[test]
    fn test_manga_list_extensions() {
        let manga_list = vec![
            Manga {
                id: "1".to_string(),
                title: "One Piece".to_string(),
                authors: vec!["Oda".to_string()],
                description: None,
                tags: vec!["Action".to_string()],
                cover_url: None,
                source_id: "test".to_string(),
            },
            Manga {
                id: "2".to_string(),
                title: "Naruto".to_string(),
                authors: vec!["Kishimoto".to_string()],
                description: None,
                tags: vec!["Action".to_string()],
                cover_url: None,
                source_id: "test".to_string(),
            },
            Manga {
                id: "3".to_string(),
                title: "One Piece".to_string(), // Duplicate title
                authors: vec!["Oda".to_string()],
                description: None,
                tags: vec!["Action".to_string()],
                cover_url: None,
                source_id: "test2".to_string(),
            },
        ];

        // Test deduplication by title
        let dedupe_result = manga_list.clone().dedupe_by_title();
        assert_eq!(dedupe_result.len(), 2); // Should remove one "One Piece"

        // Test sorting by relevance (should maintain order when no query)
        let sorted_result = manga_list.clone().sort_by_relevance();
        assert_eq!(sorted_result.len(), 3);
    }

    #[test]
    fn test_error_handling() {
        // Test that our error type can be created and displayed
        let error = Error::parse("Test parse error");
        let error_string = format!("{}", error);
        assert!(error_string.contains("Test parse error"));

        let error = Error::not_found("Test not found error");
        let error_string = format!("{}", error);
        assert!(error_string.contains("Test not found error"));
    }

    #[test]
    fn test_pathbuf_operations() {
        let base_path = PathBuf::from("tests/downloads");
        let manga_path = base_path.join("test-manga");
        let chapter_path = manga_path.join("chapter-1");

        assert!(manga_path.to_string_lossy().contains("test-manga"));
        assert!(chapter_path.to_string_lossy().contains("chapter-1"));
    }

    #[test]
    fn test_search_params_validation() {
        // Test minimum valid params
        let params = SearchParamsBuilder::default()
            .query("test".to_string())
            .build()
            .unwrap();

        assert_eq!(params.query, "test");
        assert_eq!(params.limit, None);
        assert!(params.include_tags.is_empty());
        assert!(params.exclude_tags.is_empty());
        assert_eq!(params.offset, None);
        assert!(params.sort_by.is_none());
    }

    #[test]
    fn test_search_params_from_string() {
        let params: SearchParams = "test query".into();
        assert_eq!(params.query, "test query");
        assert!(params.limit.is_none());

        let params: SearchParams = "another query".to_string().into();
        assert_eq!(params.query, "another query");
        assert!(params.limit.is_none());
    }

    #[test]
    fn test_chapter_decimal_numbers() {
        let chapter = Chapter {
            id: "special".to_string(),
            title: "Chapter 5.5: Special".to_string(),
            number: 5.5,
            pages: vec![],
            manga_id: "test".to_string(),
            source_id: "test".to_string(),
        };

        assert_eq!(chapter.number, 5.5);
        assert!(chapter.title.contains("5.5"));
    }

    #[test]
    fn test_empty_collections() {
        let manga = Manga {
            id: "test".to_string(),
            title: "Test".to_string(),
            authors: vec![],
            description: None,
            tags: vec![],
            cover_url: None,
            source_id: "test".to_string(),
        };

        assert!(manga.authors.is_empty());
        assert!(manga.tags.is_empty());
        assert!(manga.description.is_none());

        let chapter = Chapter {
            id: "test".to_string(),
            title: "Test".to_string(),
            number: 1.0,
            pages: vec![],
            manga_id: "test".to_string(),
            source_id: "test".to_string(),
        };

        assert!(chapter.pages.is_empty());
    }
}
