use std::{cell::RefCell, rc::Rc};

use bitfields::bitfield;
use ironboyadvance_common::{memory::SystemMemoryAccess, register_ops::RegisterOps, scheduler::Scheduler};

use crate::events::{GbaEvent, InterruptEvent, TimerEvent};

const PRESCALER_SELECTIONS: [usize; 4] = [1, 64, 256, 1024];

const TIMER_OVERFLOW_INTERRUPTS: [InterruptEvent; 4] = [
    InterruptEvent::Timer0Overflow,
    InterruptEvent::Timer1Overflow,
    InterruptEvent::Timer2Overflow,
    InterruptEvent::Timer3Overflow,
];

#[bitfield(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TimerControl {
    #[bits(2)]
    prescaler_selection: u8,
    cascade_enabled: bool,
    #[bits(3)]
    _not_used_3_5: u8,
    irq_enabled: bool,
    enabled: bool,
    _not_used_8_15: u8,
}

impl TimerControl {
    pub fn prescaler(&self) -> usize {
        PRESCALER_SELECTIONS[self.prescaler_selection() as usize]
    }
}

impl RegisterOps<u16> for TimerControl {
    fn register(&self) -> u16 {
        self.into_bits()
    }

    fn write_register(&mut self, bits: u16) {
        self.set_bits(bits);
    }
}

pub struct Timer {
    id: usize,
    counter: usize,
    reload: u16,
    control: TimerControl,
    start_time: usize,
    active: bool,
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
}

impl Timer {
    pub fn new(id: usize, scheduler: Rc<RefCell<Scheduler<GbaEvent>>>) -> Self {
        let start_time = scheduler.borrow().timestamp();
        Self {
            id,
            counter: 0,
            reload: 0,
            control: TimerControl::from_bits(0),
            start_time,
            active: false,
            scheduler,
        }
    }

    fn delta_time(&self) -> usize {
        (self.scheduler.borrow().timestamp() - self.start_time) / self.control.prescaler()
    }

    pub fn read_counter(&self) -> u16 {
        let mut counter = self.counter;
        if self.active {
            counter = counter.wrapping_add(self.delta_time())
        }
        counter as u16
    }

    pub fn write_reload(&mut self, address: u32, value: u8) {
        self.scheduler.borrow_mut().schedule((
            GbaEvent::Timer(TimerEvent::ReloadWrite {
                timer_id: self.id,
                address,
                value,
            }),
            1,
        ));
    }

    pub fn write_control(&mut self, address: u32, value: u8) {
        if address & 1 == 1 {
            return; // TMxCNT_H high byte contains unused bits, nothing to schedule
        }

        self.scheduler.borrow_mut().schedule((
            GbaEvent::Timer(TimerEvent::ControlWrite {
                timer_id: self.id,
                value,
            }),
            1,
        ));
    }

    fn reload(&mut self) {
        self.counter = self.reload as usize;

        if self.control.irq_enabled() {
            self.scheduler
                .borrow_mut()
                .schedule((GbaEvent::Interrupt(TIMER_OVERFLOW_INTERRUPTS[self.id]), 0));
        }
    }

    fn stop(&mut self) {
        self.active = false;
        self.scheduler
            .borrow_mut()
            .cancel_events(GbaEvent::Timer(TimerEvent::Overflow { timer_id: self.id }));
    }

    fn start(&mut self) {
        let current_time = self.scheduler.borrow().timestamp();
        let prescalar = self.control.prescaler();
        let elapsed_time = current_time % prescalar;

        self.start_time = current_time - elapsed_time;
        self.active = true;

        let cycles = (0x10000 - self.counter) * prescalar - elapsed_time;
        self.scheduler
            .borrow_mut()
            .schedule((GbaEvent::Timer(TimerEvent::Overflow { timer_id: self.id }), cycles));
    }
}

pub struct TimerController {
    timers: [Timer; 4],
}

