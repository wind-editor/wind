#[derive(Clone, Copy)]
pub struct Boundaries {
    pub width: u16,
    pub height: u16,
}

impl Boundaries {
    pub fn new(width: u16, height: u16) -> Boundaries {
        Boundaries { width, height }
    }
}
