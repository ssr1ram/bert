//! Prompt builder UI and logic

use super::colors::ColorScheme;
use super::state::{PromptBuilderState, ActivePane, View};
use crate::models::config::BertConfig;
use crate::errors::Result;
use std::fs;
use std::path::PathBuf;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct TreeItem {
    pub path: PathBuf,          // Full path relative to library root
    pub display_name: String,   // Just the file/folder name
    pub depth: usize,           // Indentation level
    pub item_type: TreeItemType,
    pub is_expanded: bool,      // Only relevant for folders
}

#[derive(Debug, Clone)]
pub enum TreeItemType {
    Folder,
    File,
}

#[derive(Debug, Clone)]
pub struct PromptBuilder<'a> {
    config: &'a BertConfig,
}

impl<'a> PromptBuilder<'a> {
    pub fn new(config: &'a BertConfig) -> Self {
        Self { config }
    }

    /// Scan library directory and build a tree structure with expanded folders
    pub fn scan_library_tree(&self, expanded_folders: &std::collections::HashSet<PathBuf>) -> Result<Vec<TreeItem>> {
        let library_dir = self.config.library_directory
            .as_ref()
            .ok_or_else(|| crate::errors::BertError::ConfigError(
                "library_directory not configured".to_string()
            ))?;

        if !library_dir.exists() {
            fs::create_dir_all(library_dir)?;
            return Ok(Vec::new());
        }

        super::tree_scan::scan_directory_tree(library_dir, expanded_folders, |p| p.is_file())
    }

    /// Remove item from build queue by index
    pub fn remove_from_queue(&self, state: &mut PromptBuilderState, index: usize) {
        if index < state.build_queue.len() {
            state.build_queue.remove(index);
        }
    }

    /// Scan sets directory and return list of available sets
    pub fn scan_sets(&self) -> Result<Vec<TreeItem>> {
        let sets_dir = self.config.sets_directory
            .as_ref()
            .ok_or_else(|| crate::errors::BertError::ConfigError(
                "sets_directory not configured".to_string()
            ))?;

        if !sets_dir.exists() {
            fs::create_dir_all(sets_dir)?;
            return Ok(Vec::new());
        }

        let mut items = Vec::new();

        let mut entries: Vec<_> = fs::read_dir(sets_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map_or(false, |ext| ext == "yaml" || ext == "yml")
            })
            .collect();

        // Sort alphabetically
        entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for entry in entries {
            let path = entry.path();
            // Strip .yaml extension for display
            let name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            items.push(TreeItem {
                path: PathBuf::from(&name), // Just the name, not full path
                display_name: name,
                depth: 0,
                item_type: TreeItemType::File, // Sets are treated as "files" in the UI
                is_expanded: false,
            });
        }

