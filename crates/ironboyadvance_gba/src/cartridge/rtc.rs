use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::rc::Rc;

use bitfields::bitfield;
use ironboyadvance_arm7tdmi::CPU_CLOCK_SPEED;
use ironboyadvance_common::scheduler::Scheduler;
use tracing::warn;

use crate::events::GbaEvent;

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;
pub(super) const RTC_SAVE_BYTES: usize = 16;
const DEFAULT_DAY_OF_WEEK_OFFSET: i64 = 3;

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
    ReceivingCommand {
        bits_transferred: u8,
    },
    Reading {
        register: RtcRegister,
        byte_index: usize,
        bits_transferred: u8,
    },
    Writing {
        register: RtcRegister,
        byte_index: usize,
        bits_transferred: u8,
    },
}

pub struct Rtc {
    scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
    save_file: PathBuf,
    save_offset: usize,
    base_unix_seconds: u64,
    offset_seconds: i64,
    day_of_week_offset: i64,
    control: RtcControl,
    state: TransferState,
    previous_pins: RtcPins,
    bytes: [u8; 7],
    output_bit: bool,
}

impl Rtc {
    pub fn new(
        base_unix_seconds: u64,
        save_file: PathBuf,
        save_offset: usize,
        scheduler: Rc<RefCell<Scheduler<GbaEvent>>>,
    ) -> Rtc {
        let mut control = RtcControl::from_bits(0);
        control.set_hour_mode_24(true);
        let (offset_seconds, day_of_week_offset) = load_save(&save_file, save_offset);

        Rtc {
            scheduler,
            save_file,
            save_offset,
            base_unix_seconds,
            offset_seconds,
            day_of_week_offset,
            control,
            state: TransferState::WaitingForChipSelectLow,
            previous_pins: RtcPins::from_bits(0),
            bytes: [0; 7],
            output_bit: false,
        }
    }

