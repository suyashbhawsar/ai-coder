use anyhow::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::{self, stdout};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::event::EventHandler;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    events: EventHandler,
    raw_mode_enabled: bool,
}

impl Clone for Tui {
    fn clone(&self) -> Self {
        // This is a shallow clone just to satisfy the compiler
        // We're not actually cloning the terminal
        panic!("Tui should not be cloned - this is just to satisfy the compiler");
    }
}

impl Tui {
    pub fn new(tick_rate: u64) -> io::Result<Self> {
        let mut stdout = stdout();

        enable_raw_mode()?;
        stdout.execute(EnterAlternateScreen)?;
        // Enable mouse capture for proper scroll handling
        stdout.execute(EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new(tick_rate);

        Ok(Self {
            terminal,
            events,
            raw_mode_enabled: true,
        })
    }
    
    // Force an immediate redraw of the UI
    pub fn immediate_refresh<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(f)?;
        // Ensure the terminal flushes the buffer
        self.terminal.backend_mut().flush()?;
        Ok(())
    }

    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }

    pub fn events(&self) -> &EventHandler {
        &self.events
    }

    pub fn toggle_raw_mode(&mut self) -> io::Result<()> {
        if self.raw_mode_enabled {
            disable_raw_mode()?;
            self.terminal.backend_mut().execute(DisableMouseCapture)?;
            self.raw_mode_enabled = false;
        } else {
            enable_raw_mode()?;
            self.terminal.backend_mut().execute(EnableMouseCapture)?;
            self.raw_mode_enabled = true;
        }
        Ok(())
    }

    pub fn init(&mut self) -> Result<()> {
        enable_raw_mode()?;
        // Enable full mouse reporting including drag events
        crossterm::execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;

        self.terminal.clear()?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        crossterm::execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        Ok(())
    }

    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        if self.raw_mode_enabled {
            disable_raw_mode().unwrap();
        }
        self.terminal
            .backend_mut()
            .execute(DisableMouseCapture).unwrap();
        self.terminal
            .backend_mut()
            .execute(LeaveAlternateScreen).unwrap();
    }
}
