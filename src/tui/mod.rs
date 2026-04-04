mod app;
mod event;
mod mutations;
mod views;

use std::io;
use std::path::Path;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;

use app::{App, EDGE_TYPES, Overlay, STATUSES, View};
use event::{Event, EventHandler};

enum TuiAction {
    Create,
    Edit,
    EditForm,
    Deprecate,
    AddEdge,
}

pub fn run(cwd: &Path) -> anyhow::Result<()> {
    let engram_dir = crate::storage::find_engram_dir(cwd)?;
    let nodes = crate::storage::load_all_nodes(&engram_dir)?;

    if nodes.is_empty() {
        println!("No nodes found. Create some with `engram node create` first.");
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the main loop, then always restore terminal — even on error
    let result = run_loop(&mut terminal, App::new(nodes, engram_dir));

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> anyhow::Result<()> {
    let events = EventHandler::new(Duration::from_millis(250));

    while app.running {
        terminal.draw(|frame| render(&mut app, frame))?;

        match events.next()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if let Some(action) = handle_key(&mut app, key) {
                    process_action(&mut app, terminal, &events, action)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: content + help bar at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    match app.view {
        View::NodeList => views::node_tree::render(app, frame, chunks[0]),
        View::NodeDetail => {
            if let Some(node) = app.nodes.get(app.selected_index) {
                views::node_detail::render(node, &app.detail_state, frame, chunks[0]);
            }
        }
        View::Search => views::search::render(&app.search_state, &app.nodes, frame, chunks[0]),
    }

    // Overlay
    match &app.overlay {
        Overlay::ConfirmDeprecate => {
            if let Some(node) = app.selected_node() {
                views::overlay::render_confirm_deprecate(&node.id, frame, chunks[0]);
            }
        }
        Overlay::CreateForm => {
            views::overlay::render_create_form(&mut app.create_form, frame, chunks[0]);
        }
        Overlay::AddEdgeForm => {
            views::overlay::render_add_edge_form(&app.add_edge_form, frame, chunks[0]);
        }
        Overlay::EditForm => {
            views::overlay::render_edit_form(&mut app.edit_form, frame, chunks[0]);
        }
        Overlay::None => {}
    }

    // Status message or help bar
    if let Some((msg, is_error)) = &app.status_message {
        let style = if *is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        frame.render_widget(
            Line::from(Span::styled(format!(" {msg}"), style)),
            chunks[1],
        );
    } else {
        let help = match app.view {
            View::NodeDetail => Line::from(vec![
                Span::styled(" q", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" quit  "),
                Span::styled("j/k", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" scroll  "),
                Span::styled("Tab", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" edge  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" follow  "),
                Span::styled("e", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" edit  "),
                Span::styled("E", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" raw-edit  "),
                Span::styled("d", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" deprecate  "),
                Span::styled("a", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" add edge  "),
                Span::styled("Bksp", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" back"),
            ]),
            _ => Line::from(vec![
                Span::styled(" q", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" quit  "),
                Span::styled("j/k", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" navigate  "),
                Span::styled("h/l", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" collapse/expand  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" open  "),
                Span::styled("/", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" search  "),
                Span::styled("s", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" sort  "),
                Span::styled("c", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" create  "),
                Span::styled("d", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" deprecate"),
            ]),
        };
        frame.render_widget(help, chunks[1]);
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Option<TuiAction> {
    // Clear status on any keypress
    app.clear_status();

    // Handle overlay input first
    match &app.overlay {
        Overlay::ConfirmDeprecate => {
            match key.code {
                KeyCode::Char('y') => return Some(TuiAction::Deprecate),
                KeyCode::Char('n') | KeyCode::Esc => app.close_overlay(),
                _ => {}
            }
            return None;
        }
        Overlay::CreateForm => {
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('s')
            {
                return Some(TuiAction::Create);
            }
            match key.code {
                KeyCode::Esc => app.close_overlay(),
                KeyCode::Tab => app.create_form.next_field(),
                KeyCode::BackTab => app.create_form.prev_field(),
                _ => {
                    app.create_form.handle_input(key);
                }
            }
            return None;
        }
        Overlay::AddEdgeForm => {
            match key.code {
                KeyCode::Esc => app.close_overlay(),
                KeyCode::Tab => app.add_edge_form.next_field(),
                KeyCode::BackTab => app.add_edge_form.prev_field(),
                KeyCode::Left if app.add_edge_form.focused_field == 1 => {
                    app.add_edge_form.prev_type();
                }
                KeyCode::Right if app.add_edge_form.focused_field == 1 => {
                    app.add_edge_form.next_type();
                }
                KeyCode::Down
                    if app.add_edge_form.focused_field == 0
                        && !app.add_edge_form.suggestions.is_empty() =>
                {
                    app.add_edge_form.next_suggestion();
                }
                KeyCode::Up
                    if app.add_edge_form.focused_field == 0
                        && !app.add_edge_form.suggestions.is_empty() =>
                {
                    app.add_edge_form.prev_suggestion();
                }
                KeyCode::Enter
                    if app.add_edge_form.focused_field == 0
                        && !app.add_edge_form.suggestions.is_empty() =>
                {
                    app.add_edge_form.accept_suggestion();
                }
                KeyCode::Enter => return Some(TuiAction::AddEdge),
                _ => {
                    app.add_edge_form.handle_input(key);
                    if app.add_edge_form.focused_field == 0 {
                        let node_ids: Vec<String> =
                            app.nodes.iter().map(|n| n.id.clone()).collect();
                        app.add_edge_form.update_suggestions(&node_ids);
                    }
                }
            }
            return None;
        }
        Overlay::EditForm => {
            // Ctrl+S saves, Enter is newline in content field
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('s')
            {
                return Some(TuiAction::EditForm);
            }
            match key.code {
                KeyCode::Esc => app.close_overlay(),
                KeyCode::Tab => app.edit_form.next_field(),
                KeyCode::BackTab => app.edit_form.prev_field(),
                KeyCode::Left if app.edit_form.focused_field == 2 => {
                    app.edit_form.prev_status();
                }
                KeyCode::Right if app.edit_form.focused_field == 2 => {
                    app.edit_form.next_status();
                }
                _ => {
                    app.edit_form.handle_input(key);
                }
            }
            return None;
        }
        Overlay::None => {}
    }

    // Normal view handling
    match app.view {
        View::NodeList => {
            match key.code {
                KeyCode::Char('q') => app.quit(),
                KeyCode::Down | KeyCode::Char('j') => {
                    app.tree_state.key_down();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.tree_state.key_up();
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    app.tree_state.key_right();
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    app.tree_state.key_left();
                }
                KeyCode::Enter => {
                    if views::node_tree::is_leaf_selected(app) {
                        // Navigate to detail view for the selected leaf node
                        if let Some(node) = views::node_tree::selected_node_from_tree(app) {
                            let node_id = node.id.clone();
                            if let Some(pos) = app.nodes.iter().position(|n| n.id == node_id) {
                                app.selected_index = pos;
                                app.enter_detail();
                            }
                        }
                    } else {
                        app.tree_state.toggle_selected();
                    }
                }
                KeyCode::Char('/') => app.enter_search(),
                KeyCode::Char('s') => app.cycle_sort(),
                KeyCode::Char('c') => app.open_create_form(),
                KeyCode::Char('d') => app.confirm_deprecate(),
                _ => {}
            }
        }
        View::NodeDetail => {
            let edge_count = app.selected_node().map(|n| n.edges.len()).unwrap_or(0);
            match key.code {
                KeyCode::Char('q') => app.quit(),
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => app.navigate_back(),
                KeyCode::Down | KeyCode::Char('j') => app.detail_state.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => app.detail_state.scroll_up(),
                KeyCode::Tab => app.detail_state.next_edge(edge_count),
                KeyCode::BackTab => app.detail_state.prev_edge(edge_count),
                KeyCode::Enter => app.navigate_to_edge(),
                KeyCode::Char('e') => app.open_edit_form(),
                KeyCode::Char('E') => return Some(TuiAction::Edit),
                KeyCode::Char('d') => app.confirm_deprecate(),
                KeyCode::Char('a') => app.open_add_edge_form(),
                _ => {}
            }
        }
        View::Search => match key.code {
            KeyCode::Esc => app.back(),
            KeyCode::Enter => app.open_search_result(),
            KeyCode::Down => app.search_state.next(),
            KeyCode::Up => app.search_state.previous(),
            _ => {
                use tui_input::backend::crossterm::EventHandler;
                app.search_state
                    .input
                    .handle_event(&crossterm::event::Event::Key(key));
                app.execute_search();
            }
        },
    }
    None
}

fn process_action(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    events: &EventHandler,
    action: TuiAction,
) -> anyhow::Result<()> {
    match action {
        TuiAction::Create => {
            let id = app.create_form.id_input.value().trim().to_string();
            let content = app.create_form.content_text();
            let weight: u8 = app
                .create_form
                .weight_input
                .value()
                .trim()
                .parse()
                .unwrap_or(50);
            app.close_overlay();
            match mutations::create_node(&app.engram_dir, &id, &content, weight) {
                Ok(_) => {
                    reload(app)?;
                    app.set_status(format!("Created '{id}'"), false);
                }
                Err(e) => app.set_status(format!("Error: {e}"), true),
            }
        }
        TuiAction::Edit => {
            if let Some(node) = app.selected_node().cloned() {
                let yaml = serde_yaml::to_string(&node).unwrap_or_default();

                // Suspend TUI and event handler
                events.pause();
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                terminal.show_cursor()?;

                let result = edit_in_editor(&yaml);

                // Resume TUI and event handler
                enable_raw_mode()?;
                execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                terminal.hide_cursor()?;
                terminal.clear()?;
                events.resume();

                match result {
                    Ok(new_yaml) => {
                        match mutations::update_node_from_yaml(&app.engram_dir, &node.id, &new_yaml)
                        {
                            Ok(_) => {
                                reload(app)?;
                                app.set_status(format!("Updated '{}'", node.id), false);
                            }
                            Err(e) => app.set_status(format!("Error: {e}"), true),
                        }
                    }
                    Err(e) => app.set_status(format!("{e}"), true),
                }
            }
        }
        TuiAction::EditForm => {
            let node_id = app.edit_form.node_id.clone();
            let content = app.edit_form.content_text();
            let weight: u8 = app
                .edit_form
                .weight_input
                .value()
                .trim()
                .parse()
                .unwrap_or(50);
            let status_str = STATUSES[app.edit_form.status_index];
            app.close_overlay();
            match mutations::update_node_fields(
                &app.engram_dir,
                &node_id,
                &content,
                weight,
                status_str,
            ) {
                Ok(_) => {
                    reload(app)?;
                    app.set_status(format!("Updated '{node_id}'"), false);
                }
                Err(e) => app.set_status(format!("Error: {e}"), true),
            }
        }
        TuiAction::Deprecate => {
            if let Some(node) = app.selected_node().cloned() {
                app.close_overlay();
                match mutations::deprecate_node(&app.engram_dir, &node.id) {
                    Ok(()) => {
                        reload(app)?;
                        app.set_status(format!("Deprecated '{}'", node.id), false);
                    }
                    Err(e) => app.set_status(format!("Error: {e}"), true),
                }
            }
        }
        TuiAction::AddEdge => {
            if let Some(node) = app.selected_node().cloned() {
                let target = app.add_edge_form.target_input.value().trim().to_string();
                let edge_type_str = EDGE_TYPES[app.add_edge_form.edge_type_index];
                let weight: u8 = app
                    .add_edge_form
                    .weight_input
                    .value()
                    .trim()
                    .parse()
                    .unwrap_or(50);
                app.close_overlay();
                let edge = parse_edge_type(edge_type_str, &target, weight);
                match mutations::add_edge(&app.engram_dir, &node.id, edge) {
                    Ok(_) => {
                        reload(app)?;
                        app.set_status(format!("Added edge → {target}"), false);
                    }
                    Err(e) => app.set_status(format!("Error: {e}"), true),
                }
            }
        }
    }
    Ok(())
}

fn reload(app: &mut App) -> anyhow::Result<()> {
    let nodes = crate::storage::load_all_nodes(&app.engram_dir)?;
    app.reload_nodes(nodes);
    Ok(())
}

fn edit_in_editor(template: &str) -> anyhow::Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let tmp = std::env::temp_dir().join(format!("engram-tui-{}.yaml", std::process::id()));
    std::fs::write(&tmp, template)?;
    let status = std::process::Command::new(&editor).arg(&tmp).status()?;
    if !status.success() {
        std::fs::remove_file(&tmp).ok();
        anyhow::bail!("Editor exited with error");
    }
    let content = std::fs::read_to_string(&tmp)?;
    std::fs::remove_file(&tmp).ok();
    if content.trim().is_empty() || content == template {
        anyhow::bail!("Aborted: no changes made");
    }
    Ok(content)
}

fn parse_edge_type(type_str: &str, target: &str, weight: u8) -> crate::models::node::Edge {
    use crate::models::node::{Edge, EdgeType};
    let edge_type = match type_str {
        "uses" => EdgeType::Uses,
        "depends_on" => EdgeType::DependsOn,
        "implements" => EdgeType::Implements,
        "rationale" => EdgeType::Rationale,
        _ => EdgeType::Related,
    };
    Edge {
        to: target.to_string(),
        edge_type,
        weight,
    }
}
