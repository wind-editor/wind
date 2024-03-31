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
    pub status_bar_fg: Color,
    pub status_bar_bg: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            text_area_fg: Color::from_u32(0x008F93A2),
            text_area_bg: Color::from_u32(0x000F111A),
            status_bar_fg: Color::from_u32(0x008F93A2),
            status_bar_bg: Color::from_u32(0x00090B10),
        }
    }
}

pub struct Painter {
    layout: Layout,
    palette: Palette,
}

impl Default for Painter {
    fn default() -> Self {
        Self {
            layout: Layout::new(
                Direction::Vertical,
                [Constraint::Percentage(95), Constraint::Percentage(5)],
            ),

            palette: Palette::default(),
        }
    }
}

impl Painter {
    #[inline]
    pub fn get_text_area(&self, boundaries: Rect) -> Rect {
        self.layout.split(boundaries)[0]
    }

    #[inline]
    pub fn get_status_bar_area(&self, boundaries: Rect) -> Rect {
        self.layout.split(boundaries)[1]
    }

    pub fn paint<T: TerminalBackend>(
        &self,
        terminal: &mut Terminal<T>,
        editor: &Editor,
    ) -> Result<()> {
        let text_area = self.get_text_area(terminal.size()?);

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
            .map(|r| Line::from(Span::from(r.render(line_start, line_end))))
            .collect();

        let lines_paragraph = Paragraph::new(lines);

        let status_bar_area = self.get_status_bar_area(terminal.size()?);

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
            f.render_widget(lines_paragraph.block(text_block), text_area);

            f.set_cursor(
                editor
                    .position
                    .column
                    .saturating_sub(editor.scroll_offset.column) as u16,
                editor.position.row.saturating_sub(editor.scroll_offset.row) as u16,
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
