//! Generic file viewer for browsing directories (specs, tasks, etc.)

use super::colors::ColorScheme;
use super::prompt_builder::{TreeItem, TreeItemType};
use super::state::{TreeViewerState, PreviewMode};
use crate::errors::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::collections::HashSet;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct FileViewer {
    root_directory: PathBuf,
    title: String,
}

impl FileViewer {
    pub fn new(root_directory: PathBuf, title: String) -> Self {
        Self {
            root_directory,
            title,
        }
    }

    /// Scan directory and build tree structure
    pub fn scan_tree(&self, expanded_folders: &HashSet<PathBuf>) -> Result<Vec<TreeItem>> {
        if !self.root_directory.exists() {
            fs::create_dir_all(&self.root_directory)?;
            return Ok(Vec::new());
        }

        let mut items = Vec::new();
        self.scan_directory_recursive(&self.root_directory, &PathBuf::new(), 0, expanded_folders, &mut items)?;

        Ok(items)
    }

    fn scan_directory_recursive(
        &self,
        root: &Path,
        relative_path: &Path,
        depth: usize,
        expanded_folders: &HashSet<PathBuf>,
        items: &mut Vec<TreeItem>,
    ) -> Result<()> {
        let full_path = root.join(relative_path);

        let mut entries: Vec<_> = fs::read_dir(&full_path)?
            .filter_map(|e| e.ok())
            .collect();

        // Sort: folders first, then files, alphabetically
        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let item_relative_path = relative_path.join(&name);

            if path.is_dir() {
                let is_expanded = expanded_folders.contains(&item_relative_path);

                items.push(TreeItem {
                    path: item_relative_path.clone(),
                    display_name: name.clone(),
                    depth,
                    item_type: TreeItemType::Folder,
                    is_expanded,
                });

                // If expanded, recursively scan children
                if is_expanded {
                    self.scan_directory_recursive(
                        root,
                        &item_relative_path,
                        depth + 1,
                        expanded_folders,
                        items,
                    )?;
                }
            } else if path.extension().map_or(false, |ext| ext == "md") {
                items.push(TreeItem {
                    path: item_relative_path,
                    display_name: name,
                    depth,
                    item_type: TreeItemType::File,
                    is_expanded: false,
                });
            }
        }

        Ok(())
    }
}

/// Render file viewer interface
pub fn render(
    f: &mut Frame,
    area: Rect,
    state: &TreeViewerState,
    viewer: &FileViewer,
    colors: &ColorScheme,
    preview_mode: PreviewMode,
) {
    // Main layout: content + help
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Two-pane layout: tree (40%) + preview (60%)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(main_chunks[0]);

    let tree_items = viewer.scan_tree(&state.expanded_folders).unwrap_or_default();

    render_tree(f, content_chunks[0], state, &tree_items, &viewer.title, colors);
    render_preview(f, content_chunks[1], state, &tree_items, viewer, colors, preview_mode);
    render_help(f, main_chunks[1], colors, preview_mode, &viewer.title);
}

fn render_tree(
    f: &mut Frame,
    area: Rect,
    state: &TreeViewerState,
    items: &[TreeItem],
    title: &str,
    colors: &ColorScheme,
) {
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == state.cursor;

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

    let display_title = if items.is_empty() {
        format!("{} (empty)", title)
    } else {
        format!("{} ({} items)", title, items.len())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{}{}", colors.active_title_prefix, display_title))
        .title_alignment(Alignment::Left)
        .border_style(colors.active_border_style())
        .style(Style::default().add_modifier(Modifier::BOLD));

    let list = List::new(list_items).block(block);

    f.render_widget(list, area);
}

fn render_preview(
    f: &mut Frame,
    area: Rect,
    state: &TreeViewerState,
    items: &[TreeItem],
    viewer: &FileViewer,
    colors: &ColorScheme,
    preview_mode: PreviewMode,
) {
    let mode_indicator = match preview_mode {
        PreviewMode::Raw => " (Raw)",
        PreviewMode::Rendered => " (Rendered)",
    };
    let title = format!("Preview{}", mode_indicator);

    let border_style = colors.inactive_border_style();

    // Get selected item
    if let Some(item) = items.get(state.cursor) {
        match item.item_type {
            TreeItemType::File => {
                // Read and display file content
                let full_path = viewer.root_directory.join(&item.path);
                if let Ok(mut file) = std::fs::File::open(full_path) {
                    let mut content = String::new();
                    if file.read_to_string(&mut content).is_ok() {
                        match preview_mode {
                            PreviewMode::Raw => {
                                // Show raw text
                                let lines: Vec<Line> = content
                                    .lines()
                                    .take(30)
                                    .map(|line| Line::from(line.to_string()))
                                    .collect();

                                let paragraph = Paragraph::new(lines)
                                    .block(Block::default()
                                        .borders(Borders::ALL)
                                        .title(title.clone())
                                        .border_style(border_style))
                                    .style(colors.normal_text_style());

                                f.render_widget(paragraph, area);
                                return;
                            }
                            PreviewMode::Rendered => {
                                // Render markdown
                                let rendered = tui_markdown::from_str(&content);
                                let paragraph = Paragraph::new(format!("{}", rendered))
                                    .block(Block::default()
                                        .borders(Borders::ALL)
                                        .title(title.clone())
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
                        .title(title.clone())
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
            .title(title)
            .border_style(border_style))
        .alignment(Alignment::Center)
        .style(colors.dimmed_text_style());

    f.render_widget(empty_text, area);
}

fn render_help(f: &mut Frame, area: Rect, colors: &ColorScheme, preview_mode: PreviewMode, title: &str) {
    let mode_text = match preview_mode {
        PreviewMode::Raw => "raw",
        PreviewMode::Rendered => "rendered",
    };

    // Add "a: archive" for Specs viewer
    let help_text = if title == "Specs" {
        format!("j/k: navigate  l/Enter: expand  h: collapse  E: expand all  C: collapse all  a: archive  p: toggle preview ({})  1-5: mode  q: quit", mode_text)
    } else {
        format!("j/k: navigate  l/Enter: expand  h: collapse  E: expand all  C: collapse all  p: toggle preview ({})  1-5: mode  q: quit", mode_text)
    };

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(colors.inactive_border_style()))
        .alignment(Alignment::Center)
        .style(colors.help_text_style());

    f.render_widget(help, area);
}
