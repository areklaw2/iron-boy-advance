use std::cell::RefCell;
use std::rc::Rc;

use bitfields::bitfield;
use ironboyadvance_arm7tdmi::CPU_CLOCK_SPEED;
use ironboyadvance_common::scheduler::Scheduler;

use crate::events::GbaEvent;

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
struct RtcPins {
    serial_clock: bool,
    serial_data: bool,
    chip_select: bool,
    #[bits(5)]
    _not_used_3_7: u8,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
struct RtcControl {
    _not_used_0: bool,
    irq_duty_hold: bool,
    _not_used_2: bool,
    per_minute_irq: bool,
    _not_used_4: bool,
    unknown_5: bool,
    hour_mode_24: bool,
    power_off: bool,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
struct HourRegister {
    #[bits(6)]
    hour: u8,
    _not_used_6: bool,
    afternoon: bool,
}

struct DateTime {
    year: u8,
    month: u8,
    day: u8,
    day_of_week: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum RtcRegister {
    ForceReset,
    Control,
    DateTime,
    Time,
    ForceIrq,
    Unused,
}

impl RtcRegister {
    fn from_command(command: u8) -> RtcRegister {
        match command {
            0 => RtcRegister::ForceReset,
            1 => RtcRegister::Control,
            2 => RtcRegister::DateTime,
            3 => RtcRegister::Time,
            6 => RtcRegister::ForceIrq,
            _ => RtcRegister::Unused,
        }
    }

