use crate::cli::Arguments;
use wind_view::editor::Editor;

use anyhow::Result;

use crossterm::event::*;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use futures_util::Stream;
use futures_util::StreamExt;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

use std::io::stdout;

type TerminalBackend = CrosstermBackend<std::io::Stdout>;

type Terminal = ratatui::terminal::Terminal<TerminalBackend>;

pub struct Application {
    terminal: Terminal,
    editor: Editor,
}

impl Application {
    pub fn new(args: Arguments) -> Result<Application> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Application {
            terminal,
            editor: Editor::new(args.file),
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
            self.draw().await?;

            if !self.handle_event(event_stream).await? {
                break;
            }
        }

        Ok(())
    }

    async fn handle_event<S>(&mut self, event_stream: &mut S) -> Result<bool>
    where
        S: Stream<Item = std::io::Result<Event>> + Unpin,
    {
        loop {
            tokio::select! {
                Some(event) = event_stream.next() => {
                    return self.handle_terminal_event(event).await;
                }
            }
        }
    }

    async fn handle_terminal_event(&mut self, event: std::io::Result<Event>) -> Result<bool> {
        Ok(match event {
            Ok(event) => match event {
                Event::Resize(width, height) => {
                    self.terminal.resize(Rect::new(0, 0, width, height))?;
                    true
                }
                Event::Key(key_event) => self.handle_key_event(key_event).await?,
                _ => true,
            },
            _ => true,
        })
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool> {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char(ch) => match ch {
                    'q' => Ok(false),
                    _ => Ok(true),
                },

                _ => Ok(true),
            }
        } else {
            match key_event.code {
                KeyCode::Up => self.handle_arrow_key(key_event.code).await,
                KeyCode::Down => self.handle_arrow_key(key_event.code).await,
                KeyCode::Left => self.handle_arrow_key(key_event.code).await,
                KeyCode::Right => self.handle_arrow_key(key_event.code).await,

                _ => Ok(true),
            }
        }
    }

    async fn handle_arrow_key(&mut self, key_code: KeyCode) -> Result<bool> {
        match key_code {
            KeyCode::Up => {
                self.editor.position.row = self.editor.position.row.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.editor.position.row < self.terminal.size()?.height as usize {
                    self.editor.position.row = self.editor.position.row.saturating_add(1);
                }
            }
            KeyCode::Left => {
                self.editor.position.column = self.editor.position.column.saturating_sub(1)
            }
            KeyCode::Right => {
                if self.editor.document.row_length(self.editor.position.row)
                    > self.editor.position.column
                {
                    self.editor.position.column = self.editor.position.column.saturating_add(1);
                }
            }

            _ => (),
        };

        Ok(true)
    }

    async fn draw(&mut self) -> Result<()> {
        self.draw_document().await?;

        self.terminal.set_cursor(
            self.editor.position.column as u16,
            self.editor.position.row as u16,
        )?;

        Ok(())
    }

    async fn draw_document(&mut self) -> Result<()> {
        let rows: Vec<Line> = self
            .editor
            .document
            .rows
            .iter()
            .map(|r| Line::from(Span::from(r.content.clone())))
            .collect();

        let paragraph = Paragraph::new(rows);
        let block = Block::default();

        let width = self.terminal.size()?.width;
        let height = self.terminal.size()?.height;

        let y = self.terminal.size()?.y;

        self.terminal.draw(|f| {
            f.render_widget(
                paragraph.block(block),
                Rect {
                    width,
                    height,

                    x: 0,
                    y,
                },
            );

            f.set_cursor(
                self.editor.position.column as u16,
                self.editor.position.row as u16,
            );
        })?;

        Ok(())
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
