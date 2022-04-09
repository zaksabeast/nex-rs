#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    counter: u32,
}

impl Counter {
    pub fn new(initial: u32) -> Self {
        Self { counter: initial }
    }

    pub fn increment(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }

    pub fn value(&mut self) -> u32 {
        self.counter
    }
}
