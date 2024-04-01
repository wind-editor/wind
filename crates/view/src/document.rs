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
    #[inline]
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = end.min(self.content.len());
        let start = start.min(end);

        self.content.get(start..end).unwrap_or_default().to_string()
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
        }

        Ok(Document {
            path: file_path,
            rows,
        })
    }

    #[inline]
    pub fn row_len(&self, index: usize) -> usize {
        match self.rows.get(index) {
            Some(row) => row.len(),
            None => 0,
        }
    }
}
