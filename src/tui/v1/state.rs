//! Application state structures

use std::path::PathBuf;
use std::collections::HashSet;
use super::colors::ColorScheme;

#[derive(Debug, Clone)]
pub struct AppState {
    pub mode: Mode,
    pub should_quit: bool,
    pub color_scheme: ColorScheme,
    pub active_menu: MenuItem,
    pub preview_mode: PreviewMode,
    /// Menu bar area for mouse click detection
    pub menu_area: Option<ratatui::layout::Rect>,
    /// Whether mouse capture is enabled (when disabled, terminal text selection works)
    pub mouse_capture_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewMode {
    Raw,      // Show raw markdown text
    Rendered, // Show rendered markdown
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Prompt,
    Spec,
    Task,
    Archive,
    Settings,
}

impl MenuItem {
    pub fn all() -> &'static [MenuItem] {
        &[
            MenuItem::Prompt,
            MenuItem::Spec,
            MenuItem::Task,
            MenuItem::Archive,
            MenuItem::Settings,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MenuItem::Prompt => "Prompt",
            MenuItem::Spec => "Spec",
            MenuItem::Task => "Task",
            MenuItem::Archive => "Archive",
            MenuItem::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    PromptBuilder(PromptBuilderState),
    SpecViewer(TreeViewerState),
    TaskViewer(TreeViewerState),
    ArchiveViewer(TreeViewerState),
    Settings,
}

/// Generic tree viewer state for browsing files
#[derive(Debug, Clone, PartialEq)]
pub struct TreeViewerState {
    pub cursor: usize,
    pub expanded_folders: HashSet<PathBuf>,
    pub selected_file: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PromptBuilderState {
    pub view: View,
    pub cursor: usize,
    pub build_queue: Vec<BuildItem>,
    pub active_pane: ActivePane,
    pub queue_cursor: usize,
    /// Set of expanded folder paths (relative to library root)
    pub expanded_folders: HashSet<PathBuf>,
    /// Pane boundaries for mouse click detection (explorer, queue, preview)
    pub pane_areas: Option<(ratatui::layout::Rect, ratatui::layout::Rect, ratatui::layout::Rect)>,
    /// Timestamp of last clipboard copy (for visual feedback)
    pub last_copy_time: Option<std::time::Instant>,
    /// Input mode state (for save/rename operations)
    pub input_mode: Option<InputMode>,
    /// Show help modal
    pub show_help: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    SaveSet { buffer: String },
    RenameSet { old_name: String, buffer: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivePane {
    Explorer,   // Left pane - file browsing
    Queue,      // Middle pane - working buffer
    Preview,    // Right pane - file content
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Library,
    Sets,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildItem {
    pub path: PathBuf,
    pub display_name: String,
    pub item_type: BuildItemType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildItemType {
    Rule,
}

impl AppState {
    pub fn new() -> Self {
        // Start with top-level folders expanded by default
        let mut expanded_folders = HashSet::new();
        // Expand common top-level folder names
        expanded_folders.insert(PathBuf::from("context"));
        expanded_folders.insert(PathBuf::from("style"));
        expanded_folders.insert(PathBuf::from("workflows"));
        expanded_folders.insert(PathBuf::from("rules"));
        expanded_folders.insert(PathBuf::from("templates"));

        // Always start with Prompt Builder view
        let mode = Mode::PromptBuilder(PromptBuilderState {
            view: View::Library,
            cursor: 0,
            build_queue: Vec::new(),
            active_pane: ActivePane::Explorer,
            queue_cursor: 0,
            expanded_folders,
            pane_areas: None,
            last_copy_time: None,
            input_mode: None,
            show_help: false,
        });

        // Detect terminal theme and set appropriate color scheme
        let color_scheme = super::colors::detect_color_scheme();

        AppState {
            mode,
            should_quit: false,
            color_scheme,
            active_menu: MenuItem::Prompt,
            preview_mode: PreviewMode::Rendered, // Default to rendered
            menu_area: None,
            mouse_capture_enabled: true, // Start with mouse capture enabled
        }
    }
}
