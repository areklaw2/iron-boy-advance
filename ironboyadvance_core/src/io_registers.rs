use std::{cell::RefCell, rc::Rc};

use bitfields::bitfield;
use ironboyadvance_arm7tdmi::memory::SystemMemoryAccess;

use crate::{scheduler::Scheduler, system_bus::ClockCycleLuts};

const WAITCNT: u32 = 0x04000204;

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct WaitStateControl {
    #[bits(2)]
    sram_wait_control: u8,
    #[bits(2)]
    ws0_first_access: u8,
    #[bits(1)]
    ws0_second_access: u8,
    #[bits(2)]
    ws1_first_access: u8,
    #[bits(1)]
    ws1_second_access: u8,
    #[bits(2)]
    ws2_first_access: u8,
    #[bits(1)]
    ws2_second_access: u8,
    #[bits(2)]
    phi_terminal_output: u8,
    _reserved: bool,
    game_pak_prefetch_buffer_enable: bool,
    game_pak_type_flag: bool,
}

pub struct IoRegisters {
    scheduler: Rc<RefCell<Scheduler>>,
    cycle_luts: Rc<RefCell<ClockCycleLuts>>,
    waitcnt: WaitStateControl,
    data: Vec<u8>,
}

impl IoRegisters {
    pub fn new(scheduler: Rc<RefCell<Scheduler>>, cycle_luts: Rc<RefCell<ClockCycleLuts>>) -> Self {
        IoRegisters {
            scheduler,
            cycle_luts,
            waitcnt: WaitStateControl::from_bits(0),
            data: vec![0; 0x400],
        }
    }
}

//TODO: Work on WaitControl
impl SystemMemoryAccess for IoRegisters {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            _ => 0, //TODO: add tracing for this
        }
    }

    fn read_16(&self, address: u32) -> u16 {
        match address {
            WAITCNT => self.waitcnt.into_bits(),
            _ => 0, //TODO: add tracing for this
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            _ => {} //TODO: add tracing for this
        }
    }

    fn write_16(&mut self, address: u32, value: u16) {
        match address {
            WAITCNT => {
                self.waitcnt = WaitStateControl::from_bits(value);
                self.cycle_luts.borrow_mut().update_wait_states(&self.waitcnt);
            }
            _ => {} //TODO: add tracing for this
        }
    }
}
