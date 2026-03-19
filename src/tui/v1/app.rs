//! Main application loop and logic

use super::{events, prompt_builder, settings, state::{AppState, Mode, ActivePane, View, MenuItem, PreviewMode}, terminal::Tui, ui};
use crate::models::config::BertConfig;
use crate::errors::Result;
use crossterm::event::KeyCode;
use std::path::PathBuf;

pub struct App {
    config: BertConfig,
    state: AppState,
}

impl App {
    pub fn new(config: BertConfig, command: Option<String>) -> Result<Self> {
        Ok(App {
            config,
            state: AppState::new(command),
        })
    }

    pub fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        while !self.state.should_quit {
            // Draw UI
            let config = self.config.clone();
            terminal.draw(|f| ui::render(f, &mut self.state, &config))?;

            // Handle events
            if let Some(event) = events::read_event()? {
                self.handle_event(event)?;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: events::AppEvent) -> Result<()> {
        match event {
            events::AppEvent::Quit => {
                self.state.should_quit = true;
            }
            events::AppEvent::Mouse(mouse) => {
                // Handle mouse clicks
                use crossterm::event::MouseEventKind;

                if let MouseEventKind::Down(_) = mouse.kind {
                    let x = mouse.column;
                    let y = mouse.row;

                    // Check if click is in menu bar
                    if let Some(menu_area) = self.state.menu_area {
                        if y >= menu_area.y && y < menu_area.y + menu_area.height {
                            // Click is in menu bar - determine which menu item
                            if let Some(menu_item) = self.get_clicked_menu_item(x, &menu_area) {
                                self.switch_to_menu(menu_item)?;
                                return Ok(());
                            }
                        }
                    }

                    // Handle pane clicks in PromptBuilder mode
                    if let Mode::PromptBuilder(ref mut state) = self.state.mode {
                        if let Some((explorer_area, queue_area, preview_area)) = state.pane_areas {
                            // Check which pane was clicked
                            if x >= explorer_area.x && x < explorer_area.x + explorer_area.width
                                && y >= explorer_area.y && y < explorer_area.y + explorer_area.height {
                                state.active_pane = ActivePane::Explorer;
                                // Handle clicking on tree items in explorer
                                self.handle_explorer_click(y, &explorer_area)?;
                            } else if x >= queue_area.x && x < queue_area.x + queue_area.width
                                && y >= queue_area.y && y < queue_area.y + queue_area.height {
                                state.active_pane = ActivePane::Queue;
                                // Handle clicking on queue items
                                self.handle_queue_click(y, &queue_area)?;
                            } else if x >= preview_area.x && x < preview_area.x + preview_area.width
                                && y >= preview_area.y && y < preview_area.y + preview_area.height {
                                state.active_pane = ActivePane::Preview;
                            }
                        }
                    }

                    // Handle clicks in tree viewer modes (Spec, Task, Archive)
                    match &self.state.mode {
                        Mode::SpecViewer(_) | Mode::TaskViewer(_) | Mode::ArchiveViewer(_) => {
                            self.handle_tree_viewer_click(x, y)?;
                        }
                        _ => {}
                    }
                }
            }
            events::AppEvent::Key(key) => {
                // Global key handlers (work in all modes)
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.state.should_quit = true;
                        return Ok(());
                    }
                    KeyCode::F(1) | KeyCode::Char('1') => {
                        self.switch_to_menu(MenuItem::Prompt)?;
                        return Ok(());
                    }
                    KeyCode::F(2) | KeyCode::Char('2') => {
                        self.switch_to_menu(MenuItem::Spec)?;
                        return Ok(());
                    }
                    KeyCode::F(3) | KeyCode::Char('3') => {
                        self.switch_to_menu(MenuItem::Task)?;
                        return Ok(());
                    }
                    KeyCode::F(4) | KeyCode::Char('4') => {
                        self.switch_to_menu(MenuItem::Archive)?;
                        return Ok(());
                    }
                    KeyCode::F(5) | KeyCode::Char('5') => {
                        self.switch_to_menu(MenuItem::Settings)?;
                        return Ok(());
                    }
                    KeyCode::Char('p') => {
                        // Toggle preview mode (raw/rendered)
                        self.state.preview_mode = match self.state.preview_mode {
                            PreviewMode::Raw => PreviewMode::Rendered,
                            PreviewMode::Rendered => PreviewMode::Raw,
                        };
                        return Ok(());
                    }
                    KeyCode::Char('m') => {
                        // Toggle mouse capture (to allow terminal text selection)
                        self.toggle_mouse_capture()?;
                        return Ok(());
                    }
                    _ => {}
                }

                // Mode-specific handlers
                match &self.state.mode {
                    Mode::PromptBuilder(_) => {
                        self.handle_prompt_builder_key(key.code)?;
                    }
                    Mode::SpecViewer(_) => {
                        self.handle_spec_viewer_key(key.code)?;
                    }
                    Mode::TaskViewer(_) => {
                        self.handle_task_viewer_key(key.code)?;
                    }
                    Mode::ArchiveViewer(_) => {
                        self.handle_archive_viewer_key(key.code)?;
                    }
                    Mode::Settings => {
                        self.handle_settings_key(key.code)?;
                    }
                }
            }
            events::AppEvent::Tick => {
                // Tick event - can be used for animations/updates
            }
        }

