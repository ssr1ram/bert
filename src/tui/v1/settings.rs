//! Settings view renderer

use super::colors::ColorScheme;
use crate::models::config::BertConfig;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the settings view showing configuration directories
pub fn render(f: &mut Frame, area: Rect, config: &BertConfig, colors: &ColorScheme) {
    // Main layout: content + help
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Render settings content
    render_settings_content(f, main_chunks[0], config, colors);

    // Render help bar
    render_help(f, main_chunks[1], colors);
}

fn render_settings_content(f: &mut Frame, area: Rect, config: &BertConfig, colors: &ColorScheme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Configuration")
        .title_alignment(Alignment::Left)
        .border_style(colors.active_border_style())
        .style(Style::default().add_modifier(Modifier::BOLD));

    // Build content lines
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "BERT Configuration",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
    ];

    // Project root
    lines.push(Line::from(vec![
        Span::styled("Project Root:     ", Style::default().fg(colors.folder_color).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{}", config.project_root.display()),
            colors.normal_text_style(),
        ),
    ]));
    lines.push(Line::from(""));

    // Directories section
    lines.push(Line::from(Span::styled(
        "Directories:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Prompts library
    add_directory_line(&mut lines, "Prompts Library", &config.library_directory, colors);

    // Sets directory
    add_directory_line(&mut lines, "Prompt Sets", &config.sets_directory, colors);

    // Specs directory
    lines.push(Line::from(vec![
        Span::styled("  Specs:          ", colors.dimmed_text_style()),
        Span::styled(
            format!("{}", config.specs_directory.display()),
            colors.normal_text_style(),
        ),
    ]));

    // Tasks directory
    lines.push(Line::from(vec![
        Span::styled("  Tasks:          ", colors.dimmed_text_style()),
        Span::styled(
            format!("{}", config.tasks_directory.display()),
            colors.normal_text_style(),
        ),
    ]));

    // Archive directory
    add_directory_line(&mut lines, "Archive", &config.archive_directory, colors);

    // Notes directory
    add_directory_line(&mut lines, "Notes", &config.notes_directory, colors);

    // Product directory
    add_directory_line(&mut lines, "Product", &config.product_directory, colors);

    // Prompt logs
    add_directory_line(&mut lines, "Prompt Logs", &config.prompt_logs, colors);

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // TUI configuration section
    lines.push(Line::from(Span::styled(
        "TUI Configuration:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let pane_widths = config.tui.pane_widths.as_ref()
        .cloned()
        .unwrap_or_default();

    lines.push(Line::from(vec![
        Span::styled("  Pane Widths:    ", colors.dimmed_text_style()),
        Span::styled(
            format!("Explorer: {}%, Buffer: {}%, Preview: {}%",
                    pane_widths.explorer, pane_widths.buffer, pane_widths.preview),
            colors.normal_text_style(),
        ),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

fn add_directory_line(lines: &mut Vec<Line>, label: &str, path: &Option<std::path::PathBuf>, colors: &ColorScheme) {
    let display_text = if let Some(ref p) = path {
        format!("{}", p.display())
    } else {
        "(not configured)".to_string()
    };

    let style = if path.is_some() {
        colors.normal_text_style()
    } else {
        colors.dimmed_text_style()
    };

    lines.push(Line::from(vec![
        Span::styled(format!("  {:16}", format!("{}:", label)), colors.dimmed_text_style()),
        Span::styled(display_text, style),
    ]));
}

fn render_help(f: &mut Frame, area: Rect, colors: &ColorScheme) {
    let help_text = "c: copy config  Hold Option/Shift to select text  m: toggle mouse  q: quit  1-5: switch view";

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(colors.inactive_border_style()))
        .alignment(Alignment::Center)
        .style(colors.help_text_style());

    f.render_widget(help, area);
}

/// Format config as a copyable string
pub fn format_config_for_clipboard(config: &BertConfig) -> String {
    let mut output = String::new();

    output.push_str("BERT Configuration\n");
    output.push_str("==================\n\n");

    output.push_str(&format!("Project Root:      {}\n\n", config.project_root.display()));

    output.push_str("Directories:\n");

    if let Some(ref path) = config.library_directory {
        output.push_str(&format!("  Prompts Library: {}\n", path.display()));
    } else {
        output.push_str("  Prompts Library: (not configured)\n");
    }

    if let Some(ref path) = config.sets_directory {
        output.push_str(&format!("  Prompt Sets:     {}\n", path.display()));
    } else {
        output.push_str("  Prompt Sets:     (not configured)\n");
    }

    output.push_str(&format!("  Specs:           {}\n", config.specs_directory.display()));
    output.push_str(&format!("  Tasks:           {}\n", config.tasks_directory.display()));

    if let Some(ref path) = config.archive_directory {
        output.push_str(&format!("  Archive:         {}\n", path.display()));
    } else {
        output.push_str("  Archive:         (not configured)\n");
    }

    if let Some(ref path) = config.notes_directory {
        output.push_str(&format!("  Notes:           {}\n", path.display()));
    } else {
        output.push_str("  Notes:           (not configured)\n");
    }

    if let Some(ref path) = config.product_directory {
        output.push_str(&format!("  Product:         {}\n", path.display()));
    } else {
        output.push_str("  Product:         (not configured)\n");
    }

    if let Some(ref path) = config.prompt_logs {
        output.push_str(&format!("  Prompt Logs:     {}\n", path.display()));
    } else {
        output.push_str("  Prompt Logs:     (not configured)\n");
    }

    output.push_str("\nTUI Configuration:\n");
    let pane_widths = config.tui.pane_widths.as_ref().cloned().unwrap_or_default();
    output.push_str(&format!("  Pane Widths:     Explorer: {}%, Buffer: {}%, Preview: {}%\n",
                             pane_widths.explorer, pane_widths.buffer, pane_widths.preview));

    output
}
