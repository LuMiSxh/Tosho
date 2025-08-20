//! HTML parsing utilities for manga sources.
//!
//! This module provides convenient functions for parsing HTML content from manga websites.
//! It uses the `scraper` crate for CSS selector-based parsing and `rayon` for parallel
//! processing of multiple elements.
//!
//! # Examples
//!
//! ```rust
//! use tosho::net::html;
//! use scraper::Html;
//!
//! let html_content = r#"
//!     <div class="manga-item">
//!         <h3 class="title">One Piece</h3>
//!         <img src="cover.jpg" alt="Cover">
//!         <span class="author">Oda Eiichiro</span>
//!     </div>
//! "#;
//!
//! let document = html::parse(html_content);
//! let title = html::select_text(&document, ".title").unwrap();
//! let cover_url = html::select_attr(&document, "img", "src").unwrap();
//! ```

use rayon::prelude::*;
use scraper::{Html, Selector};

/// Parses an HTML document from a string.
///
/// This function creates a scraper `Html` document that can be used with other
/// functions in this module for CSS selector-based parsing.
///
/// # Parameters
///
/// * `html` - The HTML content as a string
///
/// # Returns
///
/// A parsed `Html` document ready for querying with CSS selectors.
///
/// # Examples
///
/// ```rust
/// use tosho::net::html;
///
/// let html_content = "<div><p>Hello World</p></div>";
/// let document = html::parse(html_content);
/// ```
pub fn parse(html: &str) -> Html {
    Html::parse_document(html)
}

