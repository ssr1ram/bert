//! Command selector UI component

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    PromptBuilder,
    SpecBuilder,
    TaskBuilder,
}

impl Command {
    pub fn all() -> Vec<Self> {
        vec![
            Command::PromptBuilder,
            Command::SpecBuilder,
            Command::TaskBuilder,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Command::PromptBuilder => "Prompt Builder",
            Command::SpecBuilder => "Spec Builder",
            Command::TaskBuilder => "Task Builder",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Command::PromptBuilder => "Build AI prompts from rules",
            Command::SpecBuilder => "Manage specifications (coming)",
            Command::TaskBuilder => "Manage tasks (coming)",
        }
    }

    pub fn is_available(&self) -> bool {
        match self {
            Command::PromptBuilder => true,
            Command::SpecBuilder => false,
            Command::TaskBuilder => false,
        }
    }
}

pub fn render(f: &mut Frame, area: Rect, selected: usize) {
    // Main layout with title, content, and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Title
    render_title(f, chunks[0]);

    // Command list
    render_commands(f, chunks[1], selected);

    // Help text
    render_help(f, chunks[2]);
}

fn render_title(f: &mut Frame, area: Rect) {
    let title = Block::default()
        .borders(Borders::ALL)
        .title("BERT TUI")
        .title_alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn render_commands(f: &mut Frame, area: Rect, selected: usize) {
    let commands = Command::all();

    let items: Vec<ListItem> = commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = i == selected;
            let prefix = if is_selected { "▶ " } else { "  " };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if cmd.is_available() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let name_span = Span::styled(
                format!("{}{:<20}", prefix, cmd.name()),
                style,
            );

            let desc_span = Span::styled(
                cmd.description(),
                if cmd.is_available() {
                    Style::default().fg(Color::Gray)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            );

            ListItem::new(Line::from(vec![name_span, desc_span]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(list, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = "j/k: navigate  Enter: select  q: quit";

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    f.render_widget(help, area);
}
