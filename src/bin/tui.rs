//! Tosho TUI - Modern terminal user interface for manga downloading
//!
//! A completely redesigned, modern TUI with sidebar navigation, modal dialogs,
//! and improved user experience for manga downloading and conversion.

use color_eyre::{eyre::Result, install};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    collections::HashMap,
    io,
    path::PathBuf,
    time::{Duration, Instant},
};
use tokio::{sync::mpsc, time::sleep};
use tosho::prelude::*;

use tosho::tui::{
    ConversionConfig, ConversionMetadata, EbookFormat, VolumeGrouping, convert_directory,
};

// Application events
#[derive(Debug)]
enum AppEvent {
    DownloadComplete(String),
    Error(String),
    ConversionComplete(String),
}

// Application modes with improved navigation
#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    Home,
    Search,
    MangaDetails,
    Downloads,
    Sources,
    Convert,
    Help,
}

// Modal states for editing
#[derive(Debug, Clone, PartialEq)]
enum ModalState {
    None,
    ConvertSettings,
    MetadataEditor,
    PathEditor,

    HelpDialog,
}

// Metadata editor field selection
#[derive(Debug, Clone, Copy, PartialEq)]
enum MetadataField {
    Title = 0,
    Authors = 1,
    Genre = 2,
    Publisher = 3,
    Description = 4,
    Tags = 5,
}

impl MetadataField {
    fn all() -> Vec<MetadataField> {
        vec![
            MetadataField::Title,
            MetadataField::Authors,
            MetadataField::Genre,
            MetadataField::Publisher,
            MetadataField::Description,
            MetadataField::Tags,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            MetadataField::Title => "Title",
            MetadataField::Authors => "Authors",
            MetadataField::Genre => "Genre",
            MetadataField::Publisher => "Publisher",
            MetadataField::Description => "Description",
            MetadataField::Tags => "Tags",
        }
    }

    fn is_required(&self) -> bool {
        matches!(self, MetadataField::Title)
    }
}

#[derive(Debug)]
struct DownloadProgress {
    _chapter_id: String,
    title: String,
    current: usize,
    total: usize,
    completed: bool,
}

// Enhanced color scheme
mod theme {
    use ratatui::style::Color;

    pub const PRIMARY: Color = Color::Rgb(75, 85, 255); // Blue

    pub const ACCENT: Color = Color::Rgb(255, 152, 0); // Orange
    pub const SUCCESS: Color = Color::Rgb(76, 175, 80); // Green
    pub const WARNING: Color = Color::Rgb(255, 193, 7); // Yellow
    pub const ERROR: Color = Color::Rgb(244, 67, 54); // Red
    pub const INFO: Color = Color::Rgb(33, 150, 243); // Light Blue

    pub const TEXT_PRIMARY: Color = Color::Rgb(255, 255, 255);
    pub const TEXT_SECONDARY: Color = Color::Rgb(189, 189, 189);
    pub const TEXT_MUTED: Color = Color::Rgb(117, 117, 117);

    pub const BORDER: Color = Color::Rgb(66, 66, 66);
    pub const BORDER_FOCUS: Color = PRIMARY;
}

struct App {
    // Core state
    mode: AppMode,
    modal_state: ModalState,
    should_quit: bool,
    sidebar_selected: usize,

    // Search state
    search_query: String,
    search_results: Vec<Manga>,
    search_list_state: ListState,
    search_input_active: bool,

    // Manga details state
    selected_manga: Option<Manga>,
    chapters: Vec<Chapter>,
    chapters_list_state: ListState,

    // Downloads state
    downloads: HashMap<String, DownloadProgress>,
    downloads_list_state: ListState,

    // Sources state
    sources: Vec<String>,
    sources_list_state: ListState,

    // Conversion state
    conversion_config: ConversionConfig,
    conversion_source_path: String,
    conversion_in_progress: bool,

    // Modal editing state
    input_buffer: String,
    selected_field: MetadataField,
    settings_selected: usize,
    is_editing_field: bool,

    // UI state
    status_message: String,
    status_type: StatusType,
    _last_update: Instant,

    // Communication
    event_sender: mpsc::UnboundedSender<AppEvent>,
    event_receiver: mpsc::UnboundedReceiver<AppEvent>,

    // Backend
    manga_sources: Sources,
}

#[derive(Debug, Clone, PartialEq)]
enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

impl StatusType {
    fn color(&self) -> Color {
        match self {
            StatusType::Info => theme::INFO,
            StatusType::Success => theme::SUCCESS,
            StatusType::Warning => theme::WARNING,
            StatusType::Error => theme::ERROR,
        }
    }
}

impl App {
    async fn new() -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // Initialize sources
        let mut manga_sources = Sources::new();

        #[cfg(feature = "source-mangadex")]
        {
            manga_sources.add(tosho::sources::MangaDexSource::new());
        }

        #[cfg(feature = "source-kissmanga")]
        {
            manga_sources.add(tosho::sources::KissMangaSource::new());
        }

        let source_ids: Vec<String> = manga_sources
            .list_ids()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        // Initialize conversion config with proper metadata
        let mut conversion_config = ConversionConfig::default();
        if conversion_config.metadata.is_none() {
            conversion_config.metadata = Some(ConversionMetadata::default());
        }