    pub fn read_pins(&self) -> u8 {
        let mut pins = RtcPins::from_bits(0);
        pins.set_serial_data(self.output_bit);
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
                    self.bytes = [0; 7];
                    self.state = TransferState::ReceivingCommand { bits_transferred: 0 };
                }
            }
            TransferState::ReceivingCommand { .. } | TransferState::Reading { .. } | TransferState::Writing { .. } => {
                match pins.chip_select() {
                    false => self.state = TransferState::WaitingForChipSelectLow,
                    true => {
                        if clock_rising {
                            self.transfer_bit(pins);
                        }
                    }
                }
            }
        }
    }

    fn transfer_bit(&mut self, pins: RtcPins) {
        match self.state {
            TransferState::ReceivingCommand { bits_transferred } => {
                self.bytes[0] |= (pins.serial_data() as u8) << bits_transferred;
                match bits_transferred == 7 {
                    true => self.decode_command(),
                    false => {
                        self.state = TransferState::ReceivingCommand {
                            bits_transferred: bits_transferred + 1,
                        }
                    }
                }
            }
            TransferState::Reading {
                register,
                byte_index,
                bits_transferred,
            } => {
                self.output_bit = (self.bytes[byte_index] >> bits_transferred) & 1 != 0;
                self.state = match next_position(register, byte_index, bits_transferred) {
                    Some((byte_index, bits_transferred)) => TransferState::Reading {
                        register,
                        byte_index,
                        bits_transferred,
                    },
                    None => TransferState::WaitingForChipSelectLow,
                };
            }
            TransferState::Writing {
                register,
                byte_index,
                bits_transferred,
            } => {
                self.bytes[byte_index] |= (pins.serial_data() as u8) << bits_transferred;
                self.state = match next_position(register, byte_index, bits_transferred) {
                    Some((byte_index, bits_transferred)) => TransferState::Writing {
                        register,
                        byte_index,
                        bits_transferred,
                    },
                    None => {
                        self.apply_written_bytes(register);
                        TransferState::WaitingForChipSelectLow
                    }
                };
            }
            TransferState::WaitingForChipSelectLow | TransferState::WaitingForChipSelectHigh => {}
        }
    }

    fn decode_command(&mut self) {
        let command = match self.bytes[0] & 0x0F == 0x06 {
            true => self.bytes[0].reverse_bits(),
            false => self.bytes[0],
        };

        if command & 0xF0 != 0x60 {
            self.state = TransferState::WaitingForChipSelectLow;
            return;
        }

        let register = RtcRegister::from_command((command >> 1) & 0b111);
        self.bytes = [0; 7];

        self.state = match register {
            RtcRegister::ForceReset => {
                self.force_reset();
                TransferState::WaitingForChipSelectLow
            }
            RtcRegister::ForceIrq => TransferState::WaitingForChipSelectLow,
            _ => match command & 1 != 0 {
                true => {
                    self.load_registers(register);
                    TransferState::Reading {
                        register,
                        byte_index: 0,
                        bits_transferred: 0,
                    }
                }
                false => TransferState::Writing {
                    register,
                    byte_index: 0,
                    bits_transferred: 0,
                },
            },
        };
    }

    fn load_registers(&mut self, register: RtcRegister) {
        match register {
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

    fn apply_written_bytes(&mut self, register: RtcRegister) {
        match register {
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
        let days = days_from_civil(2000, 1, 1);
        self.control = RtcControl::from_bits(0);
        self.day_of_week_offset = (-days).rem_euclid(7);
        self.set_clock(days * SECONDS_PER_DAY);
    }

    fn set_clock(&mut self, unix_seconds: i64) {
        self.offset_seconds = unix_seconds - self.unadjusted_unix_seconds() as i64;
        self.save();
    }

    fn save(&self) {
        let mut bytes = [0u8; RTC_SAVE_BYTES];
        bytes[0..8].copy_from_slice(&self.offset_seconds.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.day_of_week_offset.to_le_bytes());

        let result = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.save_file)
            .and_then(|mut file| {
                file.seek(SeekFrom::Start(self.save_offset as u64))
                    .and_then(|_| file.write_all(&bytes))
            });

        if let Err(error) = result {
            warn!("rtc save failed at {:?}: {}", self.save_file, error);
        }
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

fn next_position(register: RtcRegister, byte_index: usize, bits_transferred: u8) -> Option<(usize, u8)> {
    match bits_transferred == 7 {
        false => Some((byte_index, bits_transferred + 1)),
        true => match byte_index + 1 >= register.byte_count() {
            true => None,
            false => Some((byte_index + 1, 0)),
        },
    }
}

fn load_save(path: &PathBuf, offset: usize) -> (i64, i64) {
    let mut bytes = [0u8; RTC_SAVE_BYTES];
    let read = OpenOptions::new().read(true).open(path).and_then(|mut file| {
        file.seek(SeekFrom::Start(offset as u64))
            .and_then(|_| file.read_exact(&mut bytes))
    });

    match read {
        Ok(()) => (
            i64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            i64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        ),
        Err(_) => (0, DEFAULT_DAY_OF_WEEK_OFFSET),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::gpio::Gpio;

    const DATA_ADDRESS: u32 = 0x080000C4;
    const DIRECTION_ADDRESS: u32 = 0x080000C6;
    const CONTROL_ADDRESS: u32 = 0x080000C8;

    const SERIAL_CLOCK: u16 = 1;
    const SERIAL_DATA: u16 = 2;
    const CHIP_SELECT: u16 = 4;

    const NEW_YEARS_DAY_2026: u64 = 1767225600;
    const AFTERNOON: u64 = 13 * 3600 + 45 * 60 + 30;

    fn command(register: RtcRegister, read: bool) -> u8 {
        let index: u8 = match register {
            RtcRegister::ForceReset => 0,
            RtcRegister::Control => 1,
            RtcRegister::DateTime => 2,
            RtcRegister::Time => 3,
            RtcRegister::ForceIrq => 6,
            RtcRegister::Unused => 7,
        };

        (0x60 | (index << 1) | read as u8).reverse_bits()
    }

    fn save_path(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("ironboyadvance-rtc-{name}.sav"));
        let _ = std::fs::remove_file(&path);
        path
    }

    fn open(path: PathBuf, unix_seconds: u64) -> Gpio {
        let scheduler = Rc::new(RefCell::new(Scheduler::new()));
        let mut gpio = Gpio::new(Some(Rtc::new(unix_seconds, path, 0, scheduler)));
        gpio.write_16(CONTROL_ADDRESS, 1);
        gpio
    }

    fn harness(name: &str, unix_seconds: u64) -> Gpio {
        open(save_path(name), unix_seconds)
    }

    fn begin_transfer(gpio: &mut Gpio) {
        gpio.write_16(DIRECTION_ADDRESS, SERIAL_CLOCK | SERIAL_DATA | CHIP_SELECT);
        gpio.write_16(DATA_ADDRESS, SERIAL_CLOCK);
        gpio.write_16(DATA_ADDRESS, SERIAL_CLOCK | CHIP_SELECT);
    }

    fn end_transfer(gpio: &mut Gpio) {
        gpio.write_16(DATA_ADDRESS, 0);
    }

    fn send_byte(gpio: &mut Gpio, byte: u8) {
        gpio.write_16(DIRECTION_ADDRESS, SERIAL_CLOCK | SERIAL_DATA | CHIP_SELECT);
        for index in 0..8 {
            let bit = ((byte >> index) & 1) as u16;
            gpio.write_16(DATA_ADDRESS, CHIP_SELECT | (bit * SERIAL_DATA));
            gpio.write_16(DATA_ADDRESS, CHIP_SELECT | (bit * SERIAL_DATA) | SERIAL_CLOCK);
        }
    }

    fn receive_byte(gpio: &mut Gpio) -> u8 {
        gpio.write_16(DIRECTION_ADDRESS, SERIAL_CLOCK | CHIP_SELECT);
        let mut byte = 0;
        for index in 0..8 {
            gpio.write_16(DATA_ADDRESS, CHIP_SELECT);
            gpio.write_16(DATA_ADDRESS, CHIP_SELECT | SERIAL_CLOCK);
            let bit = (gpio.read_16(DATA_ADDRESS) & SERIAL_DATA) >> 1;
            byte |= (bit as u8) << index;
        }
        byte
    }

    fn read_register(gpio: &mut Gpio, command: u8, count: usize) -> Vec<u8> {
        begin_transfer(gpio);
        send_byte(gpio, command);
        let bytes = (0..count).map(|_| receive_byte(gpio)).collect();
        end_transfer(gpio);
        bytes
    }

    fn write_register(gpio: &mut Gpio, command: u8, bytes: &[u8]) {
        begin_transfer(gpio);
        send_byte(gpio, command);
        for byte in bytes {
            send_byte(gpio, *byte);
        }
        end_transfer(gpio);
    }

    #[test]
    fn control_boots_in_24_hour_mode_without_power_failure() {
        let mut gpio = harness("boot_control", NEW_YEARS_DAY_2026);
        assert_eq!(read_register(&mut gpio, command(RtcRegister::Control, true), 1), vec![0x40]);
    }

    #[test]
    fn date_time_reads_back_as_binary_coded_decimal() {
        let mut gpio = harness("date_time", NEW_YEARS_DAY_2026 + AFTERNOON);
        let bytes = read_register(&mut gpio, command(RtcRegister::DateTime, true), 7);

        assert_eq!(bytes[0], 0x26, "year");
        assert_eq!(bytes[1], 0x01, "month");
        assert_eq!(bytes[2], 0x01, "day");
        assert_eq!(bytes[3], 3, "day of week, thursday with 0 = monday");
        assert_eq!(bytes[4], 0x93, "hour keeps the pm flag even in 24 hour mode");
        assert_eq!(bytes[5], 0x45, "minute");
        assert_eq!(bytes[6], 0x30, "second");
    }

    #[test]
    fn time_register_matches_the_date_time_register() {
        let mut gpio = harness("time", NEW_YEARS_DAY_2026 + AFTERNOON);
        let date_time = read_register(&mut gpio, command(RtcRegister::DateTime, true), 7);
        assert_eq!(
            read_register(&mut gpio, command(RtcRegister::Time, true), 3),
            date_time[4..7].to_vec()
        );
    }

    #[test]
    fn twelve_hour_mode_folds_afternoon_hours() {
        let mut gpio = harness("twelve_hour", NEW_YEARS_DAY_2026 + AFTERNOON);
        write_register(&mut gpio, command(RtcRegister::Control, false), &[0x00]);
        assert_eq!(
            read_register(&mut gpio, command(RtcRegister::DateTime, true), 7)[4],
            0x81,
            "1 pm in 12 hour mode"
        );
    }

    #[test]
    fn command_byte_accepts_either_bit_order() {
        let mut gpio = harness("bit_order", NEW_YEARS_DAY_2026);
        let reversed = read_register(&mut gpio, command(RtcRegister::Control, true), 1);
        let plain = read_register(&mut gpio, 0x63, 1);
        assert_eq!(reversed, plain, "0xC6 and 0x63 both select the control register");
    }

    #[test]
    fn written_date_time_round_trips() {
        let mut gpio = harness("round_trip", NEW_YEARS_DAY_2026);
        let written = vec![0x99, 0x12, 0x25, 0x05, 0x08, 0x30, 0x15];
        write_register(&mut gpio, command(RtcRegister::DateTime, false), &written);
        assert_eq!(read_register(&mut gpio, command(RtcRegister::DateTime, true), 7), written);
    }

    #[test]
    fn control_writes_cannot_set_the_read_only_power_failure_flag() {
        let mut gpio = harness("read_only_power", NEW_YEARS_DAY_2026);
        write_register(&mut gpio, command(RtcRegister::Control, false), &[0xFF]);
        assert_eq!(read_register(&mut gpio, command(RtcRegister::Control, true), 1), vec![0x6A]);
    }

    #[test]
    fn force_reset_zeroes_every_register() {
        let mut gpio = harness("force_reset", NEW_YEARS_DAY_2026 + AFTERNOON);
        begin_transfer(&mut gpio);
        send_byte(&mut gpio, command(RtcRegister::ForceReset, false));
        end_transfer(&mut gpio);

        let bytes = read_register(&mut gpio, command(RtcRegister::DateTime, true), 7);
        assert_eq!(&bytes[0..4], &[0x00, 0x01, 0x01, 0x00], "day and month reset to 01h");
        assert_eq!(&bytes[4..7], &[0x00, 0x00, 0x00], "time reset to midnight");
        assert_eq!(read_register(&mut gpio, command(RtcRegister::Control, true), 1), vec![0x00]);
    }

    #[test]
    fn clock_survives_a_reload() {
        let path = save_path("reload");
        let written = vec![0x99, 0x12, 0x25, 0x05, 0x08, 0x30, 0x15];

        let mut gpio = open(path.clone(), NEW_YEARS_DAY_2026);
        write_register(&mut gpio, command(RtcRegister::DateTime, false), &written);
        drop(gpio);

        let mut reloaded = open(path, NEW_YEARS_DAY_2026);
        assert_eq!(read_register(&mut reloaded, command(RtcRegister::DateTime, true), 7), written);
    }
}