        Ok(items)
    }

    /// Load a set and add its files to the build queue
    pub fn load_set(&self, state: &mut PromptBuilderState, set_name: &str) -> Result<usize> {
        use crate::models::set::PromptSet;

        let sets_dir = self.config.sets_directory
            .as_ref()
            .ok_or_else(|| crate::errors::BertError::ConfigError(
                "sets_directory not configured".to_string()
            ))?;

        let set_path = sets_dir.join(format!("{}.yaml", set_name));
        let set = PromptSet::from_file(&set_path)?;

        let mut added_count = 0;

        for file_path in set.files {
            // Skip if already in queue
            if !state.build_queue.iter().any(|item| item.path == file_path) {
                // Extract display name from path
                let display_name = file_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                state.build_queue.push(crate::tui::v1::state::BuildItem {
                    path: file_path,
                    display_name,
                    item_type: crate::tui::v1::state::BuildItemType::Rule,
                });

                added_count += 1;
            }
        }

        Ok(added_count)
    }

    /// Save current queue as a set
    pub fn save_set(&self, state: &PromptBuilderState, set_name: &str) -> Result<()> {
        use crate::models::set::PromptSet;

        // Validate name
        PromptSet::validate_name(set_name)
            .map_err(crate::errors::BertError::ConfigError)?;

        let sets_dir = self.config.sets_directory
            .as_ref()
            .ok_or_else(|| crate::errors::BertError::ConfigError(
                "sets_directory not configured".to_string()
            ))?;

        // Extract paths from build queue
        let files: Vec<PathBuf> = state.build_queue.iter()
            .map(|item| item.path.clone())
            .collect();

        if files.is_empty() {
            return Err(crate::errors::BertError::ConfigError(
                "Cannot save empty set".to_string()
            ));
        }

        let set = PromptSet::new(set_name.to_string(), files);
        set.save(sets_dir)?;

        Ok(())
    }

    /// Delete a set
    pub fn delete_set(&self, set_name: &str) -> Result<()> {
        use crate::models::set::PromptSet;

        let sets_dir = self.config.sets_directory
            .as_ref()
            .ok_or_else(|| crate::errors::BertError::ConfigError(
                "sets_directory not configured".to_string()
            ))?;

        PromptSet::delete(sets_dir, set_name)?;
        Ok(())
    }

    /// Rename a set
    pub fn rename_set(&self, old_name: &str, new_name: &str) -> Result<()> {
        use crate::models::set::PromptSet;

        let sets_dir = self.config.sets_directory
            .as_ref()
            .ok_or_else(|| crate::errors::BertError::ConfigError(
                "sets_directory not configured".to_string()
            ))?;

        PromptSet::rename(sets_dir, old_name, new_name)?;
        Ok(())
    }

    /// Get set metadata for preview
    pub fn get_set_info(&self, set_name: &str) -> Result<crate::models::set::PromptSet> {
        use crate::models::set::PromptSet;

        let sets_dir = self.config.sets_directory
            .as_ref()
            .ok_or_else(|| crate::errors::BertError::ConfigError(
                "sets_directory not configured".to_string()
            ))?;

        let set_path = sets_dir.join(format!("{}.yaml", set_name));
        PromptSet::from_file(&set_path)
    }
}

/// Render the three-pane prompt builder interface
pub fn render(f: &mut Frame, area: Rect, state: &mut PromptBuilderState, config: &BertConfig, colors: &ColorScheme, preview_mode: super::state::PreviewMode) {
    // Main layout: content + input + help (no separate tabs line)
    let has_input = state.input_mode.is_some();
    let main_chunks = if has_input {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Input
                Constraint::Length(3),  // Help
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Help
            ])
            .split(area)
    };

    // Three-pane content layout with configurable widths
    let pane_widths = config.tui.pane_widths.as_ref()
        .cloned()
        .unwrap_or_default();

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(pane_widths.explorer),  // Explorer pane (left)
            Constraint::Percentage(pane_widths.buffer),    // Working Buffer/Queue (middle)
            Constraint::Percentage(pane_widths.preview),   // Preview pane (right)
        ])
        .split(main_chunks[0]);

    // Store pane areas for mouse click detection
    state.pane_areas = Some((content_chunks[0], content_chunks[1], content_chunks[2]));

    // Render panes
    let builder = PromptBuilder::new(config);

    // Get items based on current view (Library or Sets)
    let tree_items = match state.view {
        View::Library => builder.scan_library_tree(&state.expanded_folders).unwrap_or_default(),
        View::Sets => builder.scan_sets().unwrap_or_default(),
    };

    render_explorer_tree(f, content_chunks[0], state, &tree_items, colors);
    render_working_buffer(f, content_chunks[1], state, colors);
    render_preview_pane(f, content_chunks[2], state, &tree_items, &builder, colors, preview_mode);

    // Input prompt (if in input mode)
    if has_input {
        render_input(f, main_chunks[1], state, colors);
        render_help(f, main_chunks[2], state, colors, preview_mode);
    } else {
        render_help(f, main_chunks[1], state, colors, preview_mode);
    }

    // Help modal (overlay)
    if state.show_help {
        render_help_modal(f, area, colors);
    }
}

