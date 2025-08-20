//! JSON parsing and extraction utilities for manga API responses.
//!
//! This module provides convenient functions for extracting data from nested JSON
//! structures commonly found in manga website APIs. It supports dot notation for
//! navigating nested objects and arrays.
//!
//! # Examples
//!
//! ```rust
//! use tosho::net::json;
//! use serde_json::json;
//!
//! let data = json!({
//!     "manga": {
//!         "title": "One Piece",
//!         "author": "Oda Eiichiro",
//!         "chapters": [
//!             {"number": 1, "title": "Romance Dawn"},
//!             {"number": 2, "title": "They Call Him Straw Hat Luffy"}
//!         ]
//!     }
//! });
//!
//! let title = json::extract_path(&data, "manga.title").unwrap();
//! let chapters = json::extract_array(&data, "manga.chapters");
//! ```

use serde::de::DeserializeOwned;
use serde_json::Value;

/// Extracts a value from nested JSON using dot notation.
///
/// This function navigates through nested JSON objects using a dot-separated path.
/// It's useful for extracting values from deeply nested API responses without
/// having to manually traverse each level.
///
/// # Parameters
///
/// * `json` - The JSON value to search in
/// * `path` - Dot-separated path to the desired value (e.g., "manga.title", "data.chapters.0.name")
///
/// # Returns
///
/// * `Some(Value)` - The value at the specified path if found
/// * `None` - If any part of the path doesn't exist
///
/// # Examples
///
/// ```rust
/// use tosho::net::json;
/// use serde_json::json;
///
/// let data = json!({
///     "response": {
///         "manga": {
///             "title": "One Piece",
///             "id": 123
///         }
///     }
/// });
///
/// let title = json::extract_path(&data, "response.manga.title");
/// assert_eq!(title.unwrap().as_str(), Some("One Piece"));
///
/// let missing = json::extract_path(&data, "response.manga.author");
/// assert_eq!(missing, None);
/// ```
pub fn extract_path(json: &Value, path: &str) -> Option<Value> {
    let mut current = json;

    for key in path.split('.') {
        current = current.get(key)?;
    }

    Some(current.clone())
}

/// Extracts and deserializes a value from a nested JSON path.
///
/// This function combines path extraction with JSON deserialization, allowing you
/// to extract and convert values in a single step. It's particularly useful for
/// extracting typed data from API responses.
///
/// # Type Parameters
///
/// * `T` - The type to deserialize the value into (must implement `DeserializeOwned`)
///
/// # Parameters
///
/// * `json` - The JSON value to search in
/// * `path` - Dot-separated path to the desired value
///
/// # Returns
///
/// The deserialized value of type `T` on success.
///
/// # Errors
///
/// * [`Error::Parse`](crate::Error::Parse) - If the path doesn't exist
/// * [`Error::Json`](crate::Error::Json) - If deserialization fails
///
/// # Examples
///
/// ```rust
/// use tosho::net::json;
/// use serde_json::json;
///
/// let data = json!({
///     "manga": {
///         "id": 123,
///         "title": "One Piece",
///         "rating": 9.5
///     }
/// });
///
/// let id: u32 = json::extract_as(&data, "manga.id").unwrap();
/// let title: String = json::extract_as(&data, "manga.title").unwrap();
/// let rating: f64 = json::extract_as(&data, "manga.rating").unwrap();
/// ```
pub fn extract_as<T>(json: &Value, path: &str) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    extract_path(json, path)
        .ok_or_else(|| crate::Error::parse(format!("Path not found: {}", path)))
        .and_then(|v| serde_json::from_value(v).map_err(Into::into))
}

/// Extracts an array from a nested JSON path.
///
/// This function finds an array at the specified path and returns its elements
/// as a vector of JSON values. If the path doesn't exist or doesn't point to
/// an array, an empty vector is returned.
///
/// # Parameters
///
/// * `json` - The JSON value to search in
/// * `path` - Dot-separated path to the desired array
///
/// # Returns
///
/// A vector of JSON values from the array. Returns an empty vector if:
/// - The path doesn't exist
/// - The value at the path is not an array
/// - The array is empty
///
/// # Examples
///
/// ```rust
/// use tosho::net::json;
/// use serde_json::json;
///
/// let data = json!({
///     "response": {
///         "chapters": [
///             {"id": 1, "title": "Chapter 1"},
///             {"id": 2, "title": "Chapter 2"},
///             {"id": 3, "title": "Chapter 3"}
///         ],
///         "tags": ["Action", "Adventure", "Shounen"]
///     }
/// });
///
/// let chapters = json::extract_array(&data, "response.chapters");
/// assert_eq!(chapters.len(), 3);
///
/// let tags = json::extract_array(&data, "response.tags");
/// assert_eq!(tags.len(), 3);
///
/// let missing = json::extract_array(&data, "response.missing");
/// assert_eq!(missing.len(), 0);
/// ```
pub fn extract_array(json: &Value, path: &str) -> Vec<Value> {
    extract_path(json, path)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}
