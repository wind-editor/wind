#[derive(Default)]
pub struct Position {
    pub row: usize,
    pub column: usize,
    pub history: PositionHistory,
}

#[derive(Default)]
pub struct PositionHistory {
    pub row: usize,
    pub column: usize,
}