impl TimerController {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbaEvent>>>) -> Self {
        Self {
            timers: std::array::from_fn(|index| Timer::new(index, scheduler.clone())),
        }
    }

    pub fn handle_event(&mut self, event: TimerEvent) {
        match event {
            TimerEvent::Overflow { timer_id } => self.handle_overflow(timer_id),
            TimerEvent::ControlWrite { timer_id, value } => self.handle_control_write(timer_id, value),
            TimerEvent::ReloadWrite {
                timer_id,
                address,
                value,
            } => self.handle_reload_write(timer_id, address, value),
        }
    }

    fn handle_overflow(&mut self, timer_id: usize) {
        self.cascade(timer_id);
        self.timers[timer_id].start();
    }

    fn handle_reload_write(&mut self, timer_id: usize, address: u32, value: u8) {
        self.timers[timer_id].reload.write_byte(address, value);
    }

    fn handle_control_write(&mut self, timer_id: usize, value: u8) {
        let new_control = TimerControl::from_bits(value as u16);
        let was_enabled = self.timers[timer_id].control.enabled();

        if self.timers[timer_id].active {
            let timer = &self.timers[timer_id];
            let counter = timer.counter.wrapping_add(timer.delta_time());
            if counter >= 0x10000 {
                self.cascade(timer_id);
            }

            self.timers[timer_id].stop();
        }

        self.timers[timer_id].control = new_control;
        if !new_control.enabled() {
            return;
        }

        if !was_enabled {
            self.timers[timer_id].counter = self.timers[timer_id].reload as usize;
        }

        if !new_control.cascade_enabled() {
            self.timers[timer_id].start();
        }
    }

    fn cascade(&mut self, timer_id: usize) {
        self.timers[timer_id].reload();

        if timer_id <= 1 {
            //TODO: Handle sample rate for DMA sound channel A and/or B
        }

        if timer_id < 3 {
            let next = &mut self.timers[timer_id + 1];
            if next.control.enabled() && next.control.cascade_enabled() {
                next.counter = next.counter.wrapping_add(1);
                if next.counter == 0x10000 {
                    self.cascade(timer_id + 1);
                }
            }
        }
    }
}

impl SystemMemoryAccess for TimerController {
    fn read_8(&self, address: u32) -> u8 {
        match address {
            // TM0CNT_L, TM0CNT_H
            0x04000100..=0x04000101 => self.timers[0].read_counter().read_byte(address),
            0x04000102..=0x04000103 => self.timers[0].control.read_byte(address),
            // TM1CNT_L,  TM1CNT_H
            0x04000104..=0x04000105 => self.timers[1].read_counter().read_byte(address),
            0x04000106..=0x04000107 => self.timers[1].control.read_byte(address),
            // TM2CNT_L, TM2CNT_H
            0x04000108..=0x04000109 => self.timers[2].read_counter().read_byte(address),
            0x0400010A..=0x0400010B => self.timers[2].control.read_byte(address),
            // TM3CNT_L, TM3CNT_H
            0x0400010C..=0x0400010D => self.timers[3].read_counter().read_byte(address),
            0x0400010E..=0x0400010F => self.timers[3].control.read_byte(address),
            _ => panic!("Invalid byte read for TimerController: {:#010X}", address),
        }
    }

    fn write_8(&mut self, address: u32, value: u8) {
        match address {
            // TM0CNT_L, TM0CNT_H
            0x04000100..=0x04000101 => self.timers[0].write_reload(address, value),
            0x04000102..=0x04000103 => self.timers[0].write_control(address, value),
            // TM1CNT_L,  TM1CNT_H
            0x04000104..=0x04000105 => self.timers[1].write_reload(address, value),
            0x04000106..=0x04000107 => self.timers[1].write_control(address, value),
            // TM2CNT_L, TM2CNT_H
            0x04000108..=0x04000109 => self.timers[2].write_reload(address, value),
            0x0400010A..=0x0400010B => self.timers[2].write_control(address, value),
            // TM3CNT_L, TM3CNT_H
            0x0400010C..=0x0400010D => self.timers[3].write_reload(address, value),
            0x0400010E..=0x0400010F => self.timers[3].write_control(address, value),
            _ => panic!("Invalid byte write for TimerController: {:#010X}", address),
        }
    }
}