/// Render explorer with tree view (indented, expandable folders)
fn render_explorer_tree(f: &mut Frame, area: Rect, state: &PromptBuilderState, items: &[TreeItem], colors: &ColorScheme) {
    let is_active = state.active_pane == ActivePane::Explorer;

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == state.cursor && is_active;

            // Build indentation string (VS Code uses single space per level)
            let indent = " ".repeat(item.depth);

            // VS Code style: caret comes before the name for folders
            let (prefix, name) = match item.item_type {
                TreeItemType::Folder => {
                    if item.is_expanded {
                        ("▼ ", item.display_name.as_str())
                    } else {
                        ("▶ ", item.display_name.as_str())
                    }
                }
                TreeItemType::File => {
                    ("  ", item.display_name.as_str())
                }
            };

            let style = if is_selected {
                colors.selected_item_style()
            } else {
                match item.item_type {
                    TreeItemType::Folder => Style::default().fg(colors.folder_color),
                    TreeItemType::File => colors.normal_text_style(),
                }
            };

            // VS Code style: indent + caret + name (no trailing slash, no selection indicator)
            let content = format!("{}{}{}", indent, prefix, name);

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    // Create title with Library/Sets tabs
    let library_style = match state.view {
        View::Library => Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD),
        View::Sets => Style::default().fg(ratatui::style::Color::DarkGray),
    };

    let sets_style = match state.view {
        View::Sets => Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD),
        View::Library => Style::default().fg(ratatui::style::Color::DarkGray),
    };

    let title_line = Line::from(vec![
        Span::raw("Explorer  "),
        Span::styled("Library", library_style),
        Span::raw(" "),
        Span::styled("Sets", sets_style),
    ]);

    let block = if is_active {
        Block::default()
            .borders(Borders::ALL)
            .title(title_line)
            .title_alignment(Alignment::Left)
            .border_style(colors.active_border_style())
            .style(Style::default().add_modifier(Modifier::BOLD))
    } else {
        Block::default()
            .borders(Borders::ALL)
            .title(title_line)
            .title_alignment(Alignment::Left)
            .border_style(colors.inactive_border_style())
    };

    let list = List::new(list_items).block(block);

    f.render_widget(list, area);
}

fn render_working_buffer(f: &mut Frame, area: Rect, state: &PromptBuilderState, colors: &ColorScheme) {
    let is_active = state.active_pane == ActivePane::Queue;
    let base_title = format!("Working Buffer ({})", state.build_queue.len());

    // Check if we recently copied (show feedback for 1 second)
    let recently_copied = state.last_copy_time
        .map(|t| t.elapsed().as_secs_f32() < 1.0)
        .unwrap_or(false);

    let block = if is_active {
        let title_line = if recently_copied {
            // Show green "✓ copied!" feedback
            Line::from(vec![
                Span::styled(
                    format!("{}{} ", colors.active_title_prefix, base_title),
                    colors.active_border_style()
                ),
                Span::styled(
                    "[✓ copied!]",
                    Style::default().fg(ratatui::style::Color::Green).add_modifier(Modifier::BOLD)
                ),
            ])
        } else {
            // Normal state: show [c: copy] in red
            Line::from(vec![
                Span::styled(
                    format!("{}{} ", colors.active_title_prefix, base_title),
                    colors.active_border_style()
                ),
                Span::styled(
                    "[c: copy]",
                    Style::default().fg(ratatui::style::Color::Red).add_modifier(Modifier::BOLD)
                ),
            ])
        };

        Block::default()
            .borders(Borders::ALL)
            .title(title_line)
            .border_style(colors.active_border_style())
            .style(Style::default().add_modifier(Modifier::BOLD))
    } else {
        Block::default()
            .borders(Borders::ALL)
            .title(base_title)
            .border_style(colors.inactive_border_style())
    };

    if state.build_queue.is_empty() {
        let empty_text = Paragraph::new("No items in buffer")
            .block(block)
            .alignment(Alignment::Center)
            .style(colors.dimmed_text_style());

        f.render_widget(empty_text, area);
    } else {
        let items: Vec<ListItem> = state.build_queue
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == state.queue_cursor && is_active;
                let style = if is_selected {
                    colors.selected_item_style()
                } else {
                    colors.normal_text_style()
                };

                // Display the relative path to disambiguate files with the same name
                let display_path = item.path.to_string_lossy();
                let content = format!("{}. {}", i + 1, display_path);
                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let list = List::new(items).block(block);

        f.render_widget(list, area);
    }
}

