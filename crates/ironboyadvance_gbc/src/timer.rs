use std::{cell::RefCell, rc::Rc};

use bitfields::bitfield;
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use ironboyadvance_sm83::GbSpeed;

use crate::events::{GbcEvent, InterruptEvent, TimerEvent};

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
struct TimerControl {
    #[bits(2)]
    clock_select: u8,
    enabled: bool,
    #[bits(5)]
    _not_used_3_7: u8,
}

impl TimerControl {
    fn t_cycles(&self) -> usize {
        match self.clock_select() {
            0b01 => 16,
            0b10 => 64,
            0b11 => 256,
            _ => 1024,
        }
    }
}

pub struct Timer {
    divider: usize,
    counter: u8,
    counter_cycles: usize,
    modulo: u8,
    control: TimerControl,
    speed: GbSpeed,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
}

impl Timer {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        let timestamp = scheduler.borrow().timestamp();

        Timer {
            divider: timestamp,
            counter: 0,
            counter_cycles: 0,
            modulo: 0,
            control: TimerControl::from_bits(0),
            speed: GbSpeed::Normal,
            scheduler,
        }
    }

    fn speed_shift(&self) -> usize {
        match self.speed {
            GbSpeed::Normal => 0,
            GbSpeed::Double => 1,
        }
    }

    fn period(&self) -> usize {
        self.control.t_cycles() >> self.speed_shift()
    }

    fn elapsed_cycles(&self) -> usize {
        (self.scheduler.borrow().timestamp() - self.divider) / self.period()
    }

    fn schedule_overflow(&mut self) {
        self.scheduler
            .borrow_mut()
            .cancel_events(GbcEvent::Timer(TimerEvent::Overflow));

        if !self.control.enabled() {
            return;
        }

        let cycles = self.counter_cycles + 0x100 - self.counter as usize;
        self.scheduler
            .borrow_mut()
            .schedule_at_timestamp(GbcEvent::Timer(TimerEvent::Overflow), self.divider + cycles * self.period());
    }

    fn reset(&mut self) {
        self.counter_cycles = self.elapsed_cycles();
        self.schedule_overflow();
    }

    fn overflow(&mut self) {
        self.counter_cycles += 0x100 - self.counter as usize;
        self.counter = self.modulo;
        self.scheduler
            .borrow_mut()
            .schedule((GbcEvent::Interrupt(InterruptEvent::Timer), 0));
        self.schedule_overflow();
    }

    pub fn handle_event(&mut self, timer_event: TimerEvent) {
        match timer_event {
            TimerEvent::Overflow => self.overflow(),
        }
    }

    fn read_divider(&self) -> u8 {
        ((self.scheduler.borrow().timestamp() - self.divider) >> (8 - self.speed_shift())) as u8
    }

    fn reset_divider(&mut self) {
        self.counter = self.read_counter();
        self.divider = self.scheduler.borrow().timestamp();
        self.reset();
    }

    fn read_counter(&self) -> u8 {
        match self.control.enabled() {
            true => self.counter.wrapping_add((self.elapsed_cycles() - self.counter_cycles) as u8),
            false => self.counter,
        }
    }

    fn write_counter(&mut self, value: u8) {
        self.counter = value;
        self.reset();
    }

    fn write_control(&mut self, value: u8) {
        self.counter = self.read_counter();
        self.control = TimerControl::from_bits(value);
        self.reset();
    }

    pub fn set_speed(&mut self, speed: GbSpeed) {
        self.counter = self.read_counter();
        self.speed = speed;
        self.divider = self.scheduler.borrow().timestamp();
        self.reset();
    }
}

impl SystemMemoryAccess for Timer {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.read_divider(),
            0xFF05 => self.read_counter(),
            0xFF06 => self.modulo,
            0xFF07 => self.control.into_bits() | 0xF8,
            _ => panic!("Invalid byte read for Timer: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.reset_divider(),
            0xFF05 => self.write_counter(value),
            0xFF06 => self.modulo = value,
            0xFF07 => self.write_control(value),
            _ => panic!("Invalid byte write for Timer: {:#06X}", address),
        }
    }
}
