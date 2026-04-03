use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::tui::app::{AddEdgeFormState, CreateFormState, EDGE_TYPES};

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

pub fn render_confirm_deprecate(node_id: &str, frame: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 5, area);
    frame.render_widget(Clear, popup);
    let text = format!("Deprecate '{}'?\n\n[y] Yes  [n/Esc] Cancel", node_id);
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm Deprecate "),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup);
}

pub fn render_create_form(state: &CreateFormState, frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 11, area);
    frame.render_widget(Clear, popup);

    let fields = [
        ("ID:", state.id_input.value(), 0),
        ("Content:", state.content_input.value(), 1),
        ("Weight:", state.weight_input.value(), 2),
    ];

    let mut lines = vec![Line::from("")];
    for (label, value, idx) in &fields {
        let marker = if *idx == state.focused_field {
            "▶ "
        } else {
            "  "
        };
        let style = if *idx == state.focused_field {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(vec![
            Span::styled(marker, style),
            Span::styled(*label, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::raw(*value),
        ]));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        "  Tab: next field  Enter: create  Esc: cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Create Node "),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup);
}

pub fn render_add_edge_form(state: &AddEdgeFormState, frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 11, area);
    frame.render_widget(Clear, popup);

    let edge_type_display = EDGE_TYPES[state.edge_type_index];

    let mut lines = vec![Line::from("")];

    // Target field
    let m0 = if state.focused_field == 0 {
        "▶ "
    } else {
        "  "
    };
    let s0 = if state.focused_field == 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    lines.push(Line::from(vec![
        Span::styled(m0, s0),
        Span::styled("Target:", Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::raw(state.target_input.value()),
    ]));
    lines.push(Line::from(""));

    // Type field
    let m1 = if state.focused_field == 1 {
        "▶ "
    } else {
        "  "
    };
    let s1 = if state.focused_field == 1 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    lines.push(Line::from(vec![
        Span::styled(m1, s1),
        Span::styled("Type:", Style::default().fg(Color::Cyan)),
        Span::raw(format!(" ◀ {} ▶", edge_type_display)),
    ]));
    lines.push(Line::from(""));

    // Weight field
    let m2 = if state.focused_field == 2 {
        "▶ "
    } else {
        "  "
    };
    let s2 = if state.focused_field == 2 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    lines.push(Line::from(vec![
        Span::styled(m2, s2),
        Span::styled("Weight:", Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::raw(state.weight_input.value()),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "  Tab: next  ←/→: type  Enter: add  Esc: cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Add Edge "))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup);
}
