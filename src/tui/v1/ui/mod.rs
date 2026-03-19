//! UI rendering

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};
use crate::tui::v1::{file_viewer, menu, prompt_builder, settings, state::{AppState, Mode}};

pub fn render(f: &mut Frame, state: &mut AppState, config: &crate::models::config::BertConfig) {
    let area = f.area();

    // Create layout with menu bar at top
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Menu bar
            Constraint::Min(0),     // Content area
        ])
        .split(area);

    // Store menu area for mouse click detection
    state.menu_area = Some(chunks[0]);

    // Render menu bar
    menu::render(f, chunks[0], state.active_menu, &state.color_scheme);

    // Render content based on active mode
    match &mut state.mode {
        Mode::PromptBuilder(pb_state) => {
            prompt_builder::render(f, chunks[1], pb_state, config, &state.color_scheme, state.preview_mode);
        }
        Mode::SpecViewer(viewer_state) => {
            let viewer = file_viewer::FileViewer::new(
                config.specs_directory.clone(),
                "Specs".to_string(),
            );
            file_viewer::render(f, chunks[1], viewer_state, &viewer, &state.color_scheme, state.preview_mode);
        }
        Mode::TaskViewer(viewer_state) => {
            let viewer = file_viewer::FileViewer::new(
                config.tasks_directory.clone(),
                "Tasks".to_string(),
            );
            file_viewer::render(f, chunks[1], viewer_state, &viewer, &state.color_scheme, state.preview_mode);
        }
        Mode::ArchiveViewer(viewer_state) => {
            let archive_dir = config.archive_directory
                .clone()
                .unwrap_or_else(|| config.project_root.join("archive"));
            let viewer = file_viewer::FileViewer::new(
                archive_dir,
                "Archive".to_string(),
            );
            file_viewer::render(f, chunks[1], viewer_state, &viewer, &state.color_scheme, state.preview_mode);
        }
        Mode::Settings => {
            settings::render(f, chunks[1], config, &state.color_scheme);
        }
    }
}
