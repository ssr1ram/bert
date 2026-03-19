//! Top-level menu bar rendering

use super::colors::ColorScheme;
use super::state::MenuItem;
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the top menu bar
pub fn render(f: &mut Frame, area: ratatui::layout::Rect, active_menu: MenuItem, colors: &ColorScheme) {
    let menu_items = MenuItem::all();

    let spans: Vec<Span> = menu_items
        .iter()
        .enumerate()
        .flat_map(|(i, item)| {
            let is_active = *item == active_menu;

            let style = if is_active {
                colors.selected_item_style()
            } else {
                colors.normal_text_style()
            };

            let number = i + 1;
            let text = if is_active {
                format!(" [{}. {}] ", number, item.as_str())
            } else {
                format!("  {}. {}  ", number, item.as_str())
            };

            let mut result = vec![Span::styled(text, style)];

            // Add separator except for last item
            if i < menu_items.len() - 1 {
                result.push(Span::styled("│", colors.dimmed_text_style()));
            }

            result
        })
        .collect();

    let menu_line = Line::from(spans);

    let menu = Paragraph::new(menu_line)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(colors.inactive_border_style()))
        .alignment(Alignment::Left)
        .style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(menu, area);
}
