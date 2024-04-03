#[derive(Default, Clone, Copy)]
pub struct Position {
    pub row: usize,
    pub column: usize,
    pub history: PositionHistory,
}

#[derive(Default, Clone, Copy)]
pub struct PositionHistory {
    pub row: usize,
    pub column: usize,
}
