use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use crate::tui::app::{AddEdgeFormState, CreateFormState, EDGE_TYPES, EditFormState, STATUSES};
use crate::tui::theme::Theme;

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

pub fn render_confirm_deprecate(node_id: &str, frame: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 7, area);
    frame.render_widget(Clear, popup);
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ⚠ ", Style::default().fg(Theme::WARNING)),
            Span::raw("Deprecate "),
            Span::styled(node_id, Style::default().fg(Theme::ACCENT).bold()),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[y]", Style::default().fg(Theme::SUCCESS).bold()),
            Span::raw(" Yes   "),
            Span::styled("[n/Esc]", Style::default().fg(Theme::ERROR).bold()),
            Span::raw(" Cancel"),
        ]),
    ];
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Confirm Deprecate ")
                .title_style(Style::default().fg(Theme::HIGHLIGHT).bold()),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup);
}

pub fn render_create_form(state: &mut CreateFormState, frame: &mut Frame, area: Rect) {
    let width = (area.width).min(80).max(40);
    let height = (area.height).min(28).max(14);
    let popup = centered_rect(width, height, area);
    frame.render_widget(Clear, popup);

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Create Node ")
        .title_style(Style::default().fg(Theme::ACCENT).bold());
    let inner = outer.inner(popup);
    frame.render_widget(outer, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // id field
            Constraint::Length(1), // blank
            Constraint::Min(3),    // content textarea
            Constraint::Length(1), // weight
            Constraint::Length(1), // blank
            Constraint::Length(1), // help
        ])
        .split(inner);

    // ID field
    let m0 = if state.focused_field == 0 {
        "▶ "
    } else {
        "  "
    };
    let s0 = if state.focused_field == 0 {
        Style::default().fg(Theme::HIGHLIGHT)
    } else {
        Style::default().fg(Theme::DIM)
    };
    let id_line = Line::from(vec![
        Span::styled(m0, s0),
        Span::styled("ID: ", Style::default().fg(Theme::SECONDARY)),
        Span::raw(state.id_input.value()),
    ]);
    frame.render_widget(id_line, chunks[0]);

    // Content textarea
    let content_focused = state.focused_field == 1;
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if content_focused {
            Style::default().fg(Theme::HIGHLIGHT)
        } else {
            Style::default().fg(Theme::DIM)
        })
        .title(if content_focused {
            " Content (editing) "
        } else {
            " Content "
        });
    state.content_textarea.set_block(content_block);
    state.content_textarea.set_cursor_style(if content_focused {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    });
    frame.render_widget(&state.content_textarea, chunks[2]);

    // Weight field
    let m2 = if state.focused_field == 2 {
        "▶ "
    } else {
        "  "
    };
    let s2 = if state.focused_field == 2 {
        Style::default().fg(Theme::HIGHLIGHT)
    } else {
        Style::default().fg(Theme::DIM)
    };
    let weight_line = Line::from(vec![
        Span::styled(m2, s2),
        Span::styled("Weight: ", Style::default().fg(Theme::SECONDARY)),
        Span::raw(state.weight_input.value()),
    ]);
    frame.render_widget(weight_line, chunks[3]);

    // Help
    let help = Line::from(Span::styled(
        " Tab: next field  Ctrl+S: create  Esc: cancel",
        Style::default().fg(Theme::DIM),
    ));
    frame.render_widget(help, chunks[5]);
}

