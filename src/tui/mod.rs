mod app;
pub mod event;
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

use app::{App, View};
use event::{Event, EventHandler};

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

    let mut app = App::new(nodes, engram_dir);
    let events = EventHandler::new(Duration::from_millis(250));

    // Main loop
    while app.running {
        terminal.draw(|frame| render(&app, frame))?;

        match events.next()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                handle_key(&mut app, key);
            }
            _ => {}
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: content + help bar at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    match app.view {
        View::NodeList => views::node_list::render(app, frame, chunks[0]),
        _ => {} // Other views added in later tasks
    }

    // Help bar
    let help = Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" quit  "),
        Span::styled("j/k", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" detail  "),
        Span::styled("/", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" search  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" back"),
    ]);
    frame.render_widget(help, chunks[1]);
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Up | KeyCode::Char('k') => app.previous(),
        _ => {}
    }
}