/// Render preview pane (files or sets)
fn render_preview_pane(
    f: &mut Frame,
    area: Rect,
    state: &PromptBuilderState,
    items: &[TreeItem],
    builder: &PromptBuilder,
    colors: &ColorScheme,
    preview_mode: super::state::PreviewMode,
) {
    let is_active = state.active_pane == ActivePane::Preview;

    let mode_indicator = match preview_mode {
        super::state::PreviewMode::Raw => " (Raw)",
        super::state::PreviewMode::Rendered => " (Rendered)",
    };
    let title = format!("File Preview{}", mode_indicator);

    let border_style = if is_active {
        colors.active_border_style()
    } else {
        colors.inactive_border_style()
    };

    let title_text = if is_active {
        format!("{}{}", colors.active_title_prefix, title)
    } else {
        title.to_string()
    };

    // Get selected item from explorer
    if let Some(item) = items.get(state.cursor) {
        // Handle Sets view differently
        if matches!(state.view, View::Sets) {
            // Preview set metadata
            if let Ok(set) = builder.get_set_info(&item.display_name) {
                let mut text = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("Set: {}", set.name),
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(Span::styled(
                        format!("Created: {}", set.created.format("%b %d, %Y")),
                        colors.dimmed_text_style(),
                    )),
                    Line::from(Span::styled(
                        format!("Files: {}", set.files.len()),
                        colors.dimmed_text_style(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("Contents:", Style::default().add_modifier(Modifier::BOLD))),
                ];

                for (i, file_path) in set.files.iter().enumerate() {
                    let path_str = file_path.to_string_lossy();
                    text.push(Line::from(Span::styled(
                        format!("{}. {}", i + 1, path_str),
                        colors.normal_text_style(),
                    )));
                }

                let paragraph = Paragraph::new(text)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(title_text.clone())
                        .border_style(border_style))
                    .alignment(Alignment::Left);

                f.render_widget(paragraph, area);
                return;
            }
        }

        match item.item_type {
            TreeItemType::File => {
                // Read and display file content (Library view)
                if let Some(library_dir) = &builder.config.library_directory {
                    let full_path = library_dir.join(&item.path);
                    if let Ok(content) = std::fs::read_to_string(full_path) {
                            match preview_mode {
                                super::state::PreviewMode::Raw => {
                                    // Show raw text
                                    let lines: Vec<Line> = content
                                        .lines()
                                        .take(30)
                                        .map(|line| Line::from(line.to_string()))
                                        .collect();

                                    let paragraph = Paragraph::new(lines)
                                        .block(Block::default()
                                            .borders(Borders::ALL)
                                            .title(title_text.clone())
                                            .border_style(border_style))
                                        .style(colors.normal_text_style());

                                    f.render_widget(paragraph, area);
                                    return;
                                }
                                super::state::PreviewMode::Rendered => {
                                    // Render markdown
                                    let rendered = tui_markdown::from_str(&content);
                                    let paragraph = Paragraph::new(format!("{}", rendered))
                                        .block(Block::default()
                                            .borders(Borders::ALL)
                                            .title(title_text.clone())
                                            .border_style(border_style))
                                        .wrap(ratatui::widgets::Wrap { trim: false });

                                    f.render_widget(paragraph, area);
                                    return;
                                }
                            }
                        }
                    }
            }
            TreeItemType::Folder => {
                // Show folder info
                let expanded_text = if item.is_expanded { "expanded" } else { "collapsed" };
                let text = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("📁 {}/", item.display_name),
                        Style::default().fg(colors.folder_color).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("Folder ({}) - press l or Enter to toggle", expanded_text),
                        colors.dimmed_text_style(),
                    )),
                ];

                let paragraph = Paragraph::new(text)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(title_text.clone())
                        .border_style(border_style))
                    .alignment(Alignment::Center);

                f.render_widget(paragraph, area);
                return;
            }
        }
    }

    // Default empty state
    let empty_text = Paragraph::new("No file selected")
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title_text)
            .border_style(border_style))
        .alignment(Alignment::Center)
        .style(colors.dimmed_text_style());

    f.render_widget(empty_text, area);
}

