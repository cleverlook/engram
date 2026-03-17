use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::Receiver<Event>,
    // Keep handle so thread lives as long as handler
    _tx: mpsc::Sender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let _tx = tx.clone();
        thread::spawn(move || {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    if let Ok(CrosstermEvent::Key(key)) = event::read()
                        && tx.send(Event::Key(key)).is_err()
                    {
                        break;
                    }
                } else if tx.send(Event::Tick).is_err() {
                    break;
                }
            }
        });
        Self { rx, _tx }
    }

    pub fn next(&self) -> anyhow::Result<Event> {
        Ok(self.rx.recv()?)
    }
}
