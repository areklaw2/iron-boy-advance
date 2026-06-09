const PERIOD_TICK_CYCLES: usize = 16;

#[derive(Debug)]
pub struct Period {
    accumulator: usize,
}

impl Period {
    pub fn new() -> Self {
        Period { accumulator: 0 }
    }

    pub fn step(&mut self, cycles: usize, frequency: usize) -> usize {
        self.accumulator += cycles;
        let step = PERIOD_TICK_CYCLES * (2048 - frequency);
        let steps = self.accumulator / step;
        self.accumulator %= step;
        steps
    }

    pub fn trigger(&mut self) {
        self.accumulator = 0;
    }
}
