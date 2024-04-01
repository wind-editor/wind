use std::env;

use crate::editor::Editor;

use anyhow::Result;

use ratatui::backend::Backend as TerminalBackend;
use ratatui::layout::*;
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::*;
use ratatui::Terminal;

pub struct Palette {
    pub text_area_fg: Color,
    pub text_area_bg: Color,
    pub line_numbers_fg: Color,
    pub status_bar_fg: Color,
    pub status_bar_bg: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            text_area_fg: Color::from_u32(0x008F93A2),
            text_area_bg: Color::from_u32(0x000F111A),
            line_numbers_fg: Color::from_u32(0x008F93A2),
            status_bar_fg: Color::from_u32(0x008F93A2),
            status_bar_bg: Color::from_u32(0x00090B10),
        }
    }
}

pub struct Painter {
    areas: [Rect; 3],
    palette: Palette,
}

impl Painter {
    pub fn new(boundaries: Rect) -> Painter {
        let mut painter = Painter {
            areas: [Rect::default(); 3],
            palette: Palette::default(),
        };

        painter.recompute_areas(boundaries);

        painter
    }

    pub fn recompute_areas(&mut self, boundaries: Rect) {
        let main_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(95), Constraint::Percentage(5)],
        );

        let main_areas = main_layout.split(boundaries);

        let text_area = Layout::new(
            Direction::Horizontal,
            [Constraint::Max(5), Constraint::Min(1)],
        )
        .split(main_areas[0]);

        self.areas = [text_area[0], text_area[1], main_areas[1]];
    }

    #[inline]
    pub fn get_line_numbers_area(&self) -> Rect {
        self.areas[0]
    }

    #[inline]
    pub fn get_text_area(&self) -> Rect {
        self.areas[1]
    }

    #[inline]
    pub fn get_status_bar_area(&self) -> Rect {
        self.areas[2]
    }

    pub fn paint<T: TerminalBackend>(
        &self,
        terminal: &mut Terminal<T>,
        editor: &Editor,
    ) -> Result<()> {
        let text_area = self.get_text_area();

        let text_block = Block::default()
            .fg(self.palette.text_area_fg)
            .bg(self.palette.text_area_bg);

        let line_start = editor.scroll_offset.column;

        let line_end = editor
            .scroll_offset
            .column
            .saturating_add(text_area.width as usize);

        let lines: Vec<Line> = editor
            .document
            .rows
            .iter()
            .skip(editor.scroll_offset.row)
            .enumerate()
            .map_while(|(i, r)| {
                if i > text_area.height as usize {
                    None
                } else {
                    Some(Line::from(Span::from(r.render(line_start, line_end))))
                }
            })
            .collect();

        let line_numbers_area = self.get_line_numbers_area();

        let line_numbers_block = Block::default()
            .fg(self.palette.line_numbers_fg)
            .bg(self.palette.text_area_bg);

        let lines_paragraph = Paragraph::new(lines.clone());

        let mut line_numbers = Vec::new();

        if env::var("WIND_RELATIVE_LINE_NUMBERS").is_ok() {
            let mut i = lines.iter().enumerate().position(|(i, _)| i == editor.position.row - editor.scroll_offset.row).unwrap();

            let mut increment = false;

            while i <= lines.len() {
                if i == 0 {
                    line_numbers.push(editor.position.row + 1);

                    increment = true;
                } else {
                    line_numbers.push(i);
                }

                if increment {
                    i += 1;
                } else {
                    i -= 1;
                }
            }
        } else {
            for i in 0..lines.len() {
                line_numbers.push(i + editor.scroll_offset.row + 1);
            }
        }

        let line_numbers_paragraph = Paragraph::new(
            line_numbers
                .iter()
                .map(|n| Line::from(Span::from(format!("{}\n", n))))
                .collect::<Vec<Line>>(),
        );

        let status_bar_area = self.get_status_bar_area();

        let status_bar_block = Block::default()
            .fg(self.palette.status_bar_fg)
            .bg(self.palette.status_bar_bg);

        let file_name = match editor.document.path.as_ref() {
            Some(file_path) => file_path
                .file_name()
                .unwrap_or(file_path.as_os_str())
                .to_string_lossy()
                .to_string(),

            None => "[No Name]".to_owned(),
        };

        let file_name_paragraph = Paragraph::new(file_name);

        let position = format!("{}:{}", editor.position.row + 1, editor.position.column + 1);

        let position_paragraph = Paragraph::new(position);

        terminal.draw(|f| {
            f.set_cursor(
                (editor
                    .position
                    .column
                    .saturating_sub(editor.scroll_offset.column) as u16)
                    .saturating_add(text_area.x),
                editor.position.row.saturating_sub(editor.scroll_offset.row) as u16,
            );

            f.render_widget(lines_paragraph.block(text_block), text_area);

            f.render_widget(
                line_numbers_paragraph.block(line_numbers_block).centered(),
                line_numbers_area,
            );

            f.render_widget(
                file_name_paragraph
                    .block(status_bar_block.clone())
                    .centered(),
                status_bar_area,
            );

            f.render_widget(
                position_paragraph.block(status_bar_block).right_aligned(),
                status_bar_area,
            );
        })?;

        Ok(())
    }
}
