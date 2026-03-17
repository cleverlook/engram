use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use crate::models::node::{Node, NodeStatus};
use crate::tui::app::DetailState;

pub fn render(node: &Node, state: &DetailState, frame: &mut Frame, area: Rect) {
    let mut lines: Vec<Line> = vec![];

    // Header section
    lines.push(Line::from(Span::styled(
        &node.id,
        Style::default().fg(Color::Cyan).bold().underlined(),
    )));
    lines.push(Line::from(""));

    // Metadata
    lines.push(Line::from(vec![
        Span::styled("Status:  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:?}", node.status),
            Style::default().fg(status_color(&node.status)),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Weight:  ", Style::default().fg(Color::DarkGray)),
        Span::raw(format!("{}", node.weight)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Created: ", Style::default().fg(Color::DarkGray)),
        Span::raw(node.created.format("%Y-%m-%d %H:%M UTC").to_string()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Touched: ", Style::default().fg(Color::DarkGray)),
        Span::raw(node.touched.format("%Y-%m-%d %H:%M UTC").to_string()),
    ]));

    if !node.source_files.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Sources: ", Style::default().fg(Color::DarkGray)),
            Span::raw(node.source_files.join(", ")),
        ]));
    }

    // Content
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "── Content ──────────────────────────",
        Style::default().fg(Color::Yellow),
    )));
    for line in node.content.lines() {
        lines.push(Line::from(line.to_string()));
    }

    // Edges
    if !node.edges.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "── Edges ────────────────────────────",
            Style::default().fg(Color::Yellow),
        )));
        for edge in &node.edges {
            lines.push(Line::from(vec![
                Span::raw("  → "),
                Span::styled(&edge.to, Style::default().fg(Color::Cyan).bold()),
                Span::styled(
                    format!("  [{:?}  w:{}]", edge.edge_type, edge.weight),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    // Data lake
    if !node.data_lake.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "── Data Lake ────────────────────────",
            Style::default().fg(Color::Yellow),
        )));
        for file in &node.data_lake {
            lines.push(Line::from(format!("  📎 {file}")));
        }
    }

    let content_length = lines.len();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", node.id))
                .title_style(Style::default().bold()),
        )
        .wrap(Wrap { trim: false })
        .scroll((state.scroll, 0));

    frame.render_widget(paragraph, area);

    // Scrollbar
    let mut scrollbar_state = ScrollbarState::default()
        .content_length(content_length)
        .position(state.scroll as usize);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut scrollbar_state,
    );
}

fn status_color(status: &NodeStatus) -> Color {
    match status {
        NodeStatus::Active => Color::Green,
        NodeStatus::Dirty => Color::Yellow,
        NodeStatus::Stale => Color::Yellow,
        NodeStatus::Deprecated => Color::Red,
    }
}