    fn byte_count(self) -> usize {
        match self {
            RtcRegister::ForceReset | RtcRegister::ForceIrq => 0,
            RtcRegister::Control | RtcRegister::Unused => 1,
            RtcRegister::Time => 3,
            RtcRegister::DateTime => 7,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum TransferState {
    WaitingForChipSelectLow,
    WaitingForChipSelectHigh,
    ReceivingCommand,
    Reading,
    Writing,
}

pub struct Rtc {
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
    base_unix_seconds: u64,
    offset_seconds: i64,
    day_of_week_offset: i64,
    control: RtcControl,
    state: TransferState,
    previous_pins: RtcPins,
    register: RtcRegister,
    bytes: [u8; 7],
    byte_index: usize,
    bits_transferred: u8,
}

impl Rtc {
    pub fn new(base_unix_seconds: u64, scheduler: Rc<RefCell<Scheduler<GbaEvent>>>) -> Rtc {
        let mut control = RtcControl::from_bits(0);
        control.set_power_off(true);
        control.set_irq_duty_hold(true);

        Rtc {
            scheduler,
            base_unix_seconds,
            offset_seconds: 0,
            day_of_week_offset: 3,
            control,
            state: TransferState::WaitingForChipSelectLow,
            previous_pins: RtcPins::from_bits(0),
            register: RtcRegister::Unused,
            bytes: [0; 7],
            byte_index: 0,
            bits_transferred: 0,
        }
    }

    pub fn read_pins(&self) -> u8 {
        let outgoing_bit = match self.state {
            TransferState::Reading => (self.bytes[self.byte_index] >> self.bits_transferred) & 1 != 0,
            _ => false,
        };

        let mut pins = RtcPins::from_bits(0);
        pins.set_serial_data(outgoing_bit);
        pins.into_bits()
    }

    pub fn write_pins(&mut self, pins: u8) {
        let pins = RtcPins::from_bits(pins);
        let previous_pins = self.previous_pins;
        self.previous_pins = pins;

        let clock_rising = pins.serial_clock() && !previous_pins.serial_clock();

        match self.state {
            TransferState::WaitingForChipSelectLow => {
                if pins.serial_clock() && !pins.chip_select() {
                    self.state = TransferState::WaitingForChipSelectHigh;
                }
            }
            TransferState::WaitingForChipSelectHigh => {
                if pins.serial_clock() && pins.chip_select() {
                    self.state = TransferState::ReceivingCommand;
                    self.reset_buffer();
                }
            }
            TransferState::ReceivingCommand | TransferState::Reading | TransferState::Writing => match pins.chip_select() {
                false => self.end_transfer(),
                true => {
                    if clock_rising {
                        self.transfer_bit(pins);
                    }
                }
            },
        }
    }

    fn reset_buffer(&mut self) {
        self.bytes = [0; 7];
        self.byte_index = 0;
        self.bits_transferred = 0;
    }

    fn end_transfer(&mut self) {
        self.state = TransferState::WaitingForChipSelectLow;
    }

    fn transfer_bit(&mut self, pins: RtcPins) {
        if self.state != TransferState::Reading {
            self.bytes[self.byte_index] |= (pins.serial_data() as u8) << self.bits_transferred;
        }

        self.bits_transferred += 1;
        if self.bits_transferred < 8 {
            return;
        }

        self.bits_transferred = 0;
        match self.state {
            TransferState::ReceivingCommand => self.decode_command(),
            _ => {
                self.byte_index += 1;
                if self.byte_index >= self.register.byte_count() {
                    if self.state == TransferState::Writing {
                        self.apply_written_bytes();
                    }
                    self.end_transfer();
                }
            }
        }
    }

    fn decode_command(&mut self) {
        let command = match self.bytes[0] & 0x0F == 0x06 {
            true => self.bytes[0].reverse_bits(),
            false => self.bytes[0],
        };

        if command & 0xF0 != 0x60 {
            self.end_transfer();
            return;
        }

        self.register = RtcRegister::from_command((command >> 1) & 0b111);
        self.reset_buffer();

        match self.register {
            RtcRegister::ForceReset => {
                self.force_reset();
                self.end_transfer();
            }
            RtcRegister::ForceIrq => self.end_transfer(),
            _ => match command & 1 != 0 {
                true => {
                    self.load_registers();
                    self.state = TransferState::Reading;
                }
                false => self.state = TransferState::Writing,
            },
        }
    }

    fn load_registers(&mut self) {
        match self.register {
            RtcRegister::Control => {
                self.bytes[0] = self.control.into_bits();
                self.control.set_power_off(false);
            }
            RtcRegister::DateTime => {
                let date_time = self.current_date_time();
                self.bytes[0] = to_binary_coded_decimal(date_time.year);
                self.bytes[1] = to_binary_coded_decimal(date_time.month);
                self.bytes[2] = to_binary_coded_decimal(date_time.day);
                self.bytes[3] = date_time.day_of_week;
                let time = self.encode_time(&date_time);
                self.bytes[4..7].copy_from_slice(&time);
            }
            RtcRegister::Time => {
                let time = self.encode_time(&self.current_date_time());
                self.bytes[0..3].copy_from_slice(&time);
            }
            _ => self.bytes[0] = 0xFF,
        }
    }

    fn apply_written_bytes(&mut self) {
        match self.register {
            RtcRegister::Control => {
                let written = RtcControl::from_bits(self.bytes[0]);
                self.control.set_irq_duty_hold(written.irq_duty_hold());
                self.control.set_per_minute_irq(written.per_minute_irq());
                self.control.set_unknown_5(written.unknown_5());
                self.control.set_hour_mode_24(written.hour_mode_24());
            }
            RtcRegister::DateTime => {
                let year = 2000 + from_binary_coded_decimal(self.bytes[0]) as i64;
                let month = from_binary_coded_decimal(self.bytes[1]) as i64;
                let day = from_binary_coded_decimal(self.bytes[2]) as i64;
                let days = days_from_civil(year, month, day);
                let seconds_of_day = self.decode_seconds_of_day(&self.bytes[4..7]);
                self.day_of_week_offset = (self.bytes[3] as i64 - days).rem_euclid(7);
                self.set_clock(days * SECONDS_PER_DAY + seconds_of_day);
            }
            RtcRegister::Time => {
                let days = self.current_unix_seconds().div_euclid(SECONDS_PER_DAY);
                let seconds_of_day = self.decode_seconds_of_day(&self.bytes[0..3]);
                self.set_clock(days * SECONDS_PER_DAY + seconds_of_day);
            }
            _ => {}
        }
    }

    fn force_reset(&mut self) {
        self.control = RtcControl::from_bits(0);
        self.offset_seconds = 0;
    }

    fn set_clock(&mut self, unix_seconds: i64) {
        self.offset_seconds = unix_seconds - self.unadjusted_unix_seconds() as i64;
    }

    fn encode_time(&self, date_time: &DateTime) -> [u8; 3] {
        [
            self.encode_hour(date_time.hour),
            to_binary_coded_decimal(date_time.minute),
            to_binary_coded_decimal(date_time.second),
        ]
    }

    fn decode_seconds_of_day(&self, time: &[u8]) -> i64 {
        let register = HourRegister::from_bits(time[0]);
        let hour = from_binary_coded_decimal(register.hour()) as i64;
        let hour = match !self.control.hour_mode_24() && register.afternoon() {
            true => hour % 12 + 12,
            false => hour,
        };
        hour * 3600 + from_binary_coded_decimal(time[1]) as i64 * 60 + from_binary_coded_decimal(time[2]) as i64
    }

    fn encode_hour(&self, hour: u8) -> u8 {
        let displayed_hour = match self.control.hour_mode_24() {
            true => hour,
            false => hour % 12,
        };

        let mut register = HourRegister::from_bits(0);
        register.set_hour(to_binary_coded_decimal(displayed_hour));
        register.set_afternoon(hour >= 12);
        register.into_bits()
    }

    fn unadjusted_unix_seconds(&self) -> u64 {
        self.base_unix_seconds + (self.scheduler.borrow().timestamp() / CPU_CLOCK_SPEED as usize) as u64
    }

    fn current_unix_seconds(&self) -> i64 {
        self.unadjusted_unix_seconds() as i64 + self.offset_seconds
    }

    fn current_date_time(&self) -> DateTime {
        let unix_seconds = self.current_unix_seconds();
        let days = unix_seconds.div_euclid(SECONDS_PER_DAY);
        let seconds_of_day = unix_seconds.rem_euclid(SECONDS_PER_DAY);
        let (year, month, day) = civil_from_days(days);

        DateTime {
            year: (year - 2000).rem_euclid(100) as u8,
            month: month as u8,
            day: day as u8,
            day_of_week: (days + self.day_of_week_offset).rem_euclid(7) as u8,
            hour: (seconds_of_day / 3600) as u8,
            minute: (seconds_of_day / 60 % 60) as u8,
            second: (seconds_of_day % 60) as u8,
        }
    }
}

fn to_binary_coded_decimal(value: u8) -> u8 {
    ((value / 10) << 4) | (value % 10)
}

fn from_binary_coded_decimal(value: u8) -> u8 {
    (value >> 4) * 10 + (value & 0x0F)
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted_days = days + 719468;
    let era = shifted_days.div_euclid(146097);
    let day_of_era = shifted_days.rem_euclid(146097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = match shifted_month < 10 {
        true => shifted_month + 3,
        false => shifted_month - 9,
    };
    let year = year_of_era + era * 400 + (month <= 2) as i64;

    (year, month, day)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - (month <= 2) as i64;
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let shifted_month = match month > 2 {
        true => month - 3,
        false => month + 9,
    };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146097 + day_of_era - 719468
}
