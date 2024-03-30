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
        let text_area = self.layout.split(self.terminal.size()?)[0];

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

        let position = format!(
            "{}:{}",
            self.editor.position.row + 1,
            self.editor.position.column + 1
        );

        let position_paragraph = Paragraph::new(position);

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
                (self
                    .editor
                    .position
                    .column
                    .saturating_sub(self.editor.scroll_offset.column)) as u16,
                (self
                    .editor
                    .position
                    .row
                    .saturating_sub(self.editor.scroll_offset.row)) as u16,
            );

            f.render_widget(
                file_name_paragraph
                    .block(status_bar_area.clone())
                    .alignment(Alignment::Center),
                Rect {
                    x: areas[1].x,
                    y: areas[1].y,
                    width: areas[1].width,
                    height: areas[1].height,
                },
            );

            f.render_widget(
                position_paragraph
                    .block(status_bar_area)
                    .alignment(Alignment::Right),
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
