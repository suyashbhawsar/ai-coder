use anyhow::Result;
use crossterm::{
    event::{self, EnableMouseCapture, Event as CrosstermEvent, KeyCode, MouseEvent, MouseEventKind},
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use std::io;


pub use crossterm::event::KeyEvent;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Copy,    // Event for text copy operation
    ScrollUp,
    ScrollDown,
}

pub struct EventHandler {
    #[allow(dead_code)]
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut stdout = io::stdout();
                enable_raw_mode().expect("Failed to enable raw mode");
                stdout.execute(EnterAlternateScreen).expect("Failed to enter alternate screen");
                stdout.execute(EnableMouseCapture).expect("Failed to enable mouse capture");

                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or_else(|| Duration::from_secs(0));

                    if event::poll(timeout).expect("Failed to poll new events") {
                        match event::read().expect("Unable to read event") {
                            CrosstermEvent::Key(e) => {
                                // Handle scroll keys
                                match e.code {
                                    KeyCode::PageUp => {
                                        if let Err(err) = sender.send(Event::ScrollUp) {
                                            eprintln!("Error sending scroll up event: {}", err);
                                            break;
                                        }
                                    }
                                    KeyCode::PageDown => {
                                        if let Err(err) = sender.send(Event::ScrollDown) {
                                            eprintln!("Error sending scroll down event: {}", err);
                                            break;
                                        }
                                    }
                                    _ => {
                                        if let Err(err) = sender.send(Event::Key(e)) {
                                            eprintln!("Error sending key event: {}", err);
                                            break;
                                        }
                                    }
                                }
                            }
                            CrosstermEvent::Mouse(e) => {
                                // Handle mouse scroll events
                                match e.kind {
                                    MouseEventKind::ScrollUp => {
                                        if let Err(err) = sender.send(Event::ScrollUp) {
                                            eprintln!("Error sending scroll up event: {}", err);
                                            break;
                                        }
                                    }
                                    MouseEventKind::ScrollDown => {
                                        if let Err(err) = sender.send(Event::ScrollDown) {
                                            eprintln!("Error sending scroll down event: {}", err);
                                            break;
                                        }
                                    }
                                    _ => {
                                        if let Err(err) = sender.send(Event::Mouse(e)) {
                                            eprintln!("Error sending mouse event: {}", err);
                                            break;
                                        }
                                    }
                                }
                            }
                            CrosstermEvent::Resize(w, h) => {
                                if let Err(err) = sender.send(Event::Resize(w, h)) {
                                    eprintln!("Error sending resize event: {}", err);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        if let Err(err) = sender.send(Event::Tick) {
                            eprintln!("Error sending tick event: {}", err);
                            break;
                        }
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}
