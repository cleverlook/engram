use ratatui::prelude::*;
use ratatui::widgets::{
    Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use crate::models::node::Node;
use crate::tui::app::DetailState;
use crate::tui::theme::{Theme, status_color, status_icon};

pub fn render(node: &Node, state: &DetailState, frame: &mut Frame, area: Rect) {
    let mut lines: Vec<Line> = vec![];

    // Header section
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(
            &node.id,
            Style::default().fg(Theme::ACCENT).bold().underlined(),
        ),
    ]));
    lines.push(Line::from(""));

    // History breadcrumb
    if !state.history.is_empty() {
        let steps = state.history.len();
        lines.push(Line::from(Span::styled(
            format!(
                "   ← {} step{} back (Backspace)",
                steps,
                if steps == 1 { "" } else { "s" }
            ),
            Style::default().fg(Theme::DIM).italic(),
        )));
    }

    // Metadata
    let icon = status_icon(&node.status);
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Status:  ", Style::default().fg(Theme::DIM)),
        Span::styled(
            format!("{icon} {:?}", node.status),
            Style::default().fg(status_color(&node.status)),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Weight:  ", Style::default().fg(Theme::DIM)),
        Span::raw(format!("{}", node.weight)),
    ]));
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Created: ", Style::default().fg(Theme::DIM)),
        Span::raw(node.created.format("%Y-%m-%d %H:%M UTC").to_string()),
    ]));
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Touched: ", Style::default().fg(Theme::DIM)),
        Span::raw(node.touched.format("%Y-%m-%d %H:%M UTC").to_string()),
    ]));

    if !node.source_files.is_empty() {
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled("Sources: ", Style::default().fg(Theme::DIM)),
            Span::raw(node.source_files.join(", ")),
        ]));
    }

    // Content
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " ─── Content ──────────────────────────────────",
        Style::default().fg(Theme::HEADER),
    )));
    for line in node.content.lines() {
        lines.push(Line::from(format!("  {line}")));
    }

    // Edges
    if !node.edges.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " ─── Edges ────────────────────────────────────",
            Style::default().fg(Theme::HEADER),
        )));
        for (i, edge) in node.edges.iter().enumerate() {
            let selected = i == state.selected_edge;
            let marker = if selected { " ▶ → " } else { "   → " };
            let marker_style = if selected {
                Style::default().fg(Theme::HIGHLIGHT)
            } else {
                Style::default().fg(Theme::SECONDARY)
            };
            let id_style = if selected {
                Style::default().fg(Theme::SECONDARY).bold().underlined()
            } else {
                Style::default().fg(Theme::SECONDARY).bold()
            };
            lines.push(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(&edge.to, id_style),
                Span::styled(
                    format!("  [{:?}  w:{}]", edge.edge_type, edge.weight),
                    Style::default().fg(Theme::DIM),
                ),
            ]));
        }
    }

    // Data lake
    if !node.data_lake.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " ─── Data Lake ────────────────────────────────",
            Style::default().fg(Theme::HEADER),
        )));
        for file in &node.data_lake {
            lines.push(Line::from(format!("   📎 {file}")));
        }
    }

    let content_length = lines.len();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" {} ", node.id))
                .title_style(Style::default().fg(Theme::ACCENT).bold()),
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
