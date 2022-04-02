#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    counter: u16,
}

impl Counter {
    pub fn new(initial: u16) -> Self {
        Self { counter: initial }
    }

    pub fn increment(&mut self) -> u16 {
        self.counter += 1;
        self.counter
    }

    pub fn value(&mut self) -> u16 {
        self.counter
    }
}