fn render_input(f: &mut Frame, area: Rect, state: &PromptBuilderState, colors: &ColorScheme) {
    use super::state::InputMode;

    if let Some(ref input_mode) = state.input_mode {
        let (prompt_text, buffer) = match input_mode {
            InputMode::SaveSet { buffer } => ("Save as set: ", buffer.as_str()),
            InputMode::RenameSet { old_name, buffer } => {
                // Show old name in dimmed style, then input
                return render_rename_input(f, area, old_name, buffer, colors);
            }
        };

        let input_text = format!("{}{}", prompt_text, buffer);
        let paragraph = Paragraph::new(input_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Input")
                .border_style(Style::default().fg(ratatui::style::Color::Yellow)))
            .style(colors.normal_text_style());

        f.render_widget(paragraph, area);
    }
}

fn render_rename_input(f: &mut Frame, area: Rect, old_name: &str, buffer: &str, colors: &ColorScheme) {
    let text = vec![
        Line::from(vec![
            Span::styled("Rename '", colors.dimmed_text_style()),
            Span::styled(old_name, Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("' to: ", colors.dimmed_text_style()),
            Span::styled(buffer, colors.normal_text_style()),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Rename Set")
            .border_style(Style::default().fg(ratatui::style::Color::Yellow)))
        .style(colors.normal_text_style());

    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, area: Rect, state: &PromptBuilderState, colors: &ColorScheme, preview_mode: super::state::PreviewMode) {
    let mode_text = match preview_mode {
        super::state::PreviewMode::Raw => "raw",
        super::state::PreviewMode::Rendered => "rendered",
    };

    let help_text = match state.active_pane {
        ActivePane::Explorer => {
            let view_specific = match state.view {
                View::Library => "h: collapse  E: expand all  C: collapse all",
                View::Sets => "d: delete  r: rename",
            };
            format!("`: view  Tab: pane  j/k: nav  Space/Enter: add  {}  p: preview ({})  ?: help  q: quit", view_specific, mode_text)
        }
        ActivePane::Queue => {
            format!("`: view  Tab: pane  j/k: select  d: remove  c: copy  s: save  p: preview ({})  ?: help  q: quit", mode_text)
        }
        ActivePane::Preview => {
            format!("`: view  Tab: pane  p: toggle preview ({})  Hold Option/Shift to select text  ?: help  q: quit", mode_text)
        }
    };

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(colors.inactive_border_style()))
        .alignment(Alignment::Center)
        .style(colors.help_text_style());

    f.render_widget(help, area);
}

fn render_help_modal(f: &mut Frame, area: Rect, colors: &ColorScheme) {
    // Create a centered modal that fits within the TUI bounds
    let modal_width = 70.min(area.width.saturating_sub(4));
    let modal_height = 20.min(area.height.saturating_sub(4));

    let vertical_margin = (area.height.saturating_sub(modal_height)) / 2;
    let horizontal_margin = (area.width.saturating_sub(modal_width)) / 2;

    let modal_area = Rect {
        x: area.x + horizontal_margin,
        y: area.y + vertical_margin,
        width: modal_width,
        height: modal_height,
    };

    let help_text = vec![
        Line::from(Span::styled("PROMPT BUILDER HELP", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("NAVIGATION", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  j/k or ↑/↓     Move cursor"),
        Line::from("  h/l            Collapse/expand folders (Library)"),
        Line::from("  Tab / Shift+Tab  Next/previous pane"),
        Line::from("  ` (backtick)   Toggle Library ↔ Sets view"),
        Line::from(""),
        Line::from(vec![
            Span::styled("ACTIONS", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Space / Enter  Add file/set to buffer"),
        Line::from("  d              Remove from buffer / Delete set"),
        Line::from("  r              Rename set (Sets view)"),
        Line::from("  c              Copy buffer to clipboard"),
        Line::from("  s              Save buffer as set"),
        Line::from("  p              Toggle preview mode"),
        Line::from("  m              Toggle mouse capture"),
        Line::from(""),
        Line::from(vec![
            Span::styled("TEXT SELECTION", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Hold Option (Mac) or Shift (Linux/Win) while dragging to select text"),
        Line::from("  Or press 'm' to disable mouse, then use normal selection"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ? or Esc to close", Style::default().fg(ratatui::style::Color::Yellow)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .border_style(colors.active_border_style())
            .style(colors.normal_text_style()))
        .alignment(Alignment::Left)
        .style(colors.normal_text_style());

    f.render_widget(paragraph, modal_area);
}
