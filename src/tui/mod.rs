mod app;
pub mod event;

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
    // Placeholder — will be replaced in Task 4
    let area = frame.area();
    let text = format!(
        "engram tui — {} nodes loaded. Press q to quit.\nSelected: {}",
        app.nodes.len(),
        app.selected_node().map(|n| n.id.as_str()).unwrap_or("none"),
    );
    frame.render_widget(ratatui::widgets::Paragraph::new(text), area);
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Up | KeyCode::Char('k') => app.previous(),
        _ => {}
    }
}
