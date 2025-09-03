# Tosho TUI - User Guide

## Overview

The Tosho TUI provides a modern terminal interface for manga downloading and conversion. The interface features sidebar navigation, modal dialogs, and comprehensive conversion settings management with automatic path handling and metadata configuration.

## Key Features

- Clean sidebar navigation interface
- Modal-based editing system prevents accidental navigation
- Automatic title extraction from folder names
- Comprehensive ebook metadata editor
- Windows long path support with automatic validation
- Real-time status feedback

## Navigation

### Sidebar Sections

- **Home** - Welcome screen and quick actions
- **Search** - Search for manga across sources
- **Details** - View manga information and chapters
- **Downloads** - Track download progress
- **Sources** - View available manga sources
- **Convert** - Conversion tools and settings
- **Help** - Documentation and keyboard shortcuts

### Global Controls

- `Tab` - Navigate between sidebar sections
- `q` / `Esc` - Quit application (from main screen)
- `F1` - Show help modal
- `Up/Down` - Navigate lists and options
- `Enter` - Select or activate items

## Conversion System

### Setting Up Conversion

1. Navigate to the Convert section
2. Press `s` to open conversion settings
3. Set source path using one of these methods:
    - Navigate to "Source Path" and press Enter
    - Press `p` for quick path editor
4. Configure metadata by navigating to "Edit Metadata"
5. Return to main Convert screen and press `c` to start

### Automatic Title Detection

When you set a source path, the system automatically:

- Extracts the folder name as the default title
- Cleans up common patterns (tildes, extra spaces)
- Shortens overly long titles intelligently
- Only sets the title if it's currently "Untitled Manga" or empty

Example transformations:

- `Akuyaku Reijou no Naka no Hito ~Subtitle~` becomes `Akuyaku Reijou no Naka no Hito - Subtitle`
- Very long folder names are truncated at logical break points

### Conversion Settings

**Output Format**

- CBZ: Comic book archive format
- EPUB: Electronic publication format (requires title)

**Volume Grouping**

- Name: Group by numerical patterns in folder names
- Image Analysis: Detect volume breaks using cover detection
- Manual: Fixed number of chapters per volume
- Flat: Single volume containing all content

**Metadata Fields**

- Title (required for EPUB format)
- Authors (comma-separated for multiple)
- Genre
- Publisher
- Description
- Tags (comma-separated)

### Path Handling

The system includes robust path validation:

- Automatic Windows long path support (\\?\ prefix)
- Directory existence validation
- Real-time error feedback
- Canonicalization for proper path handling

## Modal System

### Conversion Settings Modal

Access: Press `s` from Convert screen

Navigation:

- `Up/Down` - Navigate settings
- `Enter` - Modify selected setting
- `m` - Quick access to metadata editor
- `p` - Quick access to path editor
- `Esc` - Close modal

### Metadata Editor Modal

Access: From conversion settings, select "Edit Metadata"

Two modes:

1. **Navigation Mode**
    - `Up/Down` - Navigate fields
    - `Enter` - Start editing selected field
    - `Esc` - Return to settings

2. **Edit Mode**
    - Type to edit field content
    - `Enter` - Save changes
    - `Esc` - Cancel edit

### Path Editor Modal

Access: Press `p` from Convert screen or select "Source Path" in settings

- Type or paste the directory path
- `Enter` - Validate and save path
- `Esc` - Cancel edit
- Real-time validation feedback

## Error Resolution

### Common Issues

**"ebooktitle is required" Error**

- Cause: Missing title when using EPUB format
- Solution: Set title in metadata editor
- Prevention: System now auto-sets title from folder name

**Path Syntax Errors**

- Cause: Windows long paths or invalid characters
- Solution: System automatically handles path normalization
- Prevention: Real-time path validation

**Navigation Confusion**

- Cause: Accidentally leaving edit modes
- Solution: Modal system prevents unintended navigation
- Prevention: Clear visual indicators for current mode

### Status Messages

The status bar shows color-coded feedback:

- Green: Success operations
- Yellow: Warnings
- Red: Errors
- Blue: Information

## Search and Download

### Searching for Manga

1. Navigate to Search section
2. Press `s` or `/` to activate search
3. Type search query and press Enter
4. Use `Up/Down` to navigate results
5. Press Enter to view manga details

### Downloading Chapters

1. From manga details view
2. Use `Up/Down` to navigate chapters
3. Press Enter to download single chapter
4. Press `a` to download all chapters
5. Monitor progress in Downloads section

## Technical Details

### Build Requirements

```bash
cargo build --features "tui,conversion"
cargo run --bin tosho-tui --features "tui,conversion"
```

### Path Limitations

- Maximum path length: Automatically handled on Windows
- Special characters: Supported through proper encoding
- Network paths: Not currently supported
- Relative paths: Converted to absolute paths

### Performance Considerations

- Large manga collections: UI remains responsive
- Long file paths: Automatic optimization
- Concurrent operations: Downloads and conversion can run simultaneously

## Troubleshooting

### Build Issues

If compilation fails:

1. Ensure all required features are enabled
2. Check Rust version compatibility
3. Verify dependencies are available

### Runtime Issues

**Black or corrupted display**

- Check terminal color support
- Verify terminal size (minimum 80x24)
- Ensure proper font support

**Keyboard input not working**

- Verify terminal has focus
- Check for conflicting terminal shortcuts
- Try different terminal emulator

**Path not found errors**

- Use absolute paths when possible
- Check directory permissions
- Verify path exists and is accessible

### Debug Mode

For detailed logging:

```bash
RUST_LOG=debug cargo run --bin tosho-tui --features "tui,conversion"
```

## Best Practices

### Conversion Workflow

1. Always verify source path exists before conversion
2. Set meaningful titles for better organization
3. Use appropriate volume grouping for your content
4. Choose output format based on target device
5. Verify metadata before starting conversion

### File Organization

- Use descriptive folder names for automatic title detection
- Maintain consistent chapter numbering
- Organize by series for easier management
- Regular cleanup of temporary files

### Performance Optimization

- Close unused modal windows
- Limit concurrent downloads when system resources are limited
- Monitor disk space during conversion operations
- Use efficient path structures for faster access

---

This guide covers the essential functionality of the Tosho TUI. For additional help, press F1 within the application or consult the inline help system.
