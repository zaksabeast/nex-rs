#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    num: usize,
}

impl Counter {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn increment(&mut self) -> usize {
        self.num += 1;
        self.num
    }

    pub fn value(&mut self) -> usize {
        self.num
    }
}
