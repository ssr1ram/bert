//! Color scheme management with light/dark theme detection

use ratatui::style::{Color, Modifier, Style};
use std::time::Duration;

/// Color scheme for the TUI
#[derive(Debug, Clone)]
pub struct ColorScheme {
    // Active pane colors
    pub active_border: Color,
    pub active_border_modifier: Modifier,
    pub active_title_prefix: &'static str,

    // Inactive pane colors
    pub inactive_border: Color,

    // Content colors
    pub selected_item: Color,
    pub normal_text: Color,
    pub dimmed_text: Color,
    pub folder_color: Color,

    // Help text
    pub help_text: Color,
}

impl ColorScheme {
    /// Create a light theme color scheme
    pub fn light() -> Self {
        Self {
            // Active pane uses bright blue for visibility on light backgrounds
            active_border: Color::Blue,
            active_border_modifier: Modifier::BOLD,
            active_title_prefix: "▶ ",

            // Inactive panes use medium gray
            inactive_border: Color::Gray,

            // Content colors optimized for light backgrounds
            selected_item: Color::Blue,
            normal_text: Color::Black,
            dimmed_text: Color::Gray,
            folder_color: Color::Black,  // Same as normal text for VS Code style

            // Help text in medium gray
            help_text: Color::Gray,
        }
    }

    /// Create a dark theme color scheme
    pub fn dark() -> Self {
        Self {
            // Active pane uses cyan for visibility on dark backgrounds
            active_border: Color::Cyan,
            active_border_modifier: Modifier::BOLD,
            active_title_prefix: "▶ ",

            // Inactive panes use dark gray
            inactive_border: Color::DarkGray,

            // Content colors optimized for dark backgrounds
            selected_item: Color::Green,
            normal_text: Color::White,
            dimmed_text: Color::DarkGray,
            folder_color: Color::White,  // Same as normal text for VS Code style

            // Help text in gray
            help_text: Color::Gray,
        }
    }

    /// Get style for active pane border
    pub fn active_border_style(&self) -> Style {
        Style::default()
            .fg(self.active_border)
            .add_modifier(self.active_border_modifier)
    }

    /// Get style for inactive pane border
    pub fn inactive_border_style(&self) -> Style {
        Style::default().fg(self.inactive_border)
    }

    /// Get style for selected item
    pub fn selected_item_style(&self) -> Style {
        Style::default()
            .fg(self.selected_item)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for normal text
    pub fn normal_text_style(&self) -> Style {
        Style::default().fg(self.normal_text)
    }

    /// Get style for dimmed text
    pub fn dimmed_text_style(&self) -> Style {
        Style::default().fg(self.dimmed_text)
    }

    /// Get style for help text
    pub fn help_text_style(&self) -> Style {
        Style::default().fg(self.help_text)
    }
}

/// Detect terminal theme and return appropriate color scheme
///
/// Uses termbg to detect the terminal background color.
/// Falls back to dark theme if detection fails or times out.
pub fn detect_color_scheme() -> ColorScheme {
    let timeout = Duration::from_millis(100);

    match termbg::theme(timeout) {
        Ok(termbg::Theme::Light) => ColorScheme::light(),
        Ok(termbg::Theme::Dark) => ColorScheme::dark(),
        Err(_) => {
            // Default to dark theme if detection fails
            // Most development environments use dark themes
            ColorScheme::dark()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_scheme_has_different_colors_than_dark() {
        let light = ColorScheme::light();
        let dark = ColorScheme::dark();

        // Verify they use different color schemes
        assert_ne!(light.active_border, dark.active_border);
        assert_ne!(light.selected_item, dark.selected_item);
    }

    #[test]
    fn test_detect_returns_valid_scheme() {
        let scheme = detect_color_scheme();
        // Just verify it doesn't panic and returns something
        let _ = scheme.active_border_style();
    }
}