pub fn render_add_edge_form(state: &AddEdgeFormState, frame: &mut Frame, area: Rect) {
    let suggestion_count = state.suggestions.len();
    let height = 11 + suggestion_count.min(8) as u16;
    let popup = centered_rect(60, height, area);
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
        Style::default().fg(Theme::HIGHLIGHT)
    } else {
        Style::default().fg(Theme::DIM)
    };
    lines.push(Line::from(vec![
        Span::styled(m0, s0),
        Span::styled("Target:", Style::default().fg(Theme::SECONDARY)),
        Span::raw(" "),
        Span::raw(state.target_input.value()),
    ]));

    // Suggestions dropdown
    if !state.suggestions.is_empty() && state.focused_field == 0 {
        for (i, suggestion) in state.suggestions.iter().enumerate() {
            let is_selected = i == state.selected_suggestion;
            let prefix = if is_selected { "   ▶ " } else { "     " };
            let style = if is_selected {
                Style::default().fg(Theme::TEXT).bg(Theme::SELECTED_BG)
            } else {
                Style::default().fg(Theme::DIM)
            };
            lines.push(Line::from(Span::styled(
                format!("{prefix}{suggestion}"),
                style,
            )));
        }
    }

    lines.push(Line::from(""));

    // Type field
    let m1 = if state.focused_field == 1 {
        "▶ "
    } else {
        "  "
    };
    let s1 = if state.focused_field == 1 {
        Style::default().fg(Theme::HIGHLIGHT)
    } else {
        Style::default().fg(Theme::DIM)
    };
    lines.push(Line::from(vec![
        Span::styled(m1, s1),
        Span::styled("Type:", Style::default().fg(Theme::SECONDARY)),
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
        Style::default().fg(Theme::HIGHLIGHT)
    } else {
        Style::default().fg(Theme::DIM)
    };
    lines.push(Line::from(vec![
        Span::styled(m2, s2),
        Span::styled("Weight:", Style::default().fg(Theme::SECONDARY)),
        Span::raw(" "),
        Span::raw(state.weight_input.value()),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "  Tab: next  ↑/↓: suggest  Enter: select  Esc: cancel",
        Style::default().fg(Theme::DIM),
    )));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Add Edge ")
                .title_style(Style::default().fg(Theme::ACCENT).bold()),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup);
}

pub fn render_edit_form(state: &mut EditFormState, frame: &mut Frame, area: Rect) {
    // Use most of the screen for the edit form
    let width = (area.width).min(80).max(40);
    let height = (area.height).min(30).max(15);
    let popup = centered_rect(width, height, area);
    frame.render_widget(Clear, popup);

    // Outer block
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" Edit: {} ", state.node_id))
        .title_style(Style::default().fg(Theme::ACCENT).bold());
    let inner = outer.inner(popup);
    frame.render_widget(outer, popup);

    // Layout: node id (1) + content textarea (fill) + weight row (1) + status row (1) + help (1) + spacing (2)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // node id + blank
            Constraint::Min(3),    // content textarea
            Constraint::Length(1), // weight
            Constraint::Length(1), // blank
            Constraint::Length(1), // status
            Constraint::Length(1), // blank
            Constraint::Length(1), // help
        ])
        .split(inner);

    // Node ID (read-only)
    let id_line = Line::from(vec![
        Span::styled(" Node: ", Style::default().fg(Theme::DIM)),
        Span::styled(&state.node_id, Style::default().fg(Theme::ACCENT).bold()),
    ]);
    frame.render_widget(id_line, chunks[0]);

    // Content textarea
    let content_focused = state.focused_field == 0;
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if content_focused {
            Style::default().fg(Theme::HIGHLIGHT)
        } else {
            Style::default().fg(Theme::DIM)
        })
        .title(if content_focused {
            " Content (editing) "
        } else {
            " Content "
        });
    state.content_textarea.set_block(content_block);
    state.content_textarea.set_cursor_style(if content_focused {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    });
    frame.render_widget(&state.content_textarea, chunks[1]);

    // Weight field
    let m1 = if state.focused_field == 1 {
        "▶ "
    } else {
        "  "
    };
    let s1 = if state.focused_field == 1 {
        Style::default().fg(Theme::HIGHLIGHT)
    } else {
        Style::default().fg(Theme::DIM)
    };
    let weight_line = Line::from(vec![
        Span::styled(m1, s1),
        Span::styled("Weight: ", Style::default().fg(Theme::SECONDARY)),
        Span::raw(state.weight_input.value()),
    ]);
    frame.render_widget(weight_line, chunks[2]);

    // Status field
    let m2 = if state.focused_field == 2 {
        "▶ "
    } else {
        "  "
    };
    let s2 = if state.focused_field == 2 {
        Style::default().fg(Theme::HIGHLIGHT)
    } else {
        Style::default().fg(Theme::DIM)
    };
    let status_display = STATUSES[state.status_index];
    let status_line = Line::from(vec![
        Span::styled(m2, s2),
        Span::styled("Status: ", Style::default().fg(Theme::SECONDARY)),
        Span::raw(format!("◀ {} ▶", status_display)),
    ]);
    frame.render_widget(status_line, chunks[4]);

    // Help
    let help = Line::from(Span::styled(
        " Tab: next field  ←/→: status  Ctrl+S: save  Esc: cancel",
        Style::default().fg(Theme::DIM),
    ));
    frame.render_widget(help, chunks[6]);
}
