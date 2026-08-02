use std::{
    cell::RefCell,
    io::{Write, stdout},
    rc::Rc,
};

use bitfields::bitfield;
use ironboyadvance_common::{memory::SystemMemoryAccess, scheduler::Scheduler};
use ironboyadvance_sm83::{CPU_CLOCK_SPEED, GbSpeed};

use crate::events::{GbcEvent, InterruptEvent, SerialEvent};

const NORMAL_CLOCK_FREQUENCY: usize = 8192;
const FAST_CLOCK_FREQUENCY: usize = 262144;
const NORMAL_CLOCK_CYCLES: usize = CPU_CLOCK_SPEED as usize / NORMAL_CLOCK_FREQUENCY;
const FAST_CLOCK_CYCLES: usize = CPU_CLOCK_SPEED as usize / FAST_CLOCK_FREQUENCY;
const BITS_TO_TRANSFER: u8 = 8;
const DISCONNECTED_BIT: u8 = 1;

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
struct SerialTransferControl {
    internal_clock: bool,
    fast_clock: bool,
    #[bits(5)]
    _not_used_2_6: u8,
    transfer_start: bool,
}

pub struct SerialTransfer {
    serial_transfer_data: u8,
    serial_transfer_control: SerialTransferControl,
    bits_remaining: u8,
    transferred_byte: u8,
    speed: GbSpeed,
    scheduler: Rc<RefCell<Scheduler<GbcEvent>>>,
    //TODO: add an output vec for the transfer
}

impl SerialTransfer {
    pub fn new(scheduler: Rc<RefCell<Scheduler<GbcEvent>>>) -> Self {
        SerialTransfer {
            serial_transfer_data: 0,
            serial_transfer_control: SerialTransferControl::from_bits(0),
            bits_remaining: 0,
            transferred_byte: 0,
            speed: GbSpeed::Normal,
            scheduler,
        }
    }

    pub fn set_speed(&mut self, speed: GbSpeed) {
        self.speed = speed;
    }

    pub fn handle_event(&mut self, serial_event: SerialEvent, timestamp: usize) {
        match serial_event {
            SerialEvent::TransferBit => self.shift_bit(timestamp),
        }
    }

    fn shift_bit(&mut self, timestamp: usize) {
        self.serial_transfer_data = (self.serial_transfer_data << 1) | DISCONNECTED_BIT;
        self.bits_remaining -= 1;

        match self.bits_remaining {
            0 => self.complete_transfer(),
            _ => self.schedule_bit_transfer(timestamp),
        }
    }

    fn complete_transfer(&mut self) {
        self.serial_transfer_control.set_transfer_start(false);
        print!("{}", self.transferred_byte as char);
        let _ = stdout().flush();
        self.scheduler
            .borrow_mut()
            .schedule((GbcEvent::Interrupt(InterruptEvent::Serial), 0));
    }

    fn start_transfer(&mut self) {
        self.bits_remaining = BITS_TO_TRANSFER;
        self.transferred_byte = self.serial_transfer_data;
        let timestamp = self.scheduler.borrow().timestamp();
        self.schedule_bit_transfer(timestamp);
    }

    fn cancel_transfer(&mut self) {
        self.bits_remaining = 0;
        self.scheduler
            .borrow_mut()
            .cancel_events(GbcEvent::Serial(SerialEvent::TransferBit));
    }

    fn schedule_bit_transfer(&mut self, timestamp: usize) {
        let clock_cycles = match self.serial_transfer_control.fast_clock() {
            true => FAST_CLOCK_CYCLES,
            false => NORMAL_CLOCK_CYCLES,
        };

        let cycles = match self.speed {
            GbSpeed::Normal => clock_cycles,
            GbSpeed::Double => clock_cycles / 2,
        };

        self.scheduler
            .borrow_mut()
            .schedule_at_timestamp(GbcEvent::Serial(SerialEvent::TransferBit), timestamp + cycles);
    }

    fn write_control(&mut self, value: u8) {
        let transfer_in_progress = self.bits_remaining != 0;
        self.serial_transfer_control = SerialTransferControl::from_bits(value);

        match (self.serial_transfer_control.transfer_start(), transfer_in_progress) {
            (true, false) if self.serial_transfer_control.internal_clock() => self.start_transfer(),
            (false, true) => self.cancel_transfer(),
            _ => (),
        }
    }
}

impl SystemMemoryAccess for SerialTransfer {
    type Address = u16;

    fn read_8(&self, address: u16) -> u8 {
        match address {
            0xFF01 => self.serial_transfer_data,
            0xFF02 => self.serial_transfer_control.into_bits() | 0x7C,
            _ => panic!("Invalid byte read for SerialTransfer: {:#06X}", address),
        }
    }

    fn write_8(&mut self, address: u16, value: u8) {
        match address {
            0xFF01 => self.serial_transfer_data = value,
            0xFF02 => self.write_control(value),
            _ => panic!("Invalid byte write for SerialTransfer: {:#06X}", address),
        }
    }
}
