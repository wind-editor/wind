#[derive(Default)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

impl Position {
    pub fn new(row: usize, column: usize) -> Position {
        Position { row, column }
    }
}
