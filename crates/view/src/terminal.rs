use anyhow::Result;

use crossterm::event::*;
use crossterm::terminal::*;
use crossterm::ExecutableCommand;

use std::io;
use std::io::{Read, Write};

pub struct Terminal {}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {}
    }

    pub fn wait_event(&self) -> io::Result<Event> {
        read()
    }

    pub fn enable_raw_mode(&self) -> io::Result<()> {
        enable_raw_mode()
    }

    pub fn disable_raw_mode(&self) -> io::Result<()> {
        disable_raw_mode()
    }

    pub fn enter_alternative_screen(&self) -> Result<()> {
        io::stdout().execute(EnterAlternateScreen)?;

        Ok(())
    }

    pub fn leave_alternative_screen(&self) -> Result<()> {
        io::stdout().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        io::stdout().execute(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn flush(&self) -> io::Result<()> {
        io::stdout().flush()
    }
}
