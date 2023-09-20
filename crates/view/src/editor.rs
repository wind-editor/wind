use crate::terminal::*;

use anyhow::Result;

use crossterm::event::*;

struct Position(u16, u16);

pub struct Editor {
    terminal: Terminal,
    position: Position,
    quit: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            terminal: Terminal::default(),
            position: Position(0, 0),
            quit: false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.terminal.enter_alternative_screen()?;
        self.terminal.enable_raw_mode()?;

        loop {
            self.terminal
                .move_cursor(self.position.1, self.position.0)?;
            self.terminal.clear_all()?;
            self.handle_events()?;

            if self.quit {
                break;
            }
        }

        self.terminal.disable_raw_mode()?;
        self.terminal.leave_alternative_screen()?;

        Ok(())
    }

    pub fn handle_events(&mut self) -> Result<()> {
        let event = self.terminal.wait_event()?;

        match event {
            Event::Key(key_event) => self.handle_key(key_event)?,
            _ => (),
        };

        Ok(())
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) -> Result<()> {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char(ch) => match ch {
                    'q' => self.quit = true,
                    _ => (),
                },

                _ => (),
            };

            Ok(())
        } else {
            match key_event.code {
                KeyCode::Up => self.handle_arrow_key(key_event.code)?,
                KeyCode::Down => self.handle_arrow_key(key_event.code)?,
                KeyCode::Left => self.handle_arrow_key(key_event.code)?,
                KeyCode::Right => self.handle_arrow_key(key_event.code)?,

                _ => (),
            };

            Ok(())
        }
    }

    pub fn handle_arrow_key(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Up => self.position.0 = self.position.0.saturating_sub(1),
            KeyCode::Down => {
                if self.position.0 < self.terminal.get_size()?.1 {
                    self.position.0 = self.position.0.saturating_add(1)
                }
            }
            KeyCode::Left => self.position.1 = self.position.1.saturating_sub(1),
            KeyCode::Right => {
                if self.position.1 < self.terminal.get_size()?.0 {
                    self.position.1 = self.position.1.saturating_add(1)
                }
            }

            _ => (),
        };

        Ok(())
    }
}
