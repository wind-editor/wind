use crate::cli::CLI;

use wind_view::editor::{Editor, EditorMode};
use wind_view::painter::Painter;

use anyhow::Result;

use crossterm::event::*;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use futures_util::StreamExt;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

use std::io::{stdout, Stdout};

#[derive(Clone, Copy)]
pub enum AppMessage {
    Exit,
    None,
}

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    editor: Editor,
    painter: Painter,
    message: AppMessage,
}

impl App {
    pub fn new(cli: CLI) -> Result<App> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let painter = Painter::new(terminal.size()?);

        Ok(App {
            terminal,
            editor: Editor::new(cli.file_path)?,
            painter,
            message: AppMessage::None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.start_session()?;

        self.main_loop().await?;

        self.end_session()?;

        Ok(())
    }

    fn start_session(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;

        Ok(())
    }

    fn end_session(&mut self) -> Result<()> {
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(())
    }

    async fn main_loop(&mut self) -> Result<()> {
        let mut event_stream = EventStream::new();

        loop {
            self.painter.paint(&mut self.terminal, &self.editor)?;

            if let Some(Ok(event)) = event_stream.next().await {
                self.handle_terminal_event(event)?;
            }

            match self.message {
                AppMessage::Exit => break,
                AppMessage::None => (),
            }
        }

        Ok(())
    }

    fn handle_terminal_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Resize(width, height) => {
                self.terminal.resize(Rect::new(0, 0, width, height))?;

                self.painter.recompute_areas(self.terminal.size()?);
            }

            Event::Key(key_event) => self.handle_key_event(key_event)?,

            _ => (),
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let text_area = self.painter.get_text_area();

        match key_event.code {
            KeyCode::Up => self.editor.move_up(text_area, 1)?,

            KeyCode::Down => self.editor.move_down(text_area, 1)?,

            KeyCode::Left => self.editor.move_left(text_area, 1)?,

            KeyCode::Right => self.editor.move_right(text_area, 1)?,

            KeyCode::Home => self
                .editor
                .move_left(text_area, self.editor.position.column)?,

            KeyCode::End => {
                let current_row_length = self.editor.document.row_len(self.editor.position.row);

                self.editor.move_right(
                    text_area,
                    current_row_length.saturating_sub(self.editor.position.column),
                )?;
            }

            _ => {
                match self.editor.mode {
                    EditorMode::Normal => match key_event.code {
                        KeyCode::Char('q') => {
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                self.message = AppMessage::Exit;
                            }
                        }

                        KeyCode::Char('i') => {
                            self.editor.mode = EditorMode::Insert;
                        }

                        _ => (),
                    },

                    EditorMode::Insert => match key_event.code {
                        KeyCode::Char(ch) => {
                            self.editor.document.insert(self.editor.position, ch);

                            self.editor.move_right(text_area, 1)?;
                        }

                        KeyCode::Enter => {
                            self.editor.document.insert(self.editor.position, '\n');

                            self.editor.move_right(text_area, 1)?;
                        }

                        KeyCode::Delete => self.editor.document.delete(self.editor.position),

                        KeyCode::Backspace => {
                            if self.editor.position.row > 0 || self.editor.position.column > 0 {
                                self.editor.move_left(text_area, 1)?;

                                self.editor.document.delete(self.editor.position);
                            }
                        }

                        KeyCode::Esc => {
                            self.editor.mode = EditorMode::Normal;
                        }

                        _ => (),
                    },
                };
            }
        };

        Ok(())
    }
}
