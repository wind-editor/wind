use crate::document::*;
use crate::position::*;

use anyhow::Result;

use std::path::PathBuf;

#[derive(Default)]
pub struct Editor {
    pub document: Document,
    pub position: Position,
    pub scroll_offset: Position,
}

impl Editor {
    pub fn new(file_path: Option<PathBuf>) -> Result<Editor> {
        Ok(Editor {
            document: Document::open(file_path)?,
            position: Position::default(),
            scroll_offset: Position::default(),
        })
    }
}