/// Extracts text content from the first element matching a CSS selector.
///
/// This function finds the first element matching the given CSS selector and
/// returns its combined text content, with whitespace trimmed.
///
/// # Parameters
///
/// * `html` - The parsed HTML document
/// * `selector` - CSS selector string (e.g., ".title", "#content", "h1")
///
/// # Returns
///
/// * `Some(String)` - The text content if an element was found
/// * `None` - If no element matches the selector or the selector is invalid
///
/// # Examples
///
/// ```rust
/// use tosho::net::html;
///
/// let document = html::parse(r#"<h1 class="title">One Piece</h1>"#);
/// let title = html::select_text(&document, ".title");
/// assert_eq!(title, Some("One Piece".to_string()));
/// ```
pub fn select_text(html: &Html, selector: &str) -> Option<String> {
    Selector::parse(selector).ok().and_then(|sel| {
        html.select(&sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
    })
}

/// Extracts an attribute value from the first element matching a CSS selector.
///
/// This function finds the first element matching the given CSS selector and
/// returns the value of the specified attribute.
///
/// # Parameters
///
/// * `html` - The parsed HTML document
/// * `selector` - CSS selector string
/// * `attr` - The attribute name to extract (e.g., "src", "href", "data-id")
///
/// # Returns
///
/// * `Some(String)` - The attribute value if found
/// * `None` - If no element matches, the selector is invalid, or the attribute doesn't exist
///
/// # Examples
///
/// ```rust
/// use tosho::net::html;
///
/// let document = html::parse(r#"<img src="cover.jpg" alt="Cover">"#);
/// let src = html::select_attr(&document, "img", "src");
/// assert_eq!(src, Some("cover.jpg".to_string()));
/// ```
pub fn select_attr(html: &Html, selector: &str, attr: &str) -> Option<String> {
    Selector::parse(selector).ok().and_then(|sel| {
        html.select(&sel)
            .next()
            .and_then(|el| el.value().attr(attr).map(String::from))
    })
}

/// Extracts text content from all elements matching a CSS selector.
///
/// This function finds all elements matching the given CSS selector and
/// returns their text content as a vector of strings.
///
/// # Parameters
///
/// * `html` - The parsed HTML document
/// * `selector` - CSS selector string
///
/// # Returns
///
/// A vector of strings containing the text content of all matching elements.
/// Returns an empty vector if no elements match or the selector is invalid.
///
/// # Examples
///
/// ```rust
/// use tosho::net::html;
///
/// let document = html::parse(r#"
///     <ul>
///         <li class="tag">Action</li>
///         <li class="tag">Adventure</li>
///         <li class="tag">Shounen</li>
///     </ul>
/// "#);
/// let tags = html::select_all_text(&document, ".tag");
/// assert_eq!(tags, vec!["Action", "Adventure", "Shounen"]);
/// ```
pub fn select_all_text(html: &Html, selector: &str) -> Vec<String> {
    Selector::parse(selector)
        .ok()
        .map(|sel| {
            html.select(&sel)
                .map(|el| el.text().collect::<String>().trim().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Extracts attribute values from all elements matching a CSS selector.
///
/// This function finds all elements matching the given CSS selector and
/// returns the values of the specified attribute as a vector of strings.
///
/// # Parameters
///
/// * `html` - The parsed HTML document
/// * `selector` - CSS selector string
/// * `attr` - The attribute name to extract
///
/// # Returns
///
/// A vector of strings containing the attribute values of all matching elements.
/// Returns an empty vector if no elements match, the selector is invalid, or
/// no elements have the specified attribute.
///
/// # Examples
///
/// ```rust
/// use tosho::net::html;
///
/// let document = html::parse(r#"
///     <div class="chapter-list">
///         <a href="/chapter/1">Chapter 1</a>
///         <a href="/chapter/2">Chapter 2</a>
///         <a href="/chapter/3">Chapter 3</a>
///     </div>
/// "#);
/// let links = html::select_all_attr(&document, "a", "href");
/// assert_eq!(links, vec!["/chapter/1", "/chapter/2", "/chapter/3"]);
/// ```
pub fn select_all_attr(html: &Html, selector: &str, attr: &str) -> Vec<String> {
    Selector::parse(selector)
        .ok()
        .map(|sel| {
            html.select(&sel)
                .filter_map(|el| el.value().attr(attr).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Parses manga items from HTML in parallel using rayon.
///
/// This function is optimized for parsing large lists of manga items by processing
/// them in parallel. It finds all elements matching the selector, converts them to
/// HTML strings, and processes them concurrently using rayon's parallel iterator.
///
/// # Type Parameters
///
/// * `F` - A function that takes an `ElementRef` and returns an optional `Manga`
///
/// # Parameters
///
/// * `html` - The parsed HTML document
/// * `selector` - CSS selector for manga item containers (e.g., ".manga-item", ".search-result")
/// * `parser` - Function to extract manga data from each element
///
/// # Returns
///
/// A vector of `Manga` objects parsed from the matching elements. Elements that
/// fail to parse (return `None` from the parser function) are filtered out.
///
/// # Examples
///
/// ```rust
/// use tosho::net::html;
/// use tosho::types::Manga;
///
/// let document = html::parse(r#"
///     <div class="manga-list">
///         <div class="manga-item">
///             <h3>One Piece</h3>
///             <span class="author">Oda</span>
///         </div>
///         <div class="manga-item">
///             <h3>Naruto</h3>
///             <span class="author">Kishimoto</span>
///         </div>
///     </div>
/// "#);
///
/// let manga_list = html::parse_manga_items(&document, ".manga-item", |element| {
///     let title = html::select_text(&html::parse(&element.html()), "h3")?;
///     let author = html::select_text(&html::parse(&element.html()), ".author")?;
///
///     Some(Manga {
///         id: title.clone(),
///         title,
///         authors: vec![author],
///         source_id: "example".to_string(),
///         // ... other fields
/// #       cover_url: None,
/// #       description: None,
/// #       tags: vec![],
///     })
/// });
/// ```
///
/// # Performance
///
/// This function uses rayon's parallel iterators to process manga items concurrently,
/// which can significantly improve performance when parsing large lists of items.
/// The elements are first collected into HTML strings to avoid borrowing issues
/// with parallel processing.
pub fn parse_manga_items<F>(html: &Html, selector: &str, parser: F) -> Vec<crate::Manga>
where
    F: Fn(scraper::ElementRef) -> Option<crate::Manga> + Sync,
{
    Selector::parse(selector)
        .ok()
        .map(|sel| {
            // Convert ElementRef to HTML strings which can be processed in parallel
            let elements: Vec<String> = html.select(&sel).map(|el| el.html()).collect();

            // Parse in parallel with rayon
            elements
                .into_par_iter()
                .filter_map(|html_str| {
                    let doc = Html::parse_fragment(&html_str);
                    let element = doc.root_element();
                    parser(element)
                })
                .collect()
        })
        .unwrap_or_default()
}
