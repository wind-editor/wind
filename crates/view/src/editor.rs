use crate::document::*;
use crate::position::*;

use std::path::PathBuf;

#[derive(Default)]
pub struct Editor {
    pub document: Document,
    pub position: Position,
    pub position_history: Position,
    pub scroll_offset: Position,
}

impl Editor {
    pub fn new(file: Option<PathBuf>) -> Editor {
        Editor {
            document: Document::open(file).unwrap_or_default(),
            position: Position::default(),
            position_history: Position::default(),
            scroll_offset: Position::default(),
        }
    }
}