        Ok(Self {
            mode: AppMode::Home,
            modal_state: ModalState::None,
            should_quit: false,
            sidebar_selected: 0,

            search_query: String::new(),
            search_results: Vec::new(),
            search_list_state: ListState::default(),
            search_input_active: false,

            selected_manga: None,
            chapters: Vec::new(),
            chapters_list_state: ListState::default(),

            downloads: HashMap::new(),
            downloads_list_state: ListState::default(),

            sources: source_ids.clone(),
            sources_list_state: ListState::default(),

            conversion_config,
            conversion_source_path: String::new(),
            conversion_in_progress: false,

            input_buffer: String::new(),
            selected_field: MetadataField::Title,
            settings_selected: 0,
            is_editing_field: false,

            status_message: format!("Loaded {} manga sources", source_ids.len()),
            status_type: StatusType::Success,
            _last_update: Instant::now(),

            event_sender: event_sender.clone(),
            event_receiver,

            manga_sources,
        })
    }

    fn sidebar_items() -> Vec<(&'static str, &'static str)> {
        vec![
            ("", "Home"),
            ("", "Search"),
            ("", "Details"),
            ("", "Downloads"),
            ("", "Sources"),
            ("", "Convert"),
            ("", "Help"),
        ]
    }

    fn set_status(&mut self, message: String, status_type: StatusType) {
        self.status_message = message;
        self.status_type = status_type;
    }

    fn clean_title_from_folder_name(folder_name: &str) -> String {
        let mut title = folder_name.to_string();

        // Remove common patterns that make titles look bad
        title = title.replace(" ~", " -"); // Replace ~ with -
        title = title.replace("~", " "); // Replace remaining ~ with space

        // Clean up multiple spaces
        while title.contains("  ") {
            title = title.replace("  ", " ");
        }

        // Trim whitespace
        title = title.trim().to_string();

        // If the title is still very long, try to shorten it intelligently
        if title.len() > 100 {
            // Look for common separators and take the first part
            if let Some(pos) = title.find(" - ") {
                title = title[..pos].to_string();
            } else if let Some(pos) = title.find(" ~ ") {
                title = title[..pos].to_string();
            } else if let Some(pos) = title.find(" (") {
                title = title[..pos].to_string();
            }
        }

        title
    }

    fn auto_set_title_from_path(&mut self, path: &PathBuf) {
        // Auto-set title from folder name
        if let Some(folder_name) = path.file_name() {
            if let Some(name_str) = folder_name.to_str() {
                if let Some(ref mut metadata) = self.conversion_config.metadata {
                    if metadata.title == "Untitled Manga" || metadata.title.is_empty() {
                        let cleaned_title = Self::clean_title_from_folder_name(name_str);
                        metadata.title = cleaned_title.clone();
                        self.set_status(
                            format!("Source path updated and title set to: {}", cleaned_title),
                            StatusType::Success,
                        );
                    } else {
                        self.set_status("Source path updated".to_string(), StatusType::Success);
                    }
                } else {
                    self.set_status("Source path updated".to_string(), StatusType::Success);
                }
            } else {
                self.set_status("Source path updated".to_string(), StatusType::Success);
            }
        } else {
            self.set_status("Source path updated".to_string(), StatusType::Success);
        }
    }

    fn validate_and_normalize_path(path: &str) -> Result<PathBuf, String> {
        if path.trim().is_empty() {
            return Err("Path cannot be empty".to_string());
        }

        let path_buf = PathBuf::from(path.trim());

        // Check if path exists
        if !path_buf.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        // Check if it's a directory
        if !path_buf.is_dir() {
            return Err("Path must be a directory".to_string());
        }

        // Try to canonicalize the path to handle Windows long paths and special characters
        match path_buf.canonicalize() {
            Ok(canonical) => {
                // On Windows, handle long paths by using \\?\ prefix if needed
                #[cfg(windows)]
                {
                    let path_str = canonical.to_string_lossy();
                    if path_str.len() > 260 && !path_str.starts_with("\\\\?\\") {
                        let long_path = format!("\\\\?\\{}", path_str);
                        return Ok(PathBuf::from(long_path));
                    }
                }
                Ok(canonical)
            }
            Err(e) => Err(format!("Invalid path: {}", e)),
        }
    }

    async fn handle_key_event(&mut self, key: KeyCode) -> Result<()> {
        // Handle modal states first
        if self.modal_state != ModalState::None {
            return self.handle_modal_key_event(key).await;
        }

        // Global keys (always available)
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.search_input_active {
                    self.search_input_active = false;
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::F(1) => {
                self.modal_state = ModalState::HelpDialog;
            }
            _ => {
                // Mode-specific handling
                self.handle_mode_key_event(key).await?;
            }
        }

        Ok(())
    }

    async fn handle_modal_key_event(&mut self, key: KeyCode) -> Result<()> {
        match &self.modal_state {
            ModalState::ConvertSettings => {
                match key {
                    KeyCode::Esc => {
                        self.modal_state = ModalState::None;
                        self.set_status("Settings closed".to_string(), StatusType::Info);
                    }
                    KeyCode::Up => {
                        if self.settings_selected > 0 {
                            self.settings_selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.settings_selected < 3 {
                            // 4 settings items (0-3)
                            self.settings_selected += 1;
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.handle_settings_selection().await?;
                    }
                    KeyCode::Char('m') => {
                        self.modal_state = ModalState::MetadataEditor;
                        self.set_status(
                            "Editing metadata - Use ↑↓ to navigate, Enter to edit".to_string(),
                            StatusType::Info,
                        );
                    }
                    KeyCode::Char('p') => {
                        self.modal_state = ModalState::PathEditor;
                        self.input_buffer = self.conversion_source_path.clone();
                        self.set_status(
                            "Edit source path - Press Enter to save, Esc to cancel".to_string(),
                            StatusType::Info,
                        );
                    }
                    _ => {}
                }
            }
            ModalState::MetadataEditor => {
                // If we're actively editing a field
                if self.is_editing_field {
                    match key {
                        KeyCode::Enter => {
                            self.apply_field_edit();
                            self.input_buffer.clear();
                            self.is_editing_field = false;
                        }
                        KeyCode::Esc => {
                            self.input_buffer.clear();
                            self.is_editing_field = false;
                            self.set_status("Field edit cancelled".to_string(), StatusType::Info);
                        }
                        KeyCode::Backspace => {
                            self.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            self.input_buffer.push(c);
                        }
                        _ => {}
                    }
                } else {
                    // Navigation mode
                    match key {
                        KeyCode::Esc => {
                            self.modal_state = ModalState::ConvertSettings;
                            self.set_status(
                                "Returned to conversion settings".to_string(),
                                StatusType::Info,
                            );
                        }
                        KeyCode::Up => {
                            let fields = MetadataField::all();
                            let current_idx = fields
                                .iter()
                                .position(|&f| f == self.selected_field)
                                .unwrap_or(0);
                            if current_idx > 0 {
                                self.selected_field = fields[current_idx - 1];
                            }
                        }
                        KeyCode::Down => {
                            let fields = MetadataField::all();
                            let current_idx = fields
                                .iter()
                                .position(|&f| f == self.selected_field)
                                .unwrap_or(0);
                            if current_idx < fields.len() - 1 {
                                self.selected_field = fields[current_idx + 1];
                            }
                        }
                        KeyCode::Enter => {
                            self.start_field_edit();
                            self.is_editing_field = true;
                        }
                        _ => {}
                    }
                }
            }
            ModalState::PathEditor => match key {
                KeyCode::Enter => {
                    match Self::validate_and_normalize_path(&self.input_buffer) {
                        Ok(validated_path) => {
                            self.conversion_source_path =
                                validated_path.to_string_lossy().to_string();

                            self.auto_set_title_from_path(&validated_path);
                            self.modal_state = ModalState::ConvertSettings;
                        }
                        Err(err) => {
                            self.set_status(format!("Invalid path: {}", err), StatusType::Error);
                        }
                    }
                    self.input_buffer.clear();
                }
                KeyCode::Esc => {
                    self.modal_state = ModalState::ConvertSettings;
                    self.input_buffer.clear();
                    self.set_status("Path edit cancelled".to_string(), StatusType::Info);
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                _ => {}
            },
            ModalState::HelpDialog => {
                if matches!(key, KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ')) {
                    self.modal_state = ModalState::None;
                }
            }
            ModalState::None => {} // This case is handled in the parent function
        }

        Ok(())
    }

    async fn handle_mode_key_event(&mut self, key: KeyCode) -> Result<()> {
        match key {
            // Sidebar navigation
            KeyCode::Tab => {
                self.sidebar_selected = (self.sidebar_selected + 1) % Self::sidebar_items().len();
                self.mode = match self.sidebar_selected {
                    0 => AppMode::Home,
                    1 => AppMode::Search,
                    2 => AppMode::MangaDetails,
                    3 => AppMode::Downloads,
                    4 => AppMode::Sources,
                    5 => AppMode::Convert,
                    6 => AppMode::Help,
                    _ => AppMode::Home,
                };
            }
            _ => {
                // Mode-specific keys
                match self.mode {
                    AppMode::Search => self.handle_search_keys(key).await?,
                    AppMode::MangaDetails => self.handle_manga_keys(key).await?,
                    AppMode::Downloads => self.handle_downloads_keys(key).await?,
                    AppMode::Sources => self.handle_sources_keys(key).await?,
                    AppMode::Convert => self.handle_convert_keys(key).await?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn handle_search_keys(&mut self, key: KeyCode) -> Result<()> {
        if self.search_input_active {
            match key {
                KeyCode::Enter => {
                    self.search_input_active = false;
                    if !self.search_query.trim().is_empty() {
                        self.perform_search().await?;
                    }
                }
                KeyCode::Esc => {
                    self.search_input_active = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                _ => {}
            }
        } else {
            match key {
                KeyCode::Char('s') | KeyCode::Char('/') => {
                    self.search_input_active = true;
                    self.set_status(
                        "Enter search query and press Enter".to_string(),
                        StatusType::Info,
                    );
                }
                KeyCode::Up => {
                    if let Some(selected) = self.search_list_state.selected() {
                        if selected > 0 {
                            self.search_list_state.select(Some(selected - 1));
                        }
                    }
                }
                KeyCode::Down => {
                    let len = self.search_results.len();
                    if len > 0 {
                        let selected = self.search_list_state.selected().unwrap_or(0);
                        if selected < len - 1 {
                            self.search_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some(selected) = self.search_list_state.selected() {
                        if let Some(manga) = self.search_results.get(selected) {
                            self.load_manga_details(manga.clone()).await?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_manga_keys(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up => {
                if let Some(selected) = self.chapters_list_state.selected() {
                    if selected > 0 {
                        self.chapters_list_state.select(Some(selected - 1));
                    }
                }
            }
            KeyCode::Down => {
                let len = self.chapters.len();
                if len > 0 {
                    let selected = self.chapters_list_state.selected().unwrap_or(0);
                    if selected < len - 1 {
                        self.chapters_list_state.select(Some(selected + 1));
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = self.chapters_list_state.selected() {
                    if let Some(chapter) = self.chapters.get(selected) {
                        self.download_chapter(chapter.clone()).await?;
                    }
                }
            }
            KeyCode::Char('a') => {
                self.download_all_chapters().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_downloads_keys(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up => {
                if let Some(selected) = self.downloads_list_state.selected() {
                    if selected > 0 {
                        self.downloads_list_state.select(Some(selected - 1));
                    }
                }
            }
            KeyCode::Down => {
                let len = self.downloads.len();
                if len > 0 {
                    let selected = self.downloads_list_state.selected().unwrap_or(0);
                    if selected < len - 1 {
                        self.downloads_list_state.select(Some(selected + 1));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_sources_keys(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up => {
                if let Some(selected) = self.sources_list_state.selected() {
                    if selected > 0 {
                        self.sources_list_state.select(Some(selected - 1));
                    }
                }
            }
            KeyCode::Down => {
                let len = self.sources.len();
                if len > 0 {
                    let selected = self.sources_list_state.selected().unwrap_or(0);
                    if selected < len - 1 {
                        self.sources_list_state.select(Some(selected + 1));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_convert_keys(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('s') => {
                self.modal_state = ModalState::ConvertSettings;
                self.set_status(
                    "Conversion settings - Use ↑↓ to navigate, Enter to modify".to_string(),
                    StatusType::Info,
                );
            }
            KeyCode::Char('c') => {
                if !self.conversion_source_path.trim().is_empty() {
                    self.perform_conversion().await?;
                } else {
                    self.set_status(
                        "✗ Please set a source path first (press 's' then 'p')".to_string(),
                        StatusType::Error,
                    );
                }
            }
            KeyCode::Char('p') => {
                self.modal_state = ModalState::PathEditor;
                self.input_buffer = self.conversion_source_path.clone();
                self.set_status(
                    "Edit source path - Press Enter to save, Esc to cancel".to_string(),
                    StatusType::Info,
                );
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_settings_selection(&mut self) -> Result<()> {
        match self.settings_selected {
            0 => {
                // Toggle format
                self.conversion_config.output_format = match self.conversion_config.output_format {
                    EbookFormat::Cbz => EbookFormat::Epub,
                    EbookFormat::Epub => EbookFormat::Cbz,
                };
                self.set_status(
                    format!(
                        "Format changed to {:?}",
                        self.conversion_config.output_format
                    ),
                    StatusType::Success,
                );
            }
            1 => {
                // Cycle grouping strategy
                self.conversion_config.volume_grouping =
                    match self.conversion_config.volume_grouping {
                        VolumeGrouping::Name => VolumeGrouping::ImageAnalysis,
                        VolumeGrouping::ImageAnalysis => VolumeGrouping::Manual(10),
                        VolumeGrouping::Manual(_) => VolumeGrouping::Flat,
                        VolumeGrouping::Flat => VolumeGrouping::Name,
                    };
                self.set_status(
                    format!(
                        "Grouping changed to {:?}",
                        self.conversion_config.volume_grouping
                    ),
                    StatusType::Success,
                );
            }
            2 => {
                // Edit source path
                self.modal_state = ModalState::PathEditor;
                self.input_buffer = self.conversion_source_path.clone();
                self.set_status(
                    "Edit source path - Press Enter to save, Esc to cancel".to_string(),
                    StatusType::Info,
                );
            }
            3 => {
                // Edit metadata
                self.modal_state = ModalState::MetadataEditor;
                self.set_status(
                    "Editing metadata - Use ↑↓ to navigate, Enter to edit".to_string(),
                    StatusType::Info,
                );
            }
            _ => {}
        }
        Ok(())
    }

    fn start_field_edit(&mut self) {
        if let Some(ref metadata) = self.conversion_config.metadata {
            self.input_buffer = match self.selected_field {
                MetadataField::Title => metadata.title.clone(),
                MetadataField::Authors => metadata.authors.join(", "),
                MetadataField::Genre => metadata.genre.as_deref().unwrap_or("").to_string(),
                MetadataField::Publisher => metadata.publisher.as_deref().unwrap_or("").to_string(),
                MetadataField::Description => {
                    metadata.description.as_deref().unwrap_or("").to_string()
                }
                MetadataField::Tags => metadata.tags.join(", "),
            };
            self.set_status(
                format!(
                    "Editing {} - Press Enter to save, Esc to cancel",
                    self.selected_field.name().to_lowercase()
                ),
                StatusType::Info,
            );
        }
    }

    fn apply_field_edit(&mut self) {
        if let Some(ref mut metadata) = self.conversion_config.metadata {
            match self.selected_field {
                MetadataField::Title => {
                    // Title can be empty, but we'll keep the input as-is
                    metadata.title = if self.input_buffer.trim().is_empty() {
                        String::new()
                    } else {
                        self.input_buffer.clone()
                    };
                }
                MetadataField::Authors => {
                    if self.input_buffer.trim().is_empty() {
                        metadata.authors = vec![];
                    } else {
                        metadata.authors = self
                            .input_buffer
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
                MetadataField::Genre => {
                    metadata.genre = if self.input_buffer.trim().is_empty() {
                        None
                    } else {
                        Some(self.input_buffer.trim().to_string())
                    };
                }
                MetadataField::Publisher => {
                    metadata.publisher = if self.input_buffer.trim().is_empty() {
                        None
                    } else {
                        Some(self.input_buffer.trim().to_string())
                    };
                }
                MetadataField::Description => {
                    metadata.description = if self.input_buffer.trim().is_empty() {
                        None
                    } else {
                        Some(self.input_buffer.trim().to_string())
                    };
                }
                MetadataField::Tags => {
                    if self.input_buffer.trim().is_empty() {
                        metadata.tags = vec![];
                    } else {
                        metadata.tags = self
                            .input_buffer
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
            }

            let field_value = if self.input_buffer.trim().is_empty() {
                "cleared".to_string()
            } else {
                "updated".to_string()
            };

            self.set_status(
                format!(
                    "{} {} successfully",
                    self.selected_field.name(),
                    field_value
                ),
                StatusType::Success,
            );
        }
    }

    async fn perform_search(&mut self) -> Result<()> {
        self.set_status("Searching...".to_string(), StatusType::Info);

        if let Some(_source_id) = self.sources.first() {
            match self
                .manga_sources
                .search(&self.search_query)
                .limit(20)
                .flatten()
                .await
            {
                Ok(results) => {
                    self.search_results = results;
                    self.search_list_state
                        .select(if self.search_results.is_empty() {
                            None
                        } else {
                            Some(0)
                        });
                    self.set_status(
                        format!("✓ Found {} results", self.search_results.len()),
                        StatusType::Success,
                    );
                }
                Err(e) => {
                    self.set_status(format!("✗ Search failed: {}", e), StatusType::Error);
                }
            }
        }
        Ok(())
    }

    async fn load_manga_details(&mut self, manga: Manga) -> Result<()> {
        self.selected_manga = Some(manga.clone());
        self.mode = AppMode::MangaDetails;
        self.sidebar_selected = 2;

        self.set_status("Loading chapters...".to_string(), StatusType::Info);

        // For now, create mock chapters since API is not available
        let mock_chapters = vec![
            Chapter {
                id: format!("{}-ch1", manga.id),
                number: 1.0,
                title: "Chapter 1".to_string(),
                pages: vec![],
                manga_id: manga.id.clone(),
                source_id: manga.source_id.clone(),
                #[cfg(feature = "chrono")]
                created_at: None,
            },
            Chapter {
                id: format!("{}-ch2", manga.id),
                number: 2.0,
                title: "Chapter 2".to_string(),
                pages: vec![],
                manga_id: manga.id.clone(),
                source_id: manga.source_id.clone(),
                #[cfg(feature = "chrono")]
                created_at: None,
            },
        ];

        self.chapters = mock_chapters;
        self.chapters_list_state
            .select(if self.chapters.is_empty() {
                None
            } else {
                Some(0)
            });
        self.set_status(
            format!("Loaded {} chapters", self.chapters.len()),
            StatusType::Success,
        );
        Ok(())
    }

    async fn download_chapter(&mut self, chapter: Chapter) -> Result<()> {
        let progress = DownloadProgress {
            _chapter_id: chapter.id.clone(),
            title: chapter.title.clone(),
            current: 0,
            total: 1,
            completed: false,
        };

        self.downloads.insert(chapter.id.clone(), progress);
        self.set_status(
            format!("Starting download: {}", chapter.title),
            StatusType::Info,
        );

        // Simulate download
        let sender = self.event_sender.clone();
        let chapter_id = chapter.id.clone();
        tokio::spawn(async move {
            sleep(Duration::from_secs(2)).await;
            let _ = sender.send(AppEvent::DownloadComplete(chapter_id));
        });

        Ok(())
    }

    async fn download_all_chapters(&mut self) -> Result<()> {
        if self.chapters.is_empty() {
            self.set_status("✗ No chapters to download".to_string(), StatusType::Error);
            return Ok(());
        }

        self.set_status(
            format!("Starting download of {} chapters", self.chapters.len()),
            StatusType::Info,
        );

        let chapters_clone = self.chapters.clone();
        for chapter in chapters_clone {
            self.download_chapter(chapter).await?;
        }

        Ok(())
    }

    async fn perform_conversion(&mut self) -> Result<()> {
        if self.conversion_in_progress {
            self.set_status(
                "Conversion already in progress".to_string(),
                StatusType::Warning,
            );
            return Ok(());
        }

        // Validate source path
        let source_path = match Self::validate_and_normalize_path(&self.conversion_source_path) {
            Ok(path) => path,
            Err(err) => {
                self.set_status(format!("✗ Invalid source path: {}", err), StatusType::Error);
                return Ok(());
            }
        };

        self.conversion_in_progress = true;
        self.set_status("Starting conversion...".to_string(), StatusType::Info);

        let config = self.conversion_config.clone();
        let sender = self.event_sender.clone();

        tokio::spawn(async move {
            match convert_directory(source_path, config).await {
                Ok(output_path) => {
                    let _ = sender.send(AppEvent::ConversionComplete(format!(
                        "Conversion completed! Output: {}",
                        output_path.display()
                    )));
                }
                Err(e) => {
                    let _ = sender.send(AppEvent::Error(format!("Conversion failed: {}", e)));
                }
            }
        });

        Ok(())
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::DownloadComplete(id) => {
                if let Some(progress) = self.downloads.get_mut(&id) {
                    progress.completed = true;
                    progress.current = progress.total;
                }
                self.set_status("Download completed".to_string(), StatusType::Success);
            }
            AppEvent::ConversionComplete(message) => {
                self.conversion_in_progress = false;
                self.set_status(message, StatusType::Success);
            }
            AppEvent::Error(message) => {
                self.conversion_in_progress = false;
                self.set_status(message, StatusType::Error);
            }
        }
    }
}

// Rendering implementation
impl App {
    fn render(&mut self, f: &mut Frame) {
        let size = f.size();

        // Main layout with sidebar
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(20), Constraint::Min(0)])
            .split(size);

        // Render sidebar
        self.render_sidebar(f, chunks[0]);

        // Main content area
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status bar
            ])
            .split(chunks[1]);

        // Render main content
        self.render_header(f, main_chunks[0]);
        self.render_main_content(f, main_chunks[1]);
        self.render_status_bar(f, main_chunks[2]);

        // Render modals on top
        self.render_modals(f);
    }

    fn render_sidebar(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = Self::sidebar_items()
            .iter()
            .enumerate()
            .map(|(i, (_icon, label))| {
                let style = if i == self.sidebar_selected {
                    Style::default()
                        .fg(theme::PRIMARY)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::TEXT_SECONDARY)
                };

                ListItem::new(Line::from(label.to_string())).style(style)
            })
            .collect();

        let sidebar = List::new(items)
            .block(
                Block::default()
                    .title("Tosho")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title_style(
                        Style::default()
                            .fg(theme::PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .highlight_style(
                Style::default()
                    .bg(theme::PRIMARY)
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(sidebar, area);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let mode_title = match self.mode {
            AppMode::Home => "Home",
            AppMode::Search => "Search Manga",
            AppMode::MangaDetails => "Manga Details",
            AppMode::Downloads => "Downloads",
            AppMode::Sources => "Sources",
            AppMode::Convert => "Convert",
            AppMode::Help => "Help",
        };

        let header = Paragraph::new(mode_title)
            .style(
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER)),
            )
            .alignment(Alignment::Center);

        f.render_widget(header, area);
    }

    fn render_main_content(&mut self, f: &mut Frame, area: Rect) {
        match self.mode {
            AppMode::Home => self.render_home(f, area),
            AppMode::Search => self.render_search(f, area),
            AppMode::MangaDetails => self.render_manga_details(f, area),
            AppMode::Downloads => self.render_downloads(f, area),
            AppMode::Sources => self.render_sources(f, area),
            AppMode::Convert => self.render_convert(f, area),
            AppMode::Help => self.render_help(f, area),
        }
    }

    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_color = self.status_type.color();

        let status = Paragraph::new(self.status_message.as_str())
            .style(Style::default().fg(status_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(status, area);
    }

    fn render_home(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Welcome card
                Constraint::Min(0),    // Quick actions
            ])
            .margin(1)
            .split(area);

        // Welcome card
        let welcome_text = vec![
            Line::from("Welcome to Tosho!"),
            Line::from(""),
            Line::from("A modern manga downloader and converter."),
            Line::from(""),
            Line::from("Use Tab to navigate or:"),
            Line::from("• Press 's' or '/' to search manga"),
            Line::from("• Press F1 for help"),
        ];

        let welcome = Paragraph::new(welcome_text)
            .style(Style::default().fg(theme::TEXT_PRIMARY))
            .block(
                Block::default()
                    .title("Welcome")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title_style(Style::default().fg(theme::ACCENT)),
            )
            .alignment(Alignment::Left);

        f.render_widget(welcome, chunks[0]);

        // Quick actions
        let actions = vec![
            Line::from("Search for manga"),
            Line::from("View downloads"),
            Line::from("Convert manga to ebooks"),
            Line::from("Manage sources"),
        ];

        let quick_actions = Paragraph::new(actions)
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .block(
                Block::default()
                    .title("Quick Actions")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title_style(Style::default().fg(theme::INFO)),
            );

        f.render_widget(quick_actions, chunks[1]);
    }

    fn render_search(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search input
                Constraint::Min(0),    // Results
            ])
            .margin(1)
            .split(area);

        // Search input
        let input_style = if self.search_input_active {
            Style::default().fg(theme::PRIMARY)
        } else {
            Style::default().fg(theme::TEXT_SECONDARY)
        };

        let search_input = Paragraph::new(self.search_query.as_str())
            .style(input_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if self.search_input_active {
                        Style::default().fg(theme::BORDER_FOCUS)
                    } else {
                        Style::default().fg(theme::BORDER)
                    })
                    .title("Search Query (press 's' or '/' to edit)"),
            );

        f.render_widget(search_input, chunks[0]);

        // Search results
        if self.search_results.is_empty() {
            let placeholder = Paragraph::new("No results. Press 's' or '/' to search.")
                .style(Style::default().fg(theme::TEXT_MUTED))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORDER))
                        .title("Results"),
                )
                .alignment(Alignment::Center);
            f.render_widget(placeholder, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .search_results
                .iter()
                .map(|manga| {
                    ListItem::new(vec![
                        Line::from(vec![Span::styled(
                            manga.title.clone(),
                            Style::default().fg(theme::TEXT_PRIMARY),
                        )]),
                        Line::from(vec![
                            Span::styled("   ", Style::default()),
                            Span::styled(
                                format!("by {}", manga.authors.join(", ")),
                                Style::default().fg(theme::TEXT_SECONDARY),
                            ),
                        ]),
                    ])
                })
                .collect();

            let results_list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORDER))
                        .title(format!("Results ({})", self.search_results.len())),
                )
                .highlight_style(
                    Style::default()
                        .bg(theme::PRIMARY)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(results_list, chunks[1], &mut self.search_list_state);
        }
    }

    fn render_manga_details(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Manga info
                Constraint::Min(0),    // Chapters
            ])
            .margin(1)
            .split(area);

        if let Some(ref manga) = self.selected_manga {
            // Manga info
            let info_text = vec![
                Line::from(vec![
                    Span::styled("Title: ", Style::default().fg(theme::ACCENT)),
                    Span::styled(
                        manga.title.clone(),
                        Style::default().fg(theme::TEXT_PRIMARY),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Authors: ", Style::default().fg(theme::ACCENT)),
                    Span::styled(
                        manga.authors.join(", "),
                        Style::default().fg(theme::TEXT_SECONDARY),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(theme::ACCENT)),
                    Span::styled("Available".to_string(), Style::default().fg(theme::INFO)),
                ]),
                Line::from(vec![
                    Span::styled("Tags: ", Style::default().fg(theme::ACCENT)),
                    Span::styled(
                        manga.tags.join(", "),
                        Style::default().fg(theme::TEXT_SECONDARY),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Description: ",
                    Style::default().fg(theme::ACCENT),
                )]),
                Line::from(
                    manga
                        .description
                        .as_deref()
                        .unwrap_or("No description available")
                        .to_string(),
                ),
            ];

            let manga_info = Paragraph::new(info_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORDER))
                        .title("Manga Information"),
                )
                .wrap(Wrap { trim: true });

            f.render_widget(manga_info, chunks[0]);

            // Chapters list
            if self.chapters.is_empty() {
                let placeholder = Paragraph::new("Loading chapters...")
                    .style(Style::default().fg(theme::TEXT_MUTED))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::BORDER))
                            .title("Chapters"),
                    )
                    .alignment(Alignment::Center);
                f.render_widget(placeholder, chunks[1]);
            } else {
                let items: Vec<ListItem> = self
                    .chapters
                    .iter()
                    .map(|chapter| {
                        ListItem::new(vec![
                            Line::from(vec![Span::styled(
                                chapter.title.clone(),
                                Style::default().fg(theme::TEXT_PRIMARY),
                            )]),
                            Line::from(vec![
                                Span::styled("   ", Style::default()),
                                Span::styled(
                                    format!("Chapter {}", chapter.number),
                                    Style::default().fg(theme::TEXT_SECONDARY),
                                ),
                            ]),
                        ])
                    })
                    .collect();

                let chapters_list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::BORDER))
                            .title(format!(
                                "Chapters ({}) - Enter to download, 'a' for all",
                                self.chapters.len()
                            )),
                    )
                    .highlight_style(
                        Style::default()
                            .bg(theme::PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    );

                f.render_stateful_widget(chapters_list, chunks[1], &mut self.chapters_list_state);
            }
        } else {
            let placeholder = Paragraph::new("No manga selected. Go to Search to select a manga.")
                .style(Style::default().fg(theme::TEXT_MUTED))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORDER))
                        .title("Manga Details"),
                )
                .alignment(Alignment::Center);
            f.render_widget(
                placeholder,
                area.inner(&Margin {
                    horizontal: 1,
                    vertical: 1,
                }),
            );
        }
    }

    fn render_downloads(&mut self, f: &mut Frame, area: Rect) {
        let area = area.inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });

        if self.downloads.is_empty() {
            let placeholder = Paragraph::new("No downloads yet. Download some chapters first!")
                .style(Style::default().fg(theme::TEXT_MUTED))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORDER))
                        .title("Downloads"),
                )
                .alignment(Alignment::Center);
            f.render_widget(placeholder, area);
        } else {
            let items: Vec<ListItem> = self
                .downloads
                .values()
                .map(|download| {
                    let progress = if download.total > 0 {
                        download.current as f64 / download.total as f64
                    } else {
                        0.0
                    };

                    let status_text = if download.completed {
                        "Complete"
                    } else {
                        "Downloading"
                    };

                    let progress_bar = "█".repeat((progress * 20.0) as usize);
                    let empty_bar = "░".repeat(20 - (progress * 20.0) as usize);

                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(
                                download.title.clone(),
                                Style::default().fg(theme::TEXT_PRIMARY),
                            ),
                            Span::styled(
                                format!(" [{}]", status_text),
                                Style::default().fg(theme::TEXT_SECONDARY),
                            ),
                        ]),
                        Line::from(vec![
                            Span::styled("   ", Style::default()),
                            Span::styled(progress_bar, Style::default().fg(theme::SUCCESS)),
                            Span::styled(empty_bar, Style::default().fg(theme::TEXT_MUTED)),
                            Span::styled(
                                format!(" {}/{}", download.current, download.total),
                                Style::default().fg(theme::TEXT_SECONDARY),
                            ),
                        ]),
                    ])
                })
                .collect();

            let downloads_list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORDER))
                        .title(format!("Downloads ({})", self.downloads.len())),
                )
                .highlight_style(
                    Style::default()
                        .bg(theme::PRIMARY)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(downloads_list, area, &mut self.downloads_list_state);
        }
    }

    fn render_sources(&mut self, f: &mut Frame, area: Rect) {
        let area = area.inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });

        let items: Vec<ListItem> = self
            .sources
            .iter()
            .map(|source| {
                ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        source.clone(),
                        Style::default().fg(theme::TEXT_PRIMARY),
                    )]),
                    Line::from(vec![
                        Span::styled("   ", Style::default()),
                        Span::styled("Active", Style::default().fg(theme::SUCCESS)),
                    ]),
                ])
            })
            .collect();

        let sources_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title(format!("Available Sources ({})", self.sources.len())),
            )
            .highlight_style(
                Style::default()
                    .bg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(sources_list, area, &mut self.sources_list_state);
    }

    fn render_convert(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Source path display
                Constraint::Length(8), // Settings preview
                Constraint::Min(0),    // Instructions
            ])
            .margin(1)
            .split(area);

        // Source path display
        let path_text = if self.conversion_source_path.is_empty() {
            "No source path set".to_string()
        } else {
            self.conversion_source_path.clone()
        };

        let path_color = if self.conversion_source_path.is_empty() {
            theme::ERROR
        } else {
            theme::SUCCESS
        };

        let path_display = Paragraph::new(path_text)
            .style(Style::default().fg(path_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title("Source Directory"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(path_display, chunks[0]);

        // Settings preview
        let format_line = format!("Format: {:?}", self.conversion_config.output_format);
        let grouping_line = format!("Grouping: {:?}", self.conversion_config.volume_grouping);
        let output_line = format!("Output: {}", self.conversion_config.output_path.display());

        let (title_line, title_color) = if let Some(ref metadata) = self.conversion_config.metadata
        {
            if metadata.title.is_empty() {
                (
                    "Title: ⚠ Not set (required for EPUB)".to_string(),
                    theme::ERROR,
                )
            } else {
                (format!("Title: {}", metadata.title), theme::SUCCESS)
            }
        } else {
            ("Title: ⚠ No metadata configured".to_string(), theme::ERROR)
        };

        let settings_text = vec![
            Line::from(vec![Span::styled(
                format_line,
                Style::default().fg(theme::TEXT_PRIMARY),
            )]),
            Line::from(vec![Span::styled(
                grouping_line,
                Style::default().fg(theme::TEXT_PRIMARY),
            )]),
            Line::from(vec![Span::styled(
                output_line,
                Style::default().fg(theme::TEXT_PRIMARY),
            )]),
            Line::from(vec![Span::styled(
                title_line,
                Style::default().fg(title_color),
            )]),
        ];

        let settings_preview = Paragraph::new(settings_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title("Settings Preview"),
        );

        f.render_widget(settings_preview, chunks[1]);

        // Instructions
        let instructions = vec![
            Line::from("Conversion Controls:"),
            Line::from(""),
            Line::from("p  - Edit source path"),
            Line::from("s  - Open conversion settings"),
            Line::from("c  - Start conversion"),
            Line::from(""),
            Line::from("Make sure to set a source path and configure metadata before converting."),
            Line::from("EPUB format requires at least a title to be set."),
        ];

        let help = Paragraph::new(instructions)
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title("Instructions"),
            );

        f.render_widget(help, chunks[2]);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let area = area.inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });

        let help_text = vec![
            Line::from("Tosho - Manga Downloader & Converter"),
            Line::from(""),
            Line::from("Global Controls:"),
            Line::from("  Tab       - Navigate between sections"),
            Line::from("  q/Esc     - Quit application"),
            Line::from("  F1        - Show this help"),
            Line::from(""),
            Line::from("Search:"),
            Line::from("  s or /    - Start search"),
            Line::from("  ↑↓        - Navigate results"),
            Line::from("  Enter     - Select manga"),
            Line::from(""),
            Line::from("Manga Details:"),
            Line::from("  ↑↓        - Navigate chapters"),
            Line::from("  Enter     - Download chapter"),
            Line::from("  a         - Download all chapters"),
            Line::from(""),
            Line::from("Conversion:"),
            Line::from("  p         - Edit source path"),
            Line::from("  s         - Open settings"),
            Line::from("  c         - Start conversion"),
            Line::from(""),
            Line::from("Settings Modal:"),
            Line::from("  ↑↓        - Navigate options"),
            Line::from("  Enter     - Modify setting"),
            Line::from("  m         - Edit metadata"),
            Line::from("  Esc       - Close modal"),
        ];

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme::TEXT_PRIMARY))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title("Help"),
            );

        f.render_widget(help, area);
    }

    fn render_modals(&mut self, f: &mut Frame) {
        match &self.modal_state {
            ModalState::ConvertSettings => self.render_settings_modal(f),
            ModalState::MetadataEditor => self.render_metadata_modal(f),
            ModalState::PathEditor => self.render_path_editor_modal(f),
            ModalState::HelpDialog => self.render_help_modal(f),
            ModalState::None => {}
        }
    }

    fn render_settings_modal(&mut self, f: &mut Frame) {
        let area = self.centered_rect(60, 50, f.size());
        f.render_widget(Clear, area);

        let settings_items = vec![
            format!("Output Format: {:?}", self.conversion_config.output_format),
            format!(
                "Volume Grouping: {:?}",
                self.conversion_config.volume_grouping
            ),
            format!(
                "Source Path: {}",
                if self.conversion_source_path.is_empty() {
                    "Not set"
                } else {
                    "Set"
                }
            ),
            "Edit Metadata".to_string(),
        ];

        let items: Vec<ListItem> = settings_items
            .iter()
            .enumerate()
            .map(|(i, setting)| {
                let style = if i == self.settings_selected {
                    Style::default()
                        .fg(theme::PRIMARY)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::TEXT_PRIMARY)
                };
                ListItem::new(setting.clone()).style(style)
            })
            .collect();

        let settings_list = List::new(items)
            .block(
                Block::default()
                    .title("Conversion Settings")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER_FOCUS))
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .bg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(settings_list, area);

        let help_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let help_text = "↑↓: Navigate • Enter: Modify • m: Metadata • p: Path • Esc: Close";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme::BORDER)),
            );

        f.render_widget(help, help_area[1]);
    }

    fn render_metadata_modal(&mut self, f: &mut Frame) {
        let area = self.centered_rect(70, 60, f.size());
        f.render_widget(Clear, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3), // Input field
                Constraint::Length(3), // Help text
            ])
            .split(area);

        if let Some(ref metadata) = self.conversion_config.metadata {
            let fields = MetadataField::all();
            let items: Vec<ListItem> = fields
                .iter()
                .map(|&field| {
                    let value_string = match field {
                        MetadataField::Title => metadata.title.clone(),
                        MetadataField::Authors => metadata.authors.join(", "),
                        MetadataField::Genre => metadata.genre.as_deref().unwrap_or("").to_string(),
                        MetadataField::Publisher => {
                            metadata.publisher.as_deref().unwrap_or("").to_string()
                        }
                        MetadataField::Description => {
                            metadata.description.as_deref().unwrap_or("").to_string()
                        }
                        MetadataField::Tags => metadata.tags.join(", "),
                    };

                    let display_value = if value_string.is_empty() {
                        if field == MetadataField::Authors && metadata.authors.is_empty() {
                            "No authors".to_string()
                        } else if field == MetadataField::Tags && metadata.tags.is_empty() {
                            "No tags".to_string()
                        } else {
                            "Not set".to_string()
                        }
                    } else {
                        value_string
                    };

                    let style = if field == self.selected_field {
                        Style::default()
                            .fg(theme::PRIMARY)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme::TEXT_PRIMARY)
                    };

                    let required_marker = if field.is_required() { " *" } else { "" };
                    let line_text =
                        format!("{}{}: {}", field.name(), required_marker, display_value);

                    ListItem::new(line_text).style(style)
                })
                .collect();

            let metadata_list = List::new(items)
                .block(
                    Block::default()
                        .title("Ebook Metadata")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::BORDER_FOCUS))
                        .border_type(BorderType::Rounded),
                )
                .highlight_style(
                    Style::default()
                        .bg(theme::PRIMARY)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_widget(metadata_list, chunks[0]);
        }

        // Input field when editing (always show when in metadata editor)
        let input_text = if self.is_editing_field {
            self.input_buffer.clone()
        } else {
            format!("Press Enter to edit {}", self.selected_field.name())
        };

        let input_style = if self.is_editing_field {
            Style::default().fg(theme::PRIMARY)
        } else {
            Style::default().fg(theme::TEXT_MUTED)
        };

        let input = Paragraph::new(input_text).style(input_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_FOCUS))
                .title(format!("Edit {}", self.selected_field.name())),
        );
        f.render_widget(input, chunks[1]);

        let help_text = if self.is_editing_field {
            "Type to edit • Enter: Save • Esc: Cancel"
        } else {
            "↑↓: Navigate • Enter: Edit field • Esc: Back • * = Required"
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER)),
            );

        f.render_widget(help, chunks[2]);
    }

    fn render_path_editor_modal(&self, f: &mut Frame) {
        let area = self.centered_rect(80, 20, f.size());
        f.render_widget(Clear, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(area);

        let input = Paragraph::new(self.input_buffer.as_str())
            .style(Style::default().fg(theme::PRIMARY))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER_FOCUS))
                    .border_type(BorderType::Rounded)
                    .title("Edit Source Path"),
            );

        f.render_widget(input, chunks[0]);

        let help = Paragraph::new("Enter: Save • Esc: Cancel")
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER)),
            );

        f.render_widget(help, chunks[1]);
    }

    fn render_help_modal(&self, f: &mut Frame) {
        let area = self.centered_rect(70, 80, f.size());
        f.render_widget(Clear, area);

        let help_text = vec![
            Line::from("Tosho - Quick Help"),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  Tab       - Switch sections"),
            Line::from("  ↑↓        - Navigate items"),
            Line::from("  Enter     - Select/Activate"),
            Line::from("  Esc       - Go back/Cancel"),
            Line::from("  q         - Quit application"),
            Line::from(""),
            Line::from("Search:"),
            Line::from("  s, /      - Start search"),
            Line::from("  Type to search, Enter to execute"),
            Line::from(""),
            Line::from("Conversion:"),
            Line::from("  p         - Edit source path"),
            Line::from("  s         - Settings"),
            Line::from("  c         - Start conversion"),
            Line::from(""),
            Line::from("Tips:"),
            Line::from("  • Set source path before converting"),
            Line::from("  • EPUB requires metadata title"),
            Line::from("  • Use long path format for Windows"),
            Line::from(""),
            Line::from("Press any key to close"),
        ];

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme::TEXT_PRIMARY))
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::INFO))
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(help, area);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    install()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new().await?;

    // Main loop
    loop {
        terminal.draw(|f| app.render(f))?;

        // Handle events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key_event(key.code).await?;
                }
            }
        }

        // Handle app events
        while let Ok(app_event) = app.event_receiver.try_recv() {
            app.handle_app_event(app_event);
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
