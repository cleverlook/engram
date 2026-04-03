use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
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
    paused: Arc<AtomicBool>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let _tx = tx.clone();
        let paused = Arc::new(AtomicBool::new(false));
        let paused_flag = paused.clone();
        thread::spawn(move || {
            loop {
                if paused_flag.load(Ordering::Relaxed) {
                    thread::sleep(tick_rate);
                    continue;
                }
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
        Self { rx, _tx, paused }
    }

    pub fn next(&self) -> anyhow::Result<Event> {
        Ok(self.rx.recv()?)
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        // Drain any stale events accumulated during pause
        while self.rx.try_recv().is_ok() {}
        self.paused.store(false, Ordering::Relaxed);
    }
}
