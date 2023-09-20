use crate::document::*;
use crate::position::*;
use crate::terminal::*;

use anyhow::Result;

use crossterm::event::*;

use std::path::PathBuf;

#[derive(Default)]
pub struct Editor {
    terminal: Terminal,
    document: Document,
    position: Position,
    quit: bool,
}

impl Editor {
    pub fn new(file: Option<PathBuf>) -> Editor {
        Editor {
            terminal: Terminal::default(),
            document: Document::open(file).unwrap_or_default(),
            position: Position::default(),
            quit: false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.terminal.enter_alternative_screen()?;
        self.terminal.enable_raw_mode()?;

        loop {
            self.draw_rows()?;
            self.terminal
                .move_cursor(self.position.column, self.position.row)?;
            self.handle_events()?;

            if self.quit {
                break;
            }
        }

        self.terminal.disable_raw_mode()?;
        self.terminal.leave_alternative_screen()?;

        Ok(())
    }

    fn draw_rows(&self) -> Result<()> {
        for empty_row in 0..self.terminal.get_size()?.1 {
            self.terminal.clear_current_line()?;

            if let Some(row) = self.document.row(empty_row as usize) {
                self.draw_row(row)?;
            } else {
                println!("~\r");
            }
        }

        Ok(())
    }

    fn draw_row(&self, row: &Row) -> Result<()> {
        let start = 0;
        let end = self.terminal.get_size()?.0;
        println!("{}\r", row.render(start as usize, end as usize));

        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        let event = self.terminal.wait_event()?;

        match event {
            Event::Key(key_event) => self.handle_key(key_event)?,
            _ => (),
        };

        Ok(())
    }

    fn handle_key(&mut self, key_event: KeyEvent) -> Result<()> {
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

    fn handle_arrow_key(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Up => {
                self.position.row = self.position.row.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.position.row < self.terminal.get_size()?.1 as usize {
                    self.position.row = self.position.row.saturating_add(1);
                }
            }
            KeyCode::Left => self.position.column = self.position.column.saturating_sub(1),
            KeyCode::Right => {
                if self.document.row_length(self.position.row + 1) > self.position.column {
                    self.position.column = self.position.column.saturating_add(1);
                }
            }

            _ => (),
        };

        Ok(())
    }
}
