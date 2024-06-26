use crate::boundaries::Boundaries;
use crate::document::*;
use crate::position::*;

use anyhow::Result;

use std::path::PathBuf;

#[derive(PartialEq)]
pub enum EditorStatus {
    Message(String),
    Exit,
    None,
}

impl Default for EditorStatus {
    fn default() -> Self {
        Self::None
    }
}

impl ToString for EditorStatus {
    fn to_string(&self) -> String {
        match self {
            EditorStatus::Message(msg) => msg.to_owned(),
            EditorStatus::Exit => "".to_owned(),
            EditorStatus::None => "".to_owned(),
        }
    }
}

pub enum EditorMode {
    Normal,
    Insert,
}

impl Default for EditorMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl ToString for EditorMode {
    fn to_string(&self) -> String {
        match self {
            EditorMode::Normal => "normal".to_owned(),
            EditorMode::Insert => "insert".to_owned(),
        }
    }
}

#[derive(Default)]
pub struct Editor {
    pub document: Document,
    pub position: Position,
    pub scroll_offset: Position,
    pub status: EditorStatus,
    pub mode: EditorMode,
}

impl Editor {
    pub fn new(file_path: Option<PathBuf>) -> Result<Editor> {
        Ok(Editor {
            document: Document::open(file_path)?,
            ..Default::default()
        })
    }

    pub fn move_up(&mut self, boundaries: Boundaries, offset: usize) -> Result<()> {
        if self.position.row > 0 {
            if self.position.row <= self.scroll_offset.row {
                self.scroll_offset.row = self.scroll_offset.row.saturating_sub(offset);
            }

            self.position.row -= offset;

            self.position.column = self
                .position
                .history
                .column
                .min(self.document.row_len(self.position.row));

            if self.position.column < self.scroll_offset.column {
                self.scroll_offset.column = 0;
            } else if self.position.column >= self.scroll_offset.column + boundaries.width as usize
            {
                self.scroll_offset.column = self.position.column - boundaries.width as usize + 1;
            }
        }

        Ok(())
    }

    pub fn move_down(&mut self, boundaries: Boundaries, offset: usize) -> Result<()> {
        if self.position.row.saturating_add(offset) < self.document.rows.len() {
            if self.position.row >= self.scroll_offset.row + boundaries.height as usize - offset {
                self.scroll_offset.row += offset;
            }

            self.position.row += offset;

            self.position.column = self
                .position
                .history
                .column
                .min(self.document.row_len(self.position.row));

            if self.position.column < self.scroll_offset.column {
                self.scroll_offset.column = 0;
            } else if self.position.column >= self.scroll_offset.column + boundaries.width as usize
            {
                self.scroll_offset.column = self.position.column - boundaries.width as usize + 1;
            }
        }

        Ok(())
    }

    pub fn move_left(&mut self, boundaries: Boundaries, offset: usize) -> Result<()> {
        if self.position.column > 0 {
            self.position.column = self.position.column.saturating_sub(offset);

            self.position.history.column = self.position.column;

            if self.position.column < self.scroll_offset.column {
                self.scroll_offset.column = self
                    .position
                    .column
                    .saturating_sub(boundaries.width as usize);
            }
        } else if offset != 0 {
            if self.position.row == self.scroll_offset.row && self.scroll_offset.row > 0 {
                self.scroll_offset.row -= 1;
            }

            if self.position.row > 0 {
                self.position.row -= 1;

                self.position.column = self.document.row_len(self.position.row);

                self.position.history.column = self.position.column;

                if self.position.column >= self.scroll_offset.column + boundaries.width as usize {
                    self.scroll_offset.column =
                        self.position.column + boundaries.width as usize - offset;
                }
            }
        }

        Ok(())
    }

    pub fn move_right(&mut self, boundaries: Boundaries, offset: usize) -> Result<()> {
        let current_row_length = self.document.row_len(self.position.row);

        if self.position.column < current_row_length {
            self.position.column += offset;

            self.position.history.column = self.position.column;

            if self.position.column >= self.scroll_offset.column + boundaries.width as usize {
                self.scroll_offset.column += offset;
            }
        } else if offset != 0 {
            if self.position.row
                >= self
                    .scroll_offset
                    .row
                    .saturating_add(boundaries.height as usize)
                    .saturating_sub(offset)
                && self
                    .scroll_offset
                    .row
                    .saturating_add(boundaries.height as usize)
                    < self.document.rows.len()
            {
                self.scroll_offset.row += 1;
            }

            if self.position.row.saturating_add(1) <= self.document.rows.len().saturating_sub(1) {
                self.position.row += 1;

                self.position.column = 0;

                self.position.history.column = 0;

                self.scroll_offset.column = 0;
            }
        }

        Ok(())
    }

    pub fn save(&mut self) {
        if self.document.path.is_none() {
            self.document.path = Some(PathBuf::from("temp"))
        }

        match self.document.save() {
            Ok(n) => {
                self.status = EditorStatus::Message(format!(
                    "'{}' saved, {}L {}B",
                    self.document.path.as_ref().unwrap().display(),
                    self.document.rows.len(),
                    n
                ));
            }

            Err(err) => {
                self.status =
                    EditorStatus::Message(format!("Could not save the document: {}", err));
            }
        }
    }
}
