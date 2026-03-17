use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::models::node::Node;
use crate::tui::app::SearchState;

pub fn render(state: &SearchState, nodes: &[Node], frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // Search input
    let width = chunks[0].width.saturating_sub(2) as usize; // inside borders
    let scroll = state.input.visual_scroll(width);
    let input_widget = Paragraph::new(state.input.value())
        .scroll((0, scroll as u16))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search (FTS5) ")
                .title_style(Style::default().fg(Color::Yellow).bold()),
        );
    frame.render_widget(input_widget, chunks[0]);

    // Place cursor
    frame.set_cursor_position((
        chunks[0].x + (state.input.visual_cursor().max(scroll) - scroll) as u16 + 1,
        chunks[0].y + 1,
    ));

    // Results list
    let items: Vec<ListItem> = state
        .results
        .iter()
        .map(|id| {
            let node = nodes.iter().find(|n| n.id == *id);
            let preview = node
                .map(|n| n.content.lines().next().unwrap_or("").to_string())
                .unwrap_or_default();
            ListItem::new(Line::from(vec![
                Span::styled(id, Style::default().fg(Color::Cyan).bold()),
                Span::styled(
                    format!("  {}", truncate(&preview, 60)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let results_title = format!(" Results ({}) ", state.results.len());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(results_title))
        .highlight_style(Style::default().bg(Color::DarkGray).bold())
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    if !state.results.is_empty() {
        list_state.select(Some(state.selected));
    }
    frame.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        Some((byte_idx, _)) => format!("{}…", &s[..byte_idx]),
        None => s.to_string(),
    }
}
