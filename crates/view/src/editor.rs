use crate::terminal::*;

use anyhow::Result;

pub struct Editor {
    terminal: Terminal
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            terminal: Terminal::new()
        }
    }

    pub fn run(&self) -> Result<()> {
        self.terminal.enter_alternative_screen()?;
        self.terminal.enable_raw_mode()?;
        self.terminal.clear_all()?;
        self.terminal.wait_event()?;
        self.terminal.disable_raw_mode()?;
        self.terminal.leave_alternative_screen()?;
        Ok(())
    }
}
