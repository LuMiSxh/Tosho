# Tosho Tests

This directory contains comprehensive tests for the Tosho manga source library.

## Test Structure

### Unit Tests (`unit_tests.rs`)

- Core data structures and utility functions
- Search parameters, manga/chapter structs validation
- Filename sanitization and extension extraction
- Error handling and basic functionality
- **Run with**: `cargo test --test unit_tests`

### Integration Tests (`integration.rs`)

- Source functionality with real network requests
- Search, chapter retrieval, and page extraction
- Fluent API and source collection features
- Network timeout handling
- **Run with**: `cargo test --test integration`

### Download Tests (`download_tests.rs`)

- File download functionality
- Chapter download integration
- Directory structure creation
- Concurrent downloads and error handling
- **Run with**: `cargo test --test download_tests`

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Test Categories

```bash
# Unit tests only (fast, no network)
cargo test --test unit_tests

# Integration tests (requires network)
cargo test --test integration

# Download tests (requires network, creates files)
cargo test --test download_tests
```

### Individual Tests

```bash
# Run a specific test
cargo test test_filename_sanitization

# Run tests with output
cargo test -- --nocapture

# Run tests matching pattern
cargo test download -- --nocapture
```

## Test Downloads Directory

All test downloads are saved to `tests/downloads/` which is:

- Created automatically when tests run
- Git-ignored (won't be committed to repository)
- Organized by test type and source

## Network Dependencies

Integration and download tests require internet access and may show warnings if:

- Network is unavailable
- Manga sources are down or rate limiting
- Temporary connectivity issues occur

Tests are designed to be resilient to network issues and won't fail the test suite for temporary problems.

## Test Timeouts

- Basic operations: 30 seconds
- Downloads: 120 seconds
- Network requests include proper timeout handling

## Expected Test Behavior

### Unit Tests

- Always pass (no network dependencies)
- Fast execution (under 1 second)
- Test core library functionality

### Integration Tests

- May show warnings for network issues
- Test real source functionality
- Validate API contracts

### Download Tests

- Create files in `tests/downloads/`
- Test actual file downloads
- Validate directory structure

## Debugging Tests

Enable detailed output:

```bash
RUST_LOG=debug cargo test -- --nocapture
```

Run specific failing test:

```bash
cargo test test_name -- --exact --nocapture
```

## Test Data

Tests use:

- Real manga sources (KissManga, MangaDex)
- Common search terms ("naruto", "one piece", "manga")
- Small test files for download validation
- Predictable file structures

## Continuous Integration

Tests are designed for CI environments with:

- Graceful handling of network failures
- No interactive prompts
- Predictable file locations
- Comprehensive error reporting
