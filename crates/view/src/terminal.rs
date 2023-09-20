use anyhow::Result;

use crossterm::cursor::*;
use crossterm::event::*;
use crossterm::terminal::*;
use crossterm::ExecutableCommand;

use std::io;
use std::io::Write;

#[derive(Default)]
pub struct Terminal {}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {}
    }

    pub fn get_size(&self) -> io::Result<(u16, u16)> {
        size()
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

    pub fn move_cursor(&self, x: u16, y: u16) -> Result<()> {
        io::stdout().execute(MoveTo(x, y))?;
        Ok(())
    }

    pub fn clear_current_line(&self) -> Result<()> {
        io::stdout().execute(Clear(ClearType::CurrentLine))?;
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
