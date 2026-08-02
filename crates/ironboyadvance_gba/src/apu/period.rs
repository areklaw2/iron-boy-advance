#[derive(Debug)]
pub struct Period {
    accumulator: usize,
}

impl Period {
    pub fn new() -> Self {
        Period { accumulator: 0 }
    }

    pub fn cycle(&mut self, cycles: usize, reload: usize) -> usize {
        if reload == 0 {
            return 0;
        }

        self.accumulator += cycles;
        let steps = self.accumulator / reload;
        self.accumulator %= reload;
        steps
    }

    pub fn trigger(&mut self) {
        self.accumulator = 0;
    }
}
