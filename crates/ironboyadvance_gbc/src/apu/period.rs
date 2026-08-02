use getset::CopyGetters;

pub const WAVE_TRIGGER_DELAY: u16 = 6;

#[derive(Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Period {
    timer: u16,
    reloaded: bool,
}

impl Period {
    pub fn new() -> Self {
        Period {
            timer: 0,
            reloaded: false,
        }
    }

    pub fn cycle(&mut self, cycles: usize, reload: u16) -> usize {
        self.reloaded = false;
        if reload == 0 {
            return 0;
        }

        let mut steps = 0;
        for _ in 0..cycles {
            self.timer = self.timer.saturating_sub(1);
            if self.timer == 0 {
                self.timer = reload;
                self.reloaded = true;
                steps += 1;
            }
        }

        if self.timer + 1 < reload {
            self.reloaded = false;
        }
        steps
    }

    pub fn trigger(&mut self, reload: u16) {
        self.timer = reload;
    }

    pub fn delay_wave_trigger(&mut self) {
        self.timer += WAVE_TRIGGER_DELAY;
    }
}
