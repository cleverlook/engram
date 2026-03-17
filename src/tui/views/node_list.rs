use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::models::node::NodeStatus;
use crate::tui::app::App;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    // Split: left list (40%) | right preview (60%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_list(app, frame, chunks[0]);
    render_preview(app, frame, chunks[1]);
}

fn render_list(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app
        .nodes
        .iter()
        .map(|node| {
            let status_icon = match node.status {
                NodeStatus::Active => "●",
                NodeStatus::Dirty => "◐",
                NodeStatus::Stale => "○",
                NodeStatus::Deprecated => "✕",
            };
            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", status_icon),
                    Style::default().fg(status_color(&node.status)),
                ),
                Span::styled(&node.id, Style::default().fg(Color::White).bold()),
                Span::styled(
                    format!("  w:{}", node.weight),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " Nodes ({}) [sort: {:?}] ",
                    app.nodes.len(),
                    app.sort_by
                ))
                .title_style(Style::default().bold()),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.selected_index));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_preview(app: &App, frame: &mut Frame, area: Rect) {
    let content = match app.selected_node() {
        Some(node) => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&node.id, Style::default().bold()),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:?}", node.status),
                        Style::default().fg(status_color(&node.status)),
                    ),
                    Span::styled("  Weight: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{}", node.weight)),
                ]),
                Line::from(vec![
                    Span::styled("Touched: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(node.touched.format("%Y-%m-%d %H:%M").to_string()),
                ]),
                Line::from(""),
                Line::from(Span::styled("Content:", Style::default().fg(Color::Yellow))),
            ];
            for line in node.content.lines() {
                lines.push(Line::from(line.to_string()));
            }
            if !node.edges.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Edges:",
                    Style::default().fg(Color::Yellow),
                )));
                for edge in &node.edges {
                    lines.push(Line::from(vec![
                        Span::raw("  → "),
                        Span::styled(&edge.to, Style::default().fg(Color::Cyan)),
                        Span::styled(
                            format!("  [{:?} w:{}]", edge.edge_type, edge.weight),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                }
            }
            lines
        }
        None => vec![Line::from("No node selected")],
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Preview ")
                .title_style(Style::default().bold()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn status_color(status: &NodeStatus) -> Color {
    match status {
        NodeStatus::Active => Color::Green,
        NodeStatus::Dirty => Color::Yellow,
        NodeStatus::Stale => Color::Yellow,
        NodeStatus::Deprecated => Color::Red,
    }
}
