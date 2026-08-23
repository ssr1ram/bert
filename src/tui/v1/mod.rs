//! TUI v1 implementation using ratatui

mod app;
mod colors;
mod file_viewer;
mod menu;
mod prompt_builder;
mod settings;
mod terminal;
mod events;
mod state;
mod tree_scan;
mod ui;

use crate::models::config::BertConfig;
use crate::errors::Result;

pub fn launch(config: &BertConfig) -> Result<()> {
    // Initialize terminal
    let mut terminal = terminal::init()?;

    // Install panic hook to ensure terminal cleanup on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = terminal::restore();
        original_hook(panic_info);
    }));

    // Create app state
    let mut app = app::App::new(config.clone())?;

    // Run app loop
    let result = app.run(&mut terminal);

    // Always restore terminal, even on error
    let _ = terminal::restore();

    // Restore original panic hook
    let _ = std::panic::take_hook();

    result
}
