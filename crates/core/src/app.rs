use crate::cli::CLI;

use wind_view::editor::Editor;
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
        Ok(App {
            terminal: Terminal::new(CrosstermBackend::new(stdout()))?,
            editor: Editor::new(cli.file_path)?,
            painter: Painter::default(),
            message: AppMessage::None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.start_session()?;

        self.main_loop(&mut EventStream::new()).await?;

        self.end_session()?;

        Ok(())
    }

    fn start_session(&mut self) -> Result<()> {
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(())
    }

    fn end_session(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;

        Ok(())
    }

    async fn main_loop(&mut self, event_stream: &mut EventStream) -> Result<()> {
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
            Event::Resize(width, height) => self.terminal.resize(Rect::new(0, 0, width, height))?,

            Event::Key(key_event) => self.handle_key_event(key_event)?,

            _ => (),
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let text_area = self.painter.get_text_area(self.terminal.size()?);

        match key_event.code {
            KeyCode::Char(ch) => match ch {
                'q' => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        self.message = AppMessage::Exit;
                    }
                }

                _ => (),
            },

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
                    current_row_length
                        .saturating_sub(self.editor.position.column)
                        .saturating_sub(1),
                )?;
            }

            _ => (),
        }

        Ok(())
    }
}
