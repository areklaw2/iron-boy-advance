use std::ops::BitOr;

use getset::CopyGetters;

use crate::cartridge::eeprom::EepromSize;
use crate::cartridge::header::Header;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CartridgeDevice {
    Rtc = 0b1,
    SolarSensor = 0b10,
    Tilt = 0b100,
    Gyro = 0b01000,
    Rumble = 0b10000,
    EReader = 0b100000,
}

impl CartridgeDevice {
    pub fn is_set(self, pattern: u8) -> bool {
        pattern & self as u8 != 0
    }
}

impl BitOr for CartridgeDevice {
    type Output = u8;
    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

const BACKUP_TYPE_STRINGS: &[&str] = &["SRAM_V", "EEPROM_V", "FLASH1M_V", "FLASH512_V", "FLASH_V"];

#[derive(Debug, Clone, Copy)]
pub enum BackupType {
    None,
    Sram,
    Eeprom,
    Flash64KB,
    Flash128KB,
}

#[derive(Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct CartridgeConfig {
    backup_type: BackupType,
    device_pattern: u8,
    eeprom_size: Option<EepromSize>,
}

pub fn determine_cartridge_config(data: &[u8], header: &Header) -> CartridgeConfig {
    lookup_config(header.game_code()).unwrap_or_else(|| {
        let backup_type = detect_backup_type(data);
        CartridgeConfig {
            backup_type,
            device_pattern: 0,
            eeprom_size: matches!(backup_type, BackupType::Eeprom).then_some(EepromSize::Small),
        }
    })
}

fn detect_backup_type(rom: &[u8]) -> BackupType {
    (0..rom.len())
        .step_by(4)
        .find_map(|offset| {
            BACKUP_TYPE_STRINGS
                .iter()
                .find(|backup_type| rom[offset..].starts_with(backup_type.as_bytes()))
                .copied()
        })
        .map_or(BackupType::None, |s| match s {
            "SRAM_V" => BackupType::Sram,
            "EEPROM_V" => BackupType::Eeprom,
            "FLASH_V" | "FLASH512_V" => BackupType::Flash64KB,
            "FLASH1M_V" => BackupType::Flash128KB,
            _ => unreachable!(),
        })
}

// Entries adapted from mGBA's overrides.c (MPL 2.0):
// https://github.com/mgba-emu/mgba/blob/master/src/gba/overrides.c
// Covers GPIO devices, false-positive force-none, and the Classic NES F-prefix shortcut.

fn lookup_config(game_code: &str) -> Option<CartridgeConfig> {
    use CartridgeDevice::*;
    let config = match game_code {
        // Boktai - The Sun Is in Your Hand
        "U3IJ" | "U3IE" | "U3IP" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: Rtc | SolarSensor,
            eeprom_size: Some(EepromSize::Large),
        },

        // Boktai 2 - Solar Boy Django
        "U32J" | "U32E" | "U32P" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: Rtc | SolarSensor,
            eeprom_size: Some(EepromSize::Large),
        },

        // Dragon Ball Z - The Legacy of Goku
        "ALGP" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: 0,
            eeprom_size: Some(EepromSize::Large),
        },

        // Dragon Ball Z - The Legacy of Goku II
        "ALFJ" | "ALFE" | "ALFP" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: 0,
            eeprom_size: Some(EepromSize::Large),
        },

        // Drill Dozer
        "V49J" | "V49E" | "V49P" => CartridgeConfig {
            backup_type: BackupType::Sram,
            device_pattern: Rumble as u8,
            eeprom_size: None,
        },

        // e-Reader
        "PEAJ" | "PSAJ" | "PSAE" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: EReader as u8,
            eeprom_size: None,
        },

        // Game Boy Wars Advance 1+2
        "BGWJ" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: 0,
            eeprom_size: None,
        },

        // Goodboy Galaxy
        "2GBP" => CartridgeConfig {
            backup_type: BackupType::Sram,
            device_pattern: Rumble as u8,
            eeprom_size: None,
        },

        // Koro Koro Puzzle - Happy Panechu!
        "KHPJ" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: Tilt as u8,
            eeprom_size: Some(EepromSize::Large),
        },

        // Legendz - Yomigaeru Shiren no Shima
        "BLJJ" | "BLJK" => CartridgeConfig {
            backup_type: BackupType::Flash64KB,
            device_pattern: Rtc as u8,
            eeprom_size: None,
        },

        // Legendz - Sign of Nekuromu
        "BLVJ" => CartridgeConfig {
            backup_type: BackupType::Flash64KB,
            device_pattern: Rtc as u8,
            eeprom_size: None,
        },

        // Pokemon Ruby
        "AXVJ" | "AXVE" | "AXVP" | "AXVI" | "AXVS" | "AXVD" | "AXVF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: Rtc as u8,
            eeprom_size: None,
        },

        // Pokemon Sapphire
        "AXPJ" | "AXPE" | "AXPP" | "AXPI" | "AXPS" | "AXPD" | "AXPF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: Rtc as u8,
            eeprom_size: None,
        },

        // Pokemon Emerald
        "BPEJ" | "BPEE" | "BPEP" | "BPEI" | "BPES" | "BPED" | "BPEF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: Rtc as u8,
            eeprom_size: None,
        },

        // Pokemon FireRed
        "BPRJ" | "BPRE" | "BPRP" | "BPRI" | "BPRS" | "BPRD" | "BPRF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: 0,
            eeprom_size: None,
        },

        // Pokemon LeafGreen
        "BPGJ" | "BPGE" | "BPGP" | "BPGI" | "BPGS" | "BPGD" | "BPGF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: 0,
            eeprom_size: None,
        },

        // RockMan EXE 4.5 - Real Operation
        "BR4J" => CartridgeConfig {
            backup_type: BackupType::Flash64KB,
            device_pattern: Rtc as u8,
            eeprom_size: None,
        },

        // Sennen Kazoku
        "BKAJ" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: Rtc as u8,
            eeprom_size: None,
        },

        // Shin Bokura no Taiyou - Gyakushuu no Sabata (Boktai 3)
        "U33J" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: Rtc | SolarSensor,
            eeprom_size: Some(EepromSize::Large),
        },

        // Wario Ware Twisted!
        "RZWJ" | "RZWE" | "RZWP" => CartridgeConfig {
            backup_type: BackupType::Sram,
            device_pattern: Rumble | Gyro,
            eeprom_size: None,
        },

        // Yoshi - Topsy-Turvy / Yoshi's Universal Gravitation
        "KYGJ" | "KYGE" | "KYGP" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: Tilt as u8,
            eeprom_size: Some(EepromSize::Large),
        },

        // FORCE_NONE — carts whose ROM contains stray bytes that false-positive the scan
        // Iridion II
        "AI2E" | "AI2P" => CartridgeConfig {
            backup_type: BackupType::None,
            device_pattern: 0,
            eeprom_size: None,
        },

        // Stuart Little 2
        "ASLE" | "ASLF" => CartridgeConfig {
            backup_type: BackupType::None,
            device_pattern: 0,
            eeprom_size: None,
        },

        // Top Gun - Combat Zones
        "A2YE" => CartridgeConfig {
            backup_type: BackupType::None,
            device_pattern: 0,
            eeprom_size: None,
        },

        // Classic NES Series — covers all F-prefix titles (~30 games)
        _ if game_code.starts_with('F') => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: 0,
            eeprom_size: Some(EepromSize::Large),
        },

        _ => return None,
    };
    Some(config)
}
