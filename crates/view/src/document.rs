use anyhow::Result;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Default)]
pub struct Row {
    pub content: String,
}

impl From<String> for Row {
    fn from(value: String) -> Self {
        Self { content: value }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = std::cmp::min(end, self.content.len());
        let start = std::cmp::min(start, end);

        self.content.get(start..end).unwrap_or_default().to_string()
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }
}

#[derive(Default)]
pub struct Document {
    pub name: Option<PathBuf>,
    pub rows: Vec<Row>,
}

impl Document {
    pub fn open(file_name: Option<PathBuf>) -> Result<Document> {
        let mut rows = Vec::new();

        if file_name.is_none() || file_name.as_ref().is_some_and(|f| !f.exists()) {
            return Ok(Document {
                name: file_name,
                rows,
            });
        }

        let file = File::open(file_name.as_ref().unwrap())?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            rows.push(Row::from(line?));
        }

        Ok(Document {
            name: file_name,
            rows,
        })
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn row_length(&self, index: usize) -> usize {
        match self.row(index) {
            Some(row) => row.len(),
            None => 0,
        }
    }
}
