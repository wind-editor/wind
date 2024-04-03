use crate::position::Position;

use anyhow::Result;

use unicode_segmentation::UnicodeSegmentation;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Default)]
pub struct Row {
    pub content: String,
    len: usize,
}

impl From<String> for Row {
    fn from(value: String) -> Self {
        let mut row = Self {
            content: value,
            len: 0,
        };

        row.update_len();

        row
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = end.min(self.content.len());
        let start = start.min(end);

        self.content
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect()
    }

    pub fn split(&mut self, at: usize) -> Row {
        let start = self.content.graphemes(true).take(at).collect();

        let mid: String = self.content.graphemes(true).skip(at).collect();

        self.content = start;
        self.update_len();

        Row::from(mid)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn update_len(&mut self) {
        self.len = self.content.graphemes(true).count();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[derive(Default)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub rows: Vec<Row>,
    pub modified: bool,
}

impl Document {
    pub fn open(file_path: Option<PathBuf>) -> Result<Document> {
        let mut rows = Vec::new();

        if file_path.as_ref().is_some_and(|f| f.exists()) {
            let file = File::open(file_path.as_ref().unwrap())?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                rows.push(Row::from(line?));
            }
        } else {
            rows.push(Row::default());
        }

        Ok(Document {
            path: file_path,
            rows,
            modified: false,
        })
    }

    pub fn insert_new_line(&mut self, at: Position) {
        self.modified = true;

        let row = self.rows.get_mut(at.row).unwrap();

        let new_row = row.split(at.column);

        self.rows.insert(at.row.saturating_add(1), new_row);
    }

    pub fn insert(&mut self, at: Position, ch: char) {
        self.modified = true;

        if ch == '\n' {
            self.insert_new_line(at);

            return;
        }

        let row = self.rows.get_mut(at.row).unwrap();

        row.content.insert(at.column, ch);

        row.update_len();
    }

    pub fn delete(&mut self, at: Position) {
        self.modified = true;

        if at.column == self.row_len(at.row) && at.row < self.rows.len() - 1 {
            let next_row = self.rows.remove(at.row.saturating_add(1));

            let row = self.rows.get_mut(at.row).unwrap();


            let result = row.content.graphemes(true).chain(next_row.content.graphemes(true)).collect();

            row.content = result;

            row.update_len();
        } else {
            let row = self.rows.get_mut(at.row).unwrap();

            let mut result: String = row.content.graphemes(true).collect();

            result.remove(at.column);

            row.content = result;
            
            row.update_len();
        }
    }

    #[inline]
    pub fn row_len(&self, index: usize) -> usize {
        match self.rows.get(index) {
            Some(row) => row.len(),
            None => 0,
        }
    }
}
