//! TUI module - isolated interface for terminal user interfaces
//!
//! This module provides a single entry point for TUI functionality.
//! Implementations are versioned (v1, v2, etc.) to allow easy swapping.

use crate::models::config::BertConfig;
use crate::errors::Result;

// Current active implementation
mod v1;

// Future implementations (commented out)
// mod v2;

/// Launch the TUI with optional direct command
///
/// # Arguments
/// * `config` - Bert configuration
/// * `command` - Optional direct command ("prompt", "spec", "task")
///
/// # Examples
/// ```
/// // Launch command selector
/// tui::launch(&config, None)?;
///
/// // Launch directly to prompt builder
/// tui::launch(&config, Some("prompt"))?;
/// ```
pub fn launch(config: &BertConfig, command: Option<&str>) -> Result<()> {
    // Route to active implementation
    // Change this line to switch implementations
    v1::launch(config, command)

    // To use v2 in the future:
    // v2::launch(config, command)
}
