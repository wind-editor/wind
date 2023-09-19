use crate::terminal::*;

use anyhow::Result;

use crossterm::event::*;

pub struct Editor {
    terminal: Terminal,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            terminal: Terminal::new(),
        }
    }

    pub fn run(&self) -> Result<()> {
        self.terminal.enter_alternative_screen()?;
        self.terminal.enable_raw_mode()?;
        self.handle_events()?;
        self.terminal.disable_raw_mode()?;
        self.terminal.leave_alternative_screen()?;
        Ok(())
    }

    pub fn handle_events(&self) -> Result<()> {
        loop {
            let event = self.terminal.wait_event()?;

            match event {
                Event::Key(key_event) => match self.handle_key(key_event) {
                    true => continue,
                    false => break Ok(()),
                },
                _ => continue,
            }
        }
    }

    pub fn handle_key(&self, key_event: KeyEvent) -> bool {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char(ch) => match ch {
                    'q' => return false,
                    _ => return true,
                },

                _ => return true,
            }
        }

        true
    }
}
