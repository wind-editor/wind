use crate::cli::CLI;

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

#[derive(Clone, Copy)]
pub enum AppMessage {
    Exit,
}

pub struct App {
    terminal: Terminal,
    editor: Editor,
    message: Option<AppMessage>,
    layout: Layout,
}

impl App {
    pub fn new(cli: CLI) -> Result<App> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(App {
            terminal,
            editor: Editor::new(cli.file_path)?,
            message: None,
            layout: Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(97), Constraint::Percentage(3)]),
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
            self.draw()?;

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
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char(ch) => match ch {
                    'q' => {
                        self.message = Some(AppMessage::Exit);

                        Ok(())
                    }

                    _ => Ok(()),
                },

                _ => Ok(()),
            }
        } else {
            match key_event.code {
                KeyCode::Up => self.move_up(1),

                KeyCode::Down => self.move_down(1),

                KeyCode::Left => self.move_left(1),

                KeyCode::Right => self.move_right(1),

                KeyCode::Home => self.move_left(self.editor.position.column),

                KeyCode::End => {
                    let current_row_length =
                        self.editor.document.row_length(self.editor.position.row);

                    self.move_right(
                        current_row_length
                            .saturating_sub(self.editor.position.column)
                            .saturating_sub(1),
                    )
                }

                _ => Ok(()),
            }
        }
    }

    fn move_up(&mut self, offset: usize) -> Result<()> {
        let text_area = self.layout.split(self.terminal.size()?)[0];

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
                >= self.editor.scroll_offset.column + text_area.width as usize
            {
                self.editor.scroll_offset.column =
                    self.editor.position.column - text_area.width as usize + 1;
            }
        }

        Ok(())
    }

    fn move_down(&mut self, offset: usize) -> Result<()> {
        let text_area = self.layout.split(self.terminal.size()?)[0];

        if self.editor.position.row.saturating_add(offset) < self.editor.document.rows.len() {
            if self.editor.position.row
                >= self.editor.scroll_offset.row + text_area.height as usize - offset
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
                >= self.editor.scroll_offset.column + text_area.width as usize
            {
                self.editor.scroll_offset.column =
                    self.editor.position.column - text_area.width as usize + 1;
            }
        }

        Ok(())
    }

    fn move_left(&mut self, offset: usize) -> Result<()> {
        let text_area = self.layout.split(self.terminal.size()?)[0];

        if self.editor.position.column > 0 {
            self.editor.position.column = self.editor.position.column.saturating_sub(offset);

            self.editor.position.history.column = self.editor.position.column;

            if self.editor.position.column < self.editor.scroll_offset.column {
                self.editor.scroll_offset.column = self
                    .editor
                    .position
                    .column
                    .saturating_sub(text_area.width as usize);
            }
        } else if offset != 0 {
            if self.editor.position.row == self.editor.scroll_offset.row {
                if self.editor.scroll_offset.row > 0 {
                    self.editor.scroll_offset.row -= 1;
                }
            }

            if self.editor.position.row > 0 {
                self.editor.position.row -= 1;

                self.editor.position.column = self
                    .editor
                    .document
                    .row_length(self.editor.position.row)
                    .saturating_sub(1);

                self.editor.position.history.column = self.editor.position.column;

                if self.editor.position.column
                    >= self.editor.scroll_offset.column + text_area.width as usize
                {
                    self.editor.scroll_offset.column =
                        self.editor.position.column + text_area.width as usize - offset;
                }
            }
        }

        Ok(())
    }

    fn move_right(&mut self, offset: usize) -> Result<()> {
        let current_row_length = self.editor.document.row_length(self.editor.position.row);

        let text_area = self.layout.split(self.terminal.size()?)[0];

        if self.editor.position.column < current_row_length.saturating_sub(1) {
            self.editor.position.column += offset;

            self.editor.position.history.column = self.editor.position.column;

            if self.editor.position.column
                >= self.editor.scroll_offset.column + text_area.width as usize
            {
                self.editor.scroll_offset.column += offset;
            }
        } else if offset != 0 {
            if self.editor.position.row
                >= self
                    .editor
                    .scroll_offset
                    .row
                    .saturating_add(text_area.height as usize)
                    .saturating_sub(offset)
            {
                if self
                    .editor
                    .scroll_offset
                    .row
                    .saturating_add(text_area.height as usize)
                    < self.editor.document.rows.len()
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

    fn draw(&mut self) -> Result<()> {
        let areas = self.layout.split(self.terminal.size()?);

        let line_start = self.editor.scroll_offset.column;

        let line_end = self
            .editor
            .scroll_offset
            .column
            .saturating_add(areas[0].width as usize);

        let lines: Vec<Line> = self
            .editor
            .document
            .rows
            .iter()
            .enumerate()
            .filter(|(i, _)| i >= &self.editor.scroll_offset.row)
            .map(|(_, r)| Line::from(Span::from(r.render(line_start, line_end))))
            .collect();

        let lines_paragraph = Paragraph::new(lines);

        let text_area_block = Block::default().bg(Color::from_u32(0x00151515));

        let file_name = match self.editor.document.path.as_ref() {
            Some(file_path) => file_path
                .file_name()
                .unwrap_or(file_path.as_os_str())
                .to_string_lossy()
                .to_string(),

            None => "[No Name]".to_owned(),
        };

        let file_name_paragraph = Paragraph::new(file_name);

        let status_bar_area = Block::default().bg(Color::from_u32(0x00181818));

        self.terminal.draw(|f| {
            f.render_widget(
                lines_paragraph.block(text_area_block),
                Rect {
                    x: areas[0].x,
                    y: areas[0].y,
                    width: areas[0].width,
                    height: areas[0].height,
                },
            );

            f.set_cursor(
                (self.editor.position.column - self.editor.scroll_offset.column) as u16,
                (self.editor.position.row - self.editor.scroll_offset.row) as u16,
            );

            f.render_widget(
                file_name_paragraph
                    .block(status_bar_area)
                    .alignment(Alignment::Center),
                Rect {
                    x: areas[1].x,
                    y: areas[1].y,
                    width: areas[1].width,
                    height: areas[1].height,
                },
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
