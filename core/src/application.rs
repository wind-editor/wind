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
                KeyCode::Up => {
                    self.move_up(1).await?;

                    Ok(true)
                }

                KeyCode::Down => {
                    self.move_down(1).await?;

                    Ok(true)
                }

                KeyCode::Left => {
                    self.move_left(1).await?;

                    Ok(true)
                }

                KeyCode::Right => {
                    self.move_right(1).await?;

                    Ok(true)
                }

                KeyCode::Home => {
                    self.move_left(self.editor.position.column).await?;

                    Ok(true)
                }

                KeyCode::End => {
                    let current_row_length =
                        self.editor.document.row_length(self.editor.position.row);

                    self.move_right(
                        current_row_length
                            .saturating_sub(self.editor.position.column)
                            .saturating_sub(1),
                    )
                    .await?;

                    Ok(true)
                }

                _ => Ok(true),
            }
        }
    }

    async fn move_up(&mut self, offset: usize) -> Result<()> {
        let terminal_width = self.terminal.size()?.width as usize;

        if self.editor.position.row > 0 {
            if self.editor.position.row <= self.editor.scroll_offset.row {
                self.editor.scroll_offset.row =
                    self.editor.scroll_offset.row.saturating_sub(offset);
            }

            self.editor.position.row -= offset;

            self.editor.position.column = self.editor.position.history.column.min(
                self.editor
                    .document
                    .row_length(self.editor.position.row)
                    .saturating_sub(1),
            );

            if self.editor.position.column < self.editor.scroll_offset.column {
                self.editor.scroll_offset.column = 0;
            } else if self.editor.position.column
                >= self.editor.scroll_offset.column + terminal_width
            {
                self.editor.scroll_offset.column = self.editor.position.column - terminal_width + 1;
            }
        }

        Ok(())
    }

    async fn move_down(&mut self, offset: usize) -> Result<()> {
        let terminal_width = self.terminal.size()?.width as usize;

        let terminal_height = self.terminal.size()?.height as usize;

        if self.editor.position.row.saturating_add(offset) < self.editor.document.rows.len() {
            if self.editor.position.row >= self.editor.scroll_offset.row + terminal_height - offset
            {
                self.editor.scroll_offset.row += offset;
            }

            self.editor.position.row += offset;

            self.editor.position.column = self.editor.position.history.column.min(
                self.editor
                    .document
                    .row_length(self.editor.position.row)
                    .saturating_sub(1),
            );

            if self.editor.position.column < self.editor.scroll_offset.column {
                self.editor.scroll_offset.column = 0;
            } else if self.editor.position.column
                >= self.editor.scroll_offset.column + terminal_width
            {
                self.editor.scroll_offset.column = self.editor.position.column - terminal_width + 1;
            }
        }

        Ok(())
    }

    async fn move_left(&mut self, offset: usize) -> Result<()> {
        let terminal_width = self.terminal.size()?.width as usize;

        if self.editor.position.column > 0 {
            self.editor.position.column = self.editor.position.column.saturating_sub(offset);

            self.editor.position.history.column = self.editor.position.column;

            if self.editor.position.column < self.editor.scroll_offset.column {
                self.editor.scroll_offset.column = self
                    .editor
                    .position
                    .column
                    .saturating_sub(terminal_width)
                    .max(0);
            }
        } else {
            if self.editor.position.row == self.editor.scroll_offset.row {
                if self.editor.scroll_offset.row > 0 {
                    self.editor.scroll_offset.row -= 1;
                }
            }

            if self.editor.position.row > 0 {
                self.editor.position.row -= 1;

                self.editor.position.column =
                    self.editor.document.row_length(self.editor.position.row);

                self.editor.position.history.column = self.editor.position.column;

                if self.editor.position.column >= self.editor.scroll_offset.column + terminal_width
                {
                    self.editor.scroll_offset.column =
                        self.editor.position.column + terminal_width - offset;
                }
            }
        }

        Ok(())
    }

    async fn move_right(&mut self, offset: usize) -> Result<()> {
        let current_row_length = self.editor.document.row_length(self.editor.position.row);

        let terminal_width = self.terminal.size()?.width as usize;

        let terminal_height = self.terminal.size()?.height as usize;

        if self.editor.position.column < current_row_length {
            self.editor.position.column += offset;

            self.editor.position.history.column = self.editor.position.column;

            if self.editor.position.column >= self.editor.scroll_offset.column + terminal_width {
                self.editor.scroll_offset.column += offset;
            }
        } else {
            if self.editor.position.row >= self.editor.scroll_offset.row + terminal_height - offset
            {
                if self.editor.scroll_offset.row + terminal_height < self.editor.document.rows.len()
                {
                    self.editor.scroll_offset.row += 1;
                }
            }

            if self.editor.position.row.saturating_add(1)
                <= self.editor.document.rows.len().saturating_sub(1)
            {
                self.editor.position.row += 1;

                self.editor.position.column = 0;

                self.editor.position.history.column = 0;

                self.editor.scroll_offset.column = 0;
            }
        }

        Ok(())
    }

    async fn draw(&mut self) -> Result<()> {
        self.draw_document().await
    }

    async fn draw_document(&mut self) -> Result<()> {
        let start = self.editor.scroll_offset.column;
        let end = self
            .editor
            .scroll_offset
            .column
            .saturating_add(self.terminal.size()?.width as usize);

        let rows: Vec<Line> = self
            .editor
            .document
            .rows
            .iter()
            .enumerate()
            .filter(|(i, _)| i >= &self.editor.scroll_offset.row)
            .map(|(_, r)| Line::from(Span::from(r.render(start, end))))
            .collect();

        let paragraph = Paragraph::new(rows);
        let block = Block::default();

        let width = self.terminal.size()?.width;
        let height = self.terminal.size()?.height;

        self.terminal.draw(|f| {
            f.render_widget(
                paragraph.block(block),
                Rect {
                    width,
                    height,

                    x: 0,
                    y: 0,
                },
            );

            f.set_cursor(
                (self.editor.position.column - self.editor.scroll_offset.column) as u16,
                (self.editor.position.row - self.editor.scroll_offset.row) as u16,
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
