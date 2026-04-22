use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use tracing::debug;

pub struct Timers {}

impl Timers {
    pub fn new() -> Self {
        Timers {}
    }
}

// 4000100h - TM0CNT_L - Timer 0 Counter/Reload (R/W)
// 4000104h - TM1CNT_L - Timer 1 Counter/Reload (R/W)
// 4000108h - TM2CNT_L - Timer 2 Counter/Reload (R/W)
// 400010Ch - TM3CNT_L - Timer 3 Counter/Reload (R/W)

// 4000102h - TM0CNT_H - Timer 0 Control (R/W)
// 4000106h - TM1CNT_H - Timer 1 Control (R/W)
// 400010Ah - TM2CNT_H - Timer 2 Control (R/W)
// 400010Eh - TM3CNT_H - Timer 3 Control (R/W)

impl SystemMemoryAccess for Timers {
    fn read_8(&self, address: u32) -> u8 {
        debug!("{}", address);
        0
    }

    fn write_8(&mut self, address: u32, value: u8) {
        debug!("{}, {}", address, value)
    }
}
