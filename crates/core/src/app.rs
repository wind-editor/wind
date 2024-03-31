use crate::cli::CLI;

use wind_view::editor::Editor;
use wind_view::painter::Painter;

use anyhow::Result;

use crossterm::event::*;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use futures_util::Stream;
use futures_util::StreamExt;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

use std::io::{stdout, Stdout};

#[derive(Clone, Copy)]
pub enum AppMessage {
    Exit,
}

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    editor: Editor,
    painter: Painter,
    message: Option<AppMessage>,
}

impl App {
    pub fn new(cli: CLI) -> Result<App> {
        Ok(App {
            terminal: Terminal::new(CrosstermBackend::new(stdout()))?,
            editor: Editor::new(cli.file_path)?,
            painter: Painter::default(),
            message: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.start_terminal()?;
        self.event_loop(&mut EventStream::new()).await?;
        self.end_terminal()?;

        Ok(())
    }

    async fn event_loop<S>(&mut self, event_stream: &mut S) -> Result<()>
    where
        S: Stream<Item = std::io::Result<Event>> + Unpin,
    {
        loop {
            self.painter.paint(&mut self.terminal, &self.editor)?;

            self.handle_event(event_stream).await?;

            if let Some(message) = self.message {
                match message {
                    AppMessage::Exit => break,
                };
            }
        }

        Ok(())
    }

    async fn handle_event<S>(&mut self, event_stream: &mut S) -> Result<()>
    where
        S: Stream<Item = std::io::Result<Event>> + Unpin,
    {
        loop {
            tokio::select! {
                Some(event) = event_stream.next() => {
                    return self.handle_terminal_event(event);
                }
            }
        }
    }

    fn handle_terminal_event(&mut self, event: std::io::Result<Event>) -> Result<()> {
        match event {
            Ok(event) => match event {
                Event::Resize(width, height) => {
                    self.terminal.resize(Rect::new(0, 0, width, height))?;

                    Ok(())
                }

                Event::Key(key_event) => self.handle_key_event(key_event),

                _ => Ok(()),
            },

            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let text_area = self.painter.get_text_area(self.terminal.size()?);

        match key_event.code {
            KeyCode::Char(ch) => match ch {
                'q' => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        self.message = Some(AppMessage::Exit);
                    }

                    Ok(())
                }

                _ => Ok(()),
            },

            KeyCode::Up => self.editor.move_up(text_area, 1),

            KeyCode::Down => self.editor.move_down(text_area, 1),

            KeyCode::Left => self.editor.move_left(text_area, 1),

            KeyCode::Right => self.editor.move_right(text_area, 1),

            KeyCode::Home => self
                .editor
                .move_left(text_area, self.editor.position.column),

            KeyCode::End => {
                let current_row_length = self.editor.document.row_length(self.editor.position.row);

                self.editor.move_right(
                    text_area,
                    current_row_length
                        .saturating_sub(self.editor.position.column)
                        .saturating_sub(1),
                )
            }

            _ => Ok(()),
        }
    }

    fn start_terminal(&mut self) -> Result<()> {
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(())
    }

    fn end_terminal(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;

        Ok(())
    }
}
