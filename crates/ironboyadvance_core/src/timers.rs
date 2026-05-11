use bitfields::bitfield;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;
use tracing::debug;

use crate::scheduler::event::{FutureEvent, TimersEvent};

const PRESCALER_CYCLES: [u16; 4] = [1, 64, 256, 1024];

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TimerControl {
    #[bits(2)]
    prescaler: u8,
    cascade: bool,
    #[bits(3)]
    _not_used_3_5: u8,
    irq_enable: bool,
    enabled: bool,
    _not_used_8_15: u8,
}

#[derive(Copy, Clone, Default)]
pub struct Timer {
    counter: u16,
    reload: u16,
    control: TimerControl,
    start_time: u16,
}

pub struct Timers {
    timers: [Timer; 4],
}

impl Timers {
    pub fn new() -> Self {
        Self {
            timers: [Timer::default(); 4],
        }
    }

    pub fn handle_event(&mut self, event: TimersEvent) -> Vec<FutureEvent> {
        match event {
            TimersEvent::ControlWrite(_) => todo!(),
            TimersEvent::ReloadWrite(_) => todo!(),
        }
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
