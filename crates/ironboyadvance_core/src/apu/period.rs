#[derive(Debug)]
pub struct Period {
    accumulator: usize,
}

impl Period {
    pub fn new() -> Self {
        Period { accumulator: 0 }
    }

    // Accumulate `cycles` and return how many whole `period_cycles` long steps elapsed, keeping the remainder for the next call.
    // The caller owns the channel-specific formula that produces `period_cycles`.
    pub fn step(&mut self, cycles: usize, period_cycles: usize) -> usize {
        self.accumulator += cycles;
        let steps = self.accumulator / period_cycles;
        self.accumulator %= period_cycles;
        steps
    }

    pub fn trigger(&mut self) {
        self.accumulator = 0;
    }
}