        Ok(())
    }

    fn switch_to_menu(&mut self, menu_item: MenuItem) -> Result<()> {
        use std::collections::HashSet;

        self.state.active_menu = menu_item;

        self.state.mode = match menu_item {
            MenuItem::Prompt => {
                // Auto-expand top-level folders in library for better UX
                let mut expanded_folders = HashSet::new();
                if let Some(library_dir) = &self.config.library_directory {
                    if let Ok(entries) = std::fs::read_dir(library_dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            if entry.path().is_dir() {
                                let folder_name = entry.file_name().to_string_lossy().to_string();
                                expanded_folders.insert(PathBuf::from(folder_name));
                            }
                        }
                    }
                }

                Mode::PromptBuilder(crate::tui::v1::state::PromptBuilderState {
                    view: View::Library,
                    current_path: PathBuf::new(),
                    cursor: 0,
                    build_queue: Vec::new(),
                    active_pane: ActivePane::Explorer,
                    queue_cursor: 0,
                    expanded_folders,
                    pane_areas: None,
                    last_copy_time: None,
                    input_mode: None,
                    show_help: false,
                })
            }
            MenuItem::Spec => Mode::SpecViewer(crate::tui::v1::state::TreeViewerState {
                current_path: PathBuf::new(),
                cursor: 0,
                expanded_folders: HashSet::new(),
                selected_file: None,
            }),
            MenuItem::Task => Mode::TaskViewer(crate::tui::v1::state::TreeViewerState {
                current_path: PathBuf::new(),
                cursor: 0,
                expanded_folders: HashSet::new(),
                selected_file: None,
            }),
            MenuItem::Archive => Mode::ArchiveViewer(crate::tui::v1::state::TreeViewerState {
                current_path: PathBuf::new(),
                cursor: 0,
                expanded_folders: HashSet::new(),
                selected_file: None,
            }),
            MenuItem::Settings => Mode::Settings,
        };

        Ok(())
    }

    fn handle_spec_viewer_key(&mut self, key: KeyCode) -> Result<()> {
        let root_dir = self.config.specs_directory.clone();

        // Handle spec-specific commands first
        if let KeyCode::Char('a') = key {
            return self.archive_selected_spec();
        }

        // Then handle common tree viewer commands
        if let Mode::SpecViewer(ref mut state) = self.state.mode {
            Self::handle_tree_viewer_key_static(key, state, &root_dir)?;
        }
        Ok(())
    }

    fn archive_selected_spec(&mut self) -> Result<()> {
        use super::file_viewer::FileViewer;

        if let Mode::SpecViewer(ref state) = self.state.mode {
            let viewer = FileViewer::new(self.config.specs_directory.clone(), String::new());
            let tree_items = viewer.scan_tree(&state.expanded_folders)?;

            if let Some(item) = tree_items.get(state.cursor) {
                // Only archive folders (spec directories)
                if matches!(item.item_type, super::prompt_builder::TreeItemType::Folder) {
                    // Extract spec number from folder name
                    if let Some(spec_number) = extract_spec_number(&item.path) {
                        // Call the archive command
                        match crate::commands::spec_archive::archive_spec(&self.config, &spec_number) {
                            Ok(_count) => {
                                // Success - spec has been archived
                                // UI will update on next render showing the spec is gone
                            }
                            Err(_e) => {
                                // Error - but we don't want to disrupt the TUI with eprintln
                                // TODO: Add status bar for messages
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_task_viewer_key(&mut self, key: KeyCode) -> Result<()> {
        let root_dir = self.config.tasks_directory.clone();
        if let Mode::TaskViewer(ref mut state) = self.state.mode {
            Self::handle_tree_viewer_key_static(key, state, &root_dir)?;
        }
        Ok(())
    }

    fn handle_archive_viewer_key(&mut self, key: KeyCode) -> Result<()> {
        // Use archive_directory if set, otherwise fallback to project_root/archive
        let root_dir = self.config.archive_directory
            .clone()
            .unwrap_or_else(|| self.config.project_root.join("archive"));

        if let Mode::ArchiveViewer(ref mut state) = self.state.mode {
            Self::handle_tree_viewer_key_static(key, state, &root_dir)?;
        }
        Ok(())
    }

    fn handle_tree_viewer_key_static(
        key: KeyCode,
        state: &mut crate::tui::v1::state::TreeViewerState,
        root_dir: &PathBuf,
    ) -> Result<()> {
        use super::file_viewer::FileViewer;

        let viewer = FileViewer::new(root_dir.clone(), String::new());
        let tree_items = viewer.scan_tree(&state.expanded_folders)?;

        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                let max = tree_items.len().saturating_sub(1);
                if state.cursor < max {
                    state.cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if state.cursor > 0 {
                    state.cursor -= 1;
                }
            }
            KeyCode::Char('l') | KeyCode::Enter => {
                // Toggle folder expand/collapse
                if let Some(item) = tree_items.get(state.cursor) {
                    match item.item_type {
                        prompt_builder::TreeItemType::Folder => {
                            if state.expanded_folders.contains(&item.path) {
                                state.expanded_folders.remove(&item.path);
                            } else {
                                state.expanded_folders.insert(item.path.clone());
                            }
                        }
                        prompt_builder::TreeItemType::File => {
                            // Select file for viewing
                            state.selected_file = Some(item.path.clone());
                        }
                    }
                }
            }
            KeyCode::Char('h') => {
                // Collapse current folder or move to parent
                if let Some(item) = tree_items.get(state.cursor) {
                    match item.item_type {
                        prompt_builder::TreeItemType::Folder if item.is_expanded => {
                            state.expanded_folders.remove(&item.path);
                        }
                        _ => {
                            if let Some(parent) = item.path.parent() {
                                if let Some(parent_idx) = tree_items.iter().position(|i| i.path == parent) {
                                    state.cursor = parent_idx;
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Char('E') => {
                // Expand all folders
                Self::expand_all_folders_static(state, root_dir)?;
            }
            KeyCode::Char('C') => {
                // Collapse all folders
                state.expanded_folders.clear();
            }
            _ => {}
        }

        Ok(())
    }

    /// Helper function to expand all folders recursively
    fn expand_all_folders_static(
        state: &mut crate::tui::v1::state::TreeViewerState,
        root_dir: &PathBuf,
    ) -> Result<()> {
        use std::collections::VecDeque;

        if !root_dir.exists() {
            return Ok(());
        }

        let mut to_visit = VecDeque::new();
        to_visit.push_back(PathBuf::new());

        while let Some(relative_path) = to_visit.pop_front() {
            let full_path = root_dir.join(&relative_path);

            if let Ok(entries) = std::fs::read_dir(&full_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let folder_path = relative_path.join(&name);
                        state.expanded_folders.insert(folder_path.clone());
                        to_visit.push_back(folder_path);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_prompt_builder_key(&mut self, key: KeyCode) -> Result<()> {
        if let Mode::PromptBuilder(ref mut state) = self.state.mode {
            let builder = prompt_builder::PromptBuilder::new(self.config.clone());

            // Handle input mode first (for save/rename operations)
            if let Some(ref mut input_mode) = state.input_mode.clone() {
                match key {
                    KeyCode::Char(c) => {
                        // Add character to buffer
                        match input_mode {
                            crate::tui::v1::state::InputMode::SaveSet { ref mut buffer } => {
                                buffer.push(c);
                                state.input_mode = Some(input_mode.clone());
                            }
                            crate::tui::v1::state::InputMode::RenameSet { ref old_name, ref mut buffer } => {
                                buffer.push(c);
                                state.input_mode = Some(crate::tui::v1::state::InputMode::RenameSet {
                                    old_name: old_name.clone(),
                                    buffer: buffer.clone(),
                                });
                            }
                        }
                        return Ok(());
                    }
                    KeyCode::Backspace => {
                        // Remove last character
                        match input_mode {
                            crate::tui::v1::state::InputMode::SaveSet { ref mut buffer } => {
                                buffer.pop();
                                state.input_mode = Some(input_mode.clone());
                            }
                            crate::tui::v1::state::InputMode::RenameSet { ref old_name, ref mut buffer } => {
                                buffer.pop();
                                state.input_mode = Some(crate::tui::v1::state::InputMode::RenameSet {
                                    old_name: old_name.clone(),
                                    buffer: buffer.clone(),
                                });
                            }
                        }
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        // Submit input
                        match input_mode {
                            crate::tui::v1::state::InputMode::SaveSet { buffer } => {
                                if !buffer.is_empty() {
                                    let _ = builder.save_set(&state, &buffer);
                                }
                                state.input_mode = None;
                            }
                            crate::tui::v1::state::InputMode::RenameSet { old_name, buffer } => {
                                if !buffer.is_empty() {
                                    let _ = builder.rename_set(&old_name, &buffer);
                                }
                                state.input_mode = None;
                            }
                        }
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        // Cancel input
                        state.input_mode = None;
                        return Ok(());
                    }
                    _ => {
                        return Ok(());
                    }
                }
            }

            // Global commands within PromptBuilder (work in any pane)
            match key {
                KeyCode::Char('?') => {
                    // Toggle help modal
                    state.show_help = !state.show_help;
                    return Ok(());
                }
                KeyCode::Char('`') => {
                    // Switch between Library and Sets (backtick key)
                    state.view = match state.view {
                        View::Library => View::Sets,
                        View::Sets => View::Library,
                    };
                    state.cursor = 0;
                    state.current_path = PathBuf::new();
                    return Ok(());
                }
                KeyCode::Tab => {
                    // Move to next pane (right)
                    state.active_pane = match state.active_pane {
                        ActivePane::Explorer => ActivePane::Queue,
                        ActivePane::Queue => ActivePane::Preview,
                        ActivePane::Preview => ActivePane::Explorer, // Wrap around
                    };
                    return Ok(());
                }
                KeyCode::BackTab => {
                    // Move to previous pane (left) - Shift+Tab
                    state.active_pane = match state.active_pane {
                        ActivePane::Explorer => ActivePane::Preview, // Wrap around
                        ActivePane::Queue => ActivePane::Explorer,
                        ActivePane::Preview => ActivePane::Queue,
                    };
                    return Ok(());
                }
                KeyCode::Left => {
                    // Move to left pane (no wrap)
                    state.active_pane = match state.active_pane {
                        ActivePane::Queue => ActivePane::Explorer,
                        ActivePane::Preview => ActivePane::Queue,
                        ActivePane::Explorer => ActivePane::Explorer, // Stay
                    };
                    return Ok(());
                }
                KeyCode::Right => {
                    // Move to right pane (no wrap)
                    state.active_pane = match state.active_pane {
                        ActivePane::Explorer => ActivePane::Queue,
                        ActivePane::Queue => ActivePane::Preview,
                        ActivePane::Preview => ActivePane::Preview, // Stay
                    };
                    return Ok(());
                }
                _ => {}
            }

            // Pane-specific commands
            match state.active_pane {
                ActivePane::Explorer => {
                    match key {
                        KeyCode::Char('l') | KeyCode::Enter => {
                            match state.view {
                                View::Library => {
                                    // Toggle folder expand/collapse or add file to queue
                                    let tree_items = builder.scan_library_tree(&state.expanded_folders)?;
                                    if let Some(item) = tree_items.get(state.cursor) {
                                        match item.item_type {
                                            prompt_builder::TreeItemType::Folder => {
                                                // Toggle folder expansion
                                                if state.expanded_folders.contains(&item.path) {
                                                    state.expanded_folders.remove(&item.path);
                                                } else {
                                                    state.expanded_folders.insert(item.path.clone());
                                                }
                                            }
                                            prompt_builder::TreeItemType::File => {
                                                // Add file to queue
                                                if !state.build_queue.iter().any(|i| i.path == item.path) {
                                                    state.build_queue.push(crate::tui::v1::state::BuildItem {
                                                        path: item.path.clone(),
                                                        display_name: item.display_name.clone(),
                                                        item_type: crate::tui::v1::state::BuildItemType::Rule,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                                View::Sets => {
                                    // Load set and add all files to queue
                                    let tree_items = builder.scan_sets()?;
                                    if let Some(item) = tree_items.get(state.cursor) {
                                        let _ = builder.load_set(state, &item.display_name);
                                        // Silently skip duplicates as per requirements
                                    }
                                }
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            let tree_items = match state.view {
                                View::Library => builder.scan_library_tree(&state.expanded_folders)?,
                                View::Sets => builder.scan_sets()?,
                            };
                            let max = tree_items.len().saturating_sub(1);
                            if state.cursor < max {
                                state.cursor += 1;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if state.cursor > 0 {
                                state.cursor -= 1;
                            }
                        }
                        KeyCode::Char(' ') => {
                            // Add file/set to queue (same as Enter)
                            match state.view {
                                View::Library => {
                                    let tree_items = builder.scan_library_tree(&state.expanded_folders)?;
                                    if let Some(item) = tree_items.get(state.cursor) {
                                        if matches!(item.item_type, prompt_builder::TreeItemType::File) {
                                            if !state.build_queue.iter().any(|i| i.path == item.path) {
                                                state.build_queue.push(crate::tui::v1::state::BuildItem {
                                                    path: item.path.clone(),
                                                    display_name: item.display_name.clone(),
                                                    item_type: crate::tui::v1::state::BuildItemType::Rule,
                                                });
                                            }
                                        }
                                    }
                                }
                                View::Sets => {
                                    let tree_items = builder.scan_sets()?;
                                    if let Some(item) = tree_items.get(state.cursor) {
                                        let _ = builder.load_set(state, &item.display_name);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('h') => {
                            // Collapse current folder or move to parent (Library view only)
                            if matches!(state.view, View::Library) {
                                let tree_items = builder.scan_library_tree(&state.expanded_folders)?;
                                if let Some(item) = tree_items.get(state.cursor) {
                                    match item.item_type {
                                        prompt_builder::TreeItemType::Folder if item.is_expanded => {
                                            // Collapse current folder
                                            state.expanded_folders.remove(&item.path);
                                        }
                                        _ => {
                                            // Move cursor to parent folder
                                            if let Some(parent) = item.path.parent() {
                                                // Find parent in tree
                                                if let Some(parent_idx) = tree_items.iter().position(|i| i.path == parent) {
                                                    state.cursor = parent_idx;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('E') => {
                            // Expand all folders (Library view only)
                            if matches!(state.view, View::Library) {
                                if let Some(library_dir) = self.config.library_directory.clone() {
                                    Self::expand_all_folders_prompt_builder_static(state, &library_dir)?;
                                }
                            }
                        }
                        KeyCode::Char('C') => {
                            // Collapse all folders (Library view only)
                            if matches!(state.view, View::Library) {
                                state.expanded_folders.clear();
                            }
                        }
                        KeyCode::Char('d') => {
                            // Delete set (Sets view only)
                            if matches!(state.view, View::Sets) {
                                let tree_items = builder.scan_sets()?;
                                if let Some(item) = tree_items.get(state.cursor) {
                                    // Delete the set
                                    let _ = builder.delete_set(&item.display_name);
                                    // Adjust cursor if needed
                                    if state.cursor >= tree_items.len().saturating_sub(1) && state.cursor > 0 {
                                        state.cursor -= 1;
                                    }
                                }
                            }
                        }
                        KeyCode::Char('r') => {
                            // Rename set (Sets view only)
                            if matches!(state.view, View::Sets) {
                                let tree_items = builder.scan_sets()?;
                                if let Some(item) = tree_items.get(state.cursor) {
                                    state.input_mode = Some(crate::tui::v1::state::InputMode::RenameSet {
                                        old_name: item.display_name.clone(),
                                        buffer: String::new(),
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
                ActivePane::Queue => {
                    match key {
                        KeyCode::Char('j') | KeyCode::Down => {
                            let max = state.build_queue.len().saturating_sub(1);
                            if state.queue_cursor < max {
                                state.queue_cursor += 1;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if state.queue_cursor > 0 {
                                state.queue_cursor -= 1;
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Delete => {
                            // Delete selected item from queue
                            if !state.build_queue.is_empty() {
                                builder.remove_from_queue(state, state.queue_cursor);
                                if state.queue_cursor >= state.build_queue.len() && state.queue_cursor > 0 {
                                    state.queue_cursor -= 1;
                                }
                            }
                        }
                        KeyCode::Char('c') => {
                            // Copy queue contents to clipboard
                            self.copy_queue_to_clipboard()?;
                        }
                        KeyCode::Char('s') => {
                            // Save current queue as a set
                            if !state.build_queue.is_empty() {
                                state.input_mode = Some(crate::tui::v1::state::InputMode::SaveSet {
                                    buffer: String::new(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
                ActivePane::Preview => {
                    // Preview pane is read-only for now
                    // Could add scrolling in the future
                }
            }
        }

        Ok(())
    }

    /// Expand all folders in the prompt builder
    fn expand_all_folders_prompt_builder_static(
        state: &mut crate::tui::v1::state::PromptBuilderState,
        library_dir: &PathBuf,
    ) -> Result<()> {
        use std::collections::VecDeque;

        if !library_dir.exists() {
            return Ok(());
        }

        let mut to_visit = VecDeque::new();
        to_visit.push_back(PathBuf::new());

        while let Some(relative_path) = to_visit.pop_front() {
            let full_path = library_dir.join(&relative_path);

            if let Ok(entries) = std::fs::read_dir(&full_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let folder_path = relative_path.join(&name);
                        state.expanded_folders.insert(folder_path.clone());
                        to_visit.push_back(folder_path);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_settings_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('c') => {
                // Copy settings to clipboard
                self.copy_settings_to_clipboard()?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Copy the settings/config to clipboard
    fn copy_settings_to_clipboard(&mut self) -> Result<()> {
        use arboard::Clipboard;

        let config_text = settings::format_config_for_clipboard(&self.config);

        if let Ok(mut clipboard) = Clipboard::new() {
            let _ = clipboard.set_text(config_text);
        }

        Ok(())
    }

    /// Copy the working buffer contents to clipboard
    fn copy_queue_to_clipboard(&mut self) -> Result<()> {
        use arboard::Clipboard;

        if let Mode::PromptBuilder(ref mut state) = self.state.mode {
            if state.build_queue.is_empty() {
                // Nothing to copy
                return Ok(());
            }

            // Read all files in the queue and concatenate them
            let mut content = String::new();

            for item in &state.build_queue {
                if let Some(library_dir) = &self.config.library_directory {
                    let full_path = library_dir.join(&item.path);

                    if let Ok(file_content) = std::fs::read_to_string(&full_path) {
                        content.push_str(&format!("# {}\n\n", item.display_name));
                        content.push_str(&file_content);
                        content.push_str("\n\n---\n\n");
                    }
                }
            }

            // Copy to clipboard
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(content);
                // Set timestamp for visual feedback
                state.last_copy_time = Some(std::time::Instant::now());
            }
        }

        Ok(())
    }

    /// Handle clicking on tree items in the prompt builder explorer pane
    fn handle_explorer_click(&mut self, y: u16, explorer_area: &ratatui::layout::Rect) -> Result<()> {
        use super::prompt_builder::PromptBuilder;

        if let Mode::PromptBuilder(ref mut state) = self.state.mode {
            let builder = PromptBuilder::new(self.config.clone());

            // Get items based on current view
            let tree_items = match state.view {
                View::Library => builder.scan_library_tree(&state.expanded_folders).unwrap_or_default(),
                View::Sets => builder.scan_sets().unwrap_or_default(),
            };

            // Calculate which item was clicked
            // Border takes 1 line at top, content starts at y+1
            let relative_y = y.saturating_sub(explorer_area.y + 1);
            let clicked_index = relative_y as usize;

            if clicked_index < tree_items.len() {
                state.cursor = clicked_index;

                // Handle click based on view
                match state.view {
                    View::Library => {
                        // Simulate pressing 'l' (expand/collapse or add to queue)
                        if let Some(item) = tree_items.get(clicked_index) {
                            match item.item_type {
                                super::prompt_builder::TreeItemType::Folder => {
                                    // Toggle folder expansion
                                    if state.expanded_folders.contains(&item.path) {
                                        state.expanded_folders.remove(&item.path);
                                    } else {
                                        state.expanded_folders.insert(item.path.clone());
                                    }
                                }
                                super::prompt_builder::TreeItemType::File => {
                                    // Add file to queue
                                    if !state.build_queue.iter().any(|i| i.path == item.path) {
                                        state.build_queue.push(crate::tui::v1::state::BuildItem {
                                            path: item.path.clone(),
                                            display_name: item.display_name.clone(),
                                            item_type: crate::tui::v1::state::BuildItemType::Rule,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    View::Sets => {
                        // Load set into queue
                        if let Some(item) = tree_items.get(clicked_index) {
                            let _ = builder.load_set(state, &item.display_name);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle clicking on queue items in the prompt builder
    fn handle_queue_click(&mut self, y: u16, queue_area: &ratatui::layout::Rect) -> Result<()> {
        // First, check if we need to trigger copy
        let should_copy = if let Mode::PromptBuilder(ref state) = self.state.mode {
            let relative_y = y.saturating_sub(queue_area.y);
            relative_y == 0 && state.active_pane == ActivePane::Queue
        } else {
            false
        };

        if should_copy {
            return self.copy_queue_to_clipboard();
        }

        // Otherwise, handle selecting items in the queue
        if let Mode::PromptBuilder(ref mut state) = self.state.mode {
            let relative_y = y.saturating_sub(queue_area.y);

            // Calculate which item was clicked (subtract 1 for border)
            let clicked_index = (relative_y - 1) as usize;

            if clicked_index < state.build_queue.len() {
                state.queue_cursor = clicked_index;
            }
        }
        Ok(())
    }

    /// Handle clicking on tree items in Spec/Task/Archive viewers
    fn handle_tree_viewer_click(&mut self, _x: u16, y: u16) -> Result<()> {
        use super::file_viewer::FileViewer;

        // Determine which viewer we're in and get the appropriate directory
        let (root_dir, is_two_pane) = match &self.state.mode {
            Mode::SpecViewer(_) => (self.config.specs_directory.clone(), true),
            Mode::TaskViewer(_) => (self.config.tasks_directory.clone(), true),
            Mode::ArchiveViewer(_) => {
                let archive_dir = self.config.archive_directory
                    .clone()
                    .unwrap_or_else(|| self.config.project_root.join("archive"));
                (archive_dir, true)
            }
            _ => return Ok(()),
        };

        if !is_two_pane {
            return Ok(());
        }

        // Two-pane layout: tree (40%) + preview (60%)
        // We need to determine if click is in the tree pane (left 40%)
        // The content starts after menu (3 lines) and help (3 lines at bottom)

        // Get the state
        let state = match &mut self.state.mode {
            Mode::SpecViewer(s) | Mode::TaskViewer(s) | Mode::ArchiveViewer(s) => s,
            _ => return Ok(()),
        };

        let viewer = FileViewer::new(root_dir, String::new());
        let tree_items = viewer.scan_tree(&state.expanded_folders).unwrap_or_default();

        // Approximate tree pane area (left 40% of content area)
        // Content area starts at y=3 (after menu), tree pane is roughly 40% width
        let content_start_y = 3;

        // Check if click is in tree area (rough approximation)
        if y < content_start_y {
            return Ok(());
        }

        // Calculate which item was clicked (accounting for borders)
        let relative_y = y.saturating_sub(content_start_y + 1); // +1 for border
        let clicked_index = relative_y as usize;

        if clicked_index < tree_items.len() {
            state.cursor = clicked_index;

            // Toggle folder or select file
            if let Some(item) = tree_items.get(clicked_index) {
                match item.item_type {
                    super::prompt_builder::TreeItemType::Folder => {
                        // Toggle folder expansion
                        if state.expanded_folders.contains(&item.path) {
                            state.expanded_folders.remove(&item.path);
                        } else {
                            state.expanded_folders.insert(item.path.clone());
                        }
                    }
                    super::prompt_builder::TreeItemType::File => {
                        // Select file for viewing
                        state.selected_file = Some(item.path.clone());
                    }
                }
            }
        }

        Ok(())
    }

    /// Determine which menu item was clicked based on x coordinate
    fn get_clicked_menu_item(&self, x: u16, menu_area: &ratatui::layout::Rect) -> Option<MenuItem> {
        // Menu format: " [1. Prompt] │  2. Spec  │  3. Task  │  4. Archive  │  5. Settings  "
        // Each item is approximately: "  N. Name  " (with brackets for active)
        // Separator is "│"

        // Relative x position within the menu area
        let relative_x = x.saturating_sub(menu_area.x);

        // Approximate widths (this is a heuristic based on the menu rendering)
        // Border takes 1 char on left
        let start_offset = 1;

        // Approximate each menu item width (number + dot + space + name + spaces + separator)
        // "  1. Prompt  │" ≈ 14 chars
        // " [1. Prompt] │" ≈ 14 chars (active)
        let menu_items = MenuItem::all();
        let menu_count = menu_items.len();

        // Calculate approximate width per item
        let item_names: Vec<&str> = menu_items.iter().map(|item| item.as_str()).collect();
        let mut current_x = start_offset;

        for (i, name) in item_names.iter().enumerate() {
            // Format: "  N. Name  " or " [N. Name] "
            let item_width = if menu_items[i] == self.state.active_menu {
                format!(" [{}. {}] ", i + 1, name).len() as u16
            } else {
                format!("  {}. {}  ", i + 1, name).len() as u16
            };

            // Check if click is within this item
            if relative_x >= current_x && relative_x < current_x + item_width {
                return Some(menu_items[i]);
            }

            // Move past item and separator
            current_x += item_width;
            if i < menu_count - 1 {
                current_x += 1; // separator "│"
            }
        }

        None
    }

    /// Toggle mouse capture on/off
    /// When disabled, users can use terminal's native text selection
    fn toggle_mouse_capture(&mut self) -> Result<()> {
        use crossterm::{execute, event::{EnableMouseCapture, DisableMouseCapture}};
        use std::io;

        self.state.mouse_capture_enabled = !self.state.mouse_capture_enabled;

        if self.state.mouse_capture_enabled {
            execute!(io::stdout(), EnableMouseCapture)?;
        } else {
            execute!(io::stdout(), DisableMouseCapture)?;
        }

        Ok(())
    }
}

/// Extract spec number from a spec directory path
/// Examples:
///   - "spec-08" -> Some("08")
///   - "spec-08-user-auth" -> Some("08")
///   - "other" -> None
fn extract_spec_number(path: &std::path::Path) -> Option<String> {
    let folder_name = path.file_name()?.to_str()?;

    // Match "spec-NN" or "spec-NN-slug" pattern
    if folder_name.starts_with("spec-") {
        let number_part = folder_name.strip_prefix("spec-")?;

        // Extract just the number (before the next hyphen if exists)
        let spec_number = if let Some(hyphen_pos) = number_part.find('-') {
            &number_part[..hyphen_pos]
        } else {
            number_part
        };

        Some(spec_number.to_string())
    } else {
        None
    }
}
