use std::ops::BitOr;

use getset::Getters;

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

#[derive(Debug)]
pub enum BackupType {
    None,
    Sram,
    Eeprom,
    Flash64KB,
    Flash128KB,
}

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct CartridgeConfig {
    backup_type: BackupType,
    device_pattern: u8,
}

pub fn determine_cartridge_config(data: &[u8], header: &Header) -> CartridgeConfig {
    lookup_config(header.game_code()).unwrap_or_else(|| CartridgeConfig {
        backup_type: detect_backup_type(data),
        device_pattern: 0,
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
        },

        // Boktai 2 - Solar Boy Django
        "U32J" | "U32E" | "U32P" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: Rtc | SolarSensor,
        },

        // Drill Dozer
        "V49J" | "V49E" | "V49P" => CartridgeConfig {
            backup_type: BackupType::Sram,
            device_pattern: Rumble as u8,
        },

        // e-Reader
        "PEAJ" | "PSAJ" | "PSAE" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: EReader as u8,
        },

        // Game Boy Wars Advance 1+2
        "BGWJ" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: 0,
        },

        // Goodboy Galaxy
        "2GBP" => CartridgeConfig {
            backup_type: BackupType::Sram,
            device_pattern: Rumble as u8,
        },

        // Koro Koro Puzzle - Happy Panechu!
        "KHPJ" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: Tilt as u8,
        },

        // Legendz - Yomigaeru Shiren no Shima
        "BLJJ" | "BLJK" => CartridgeConfig {
            backup_type: BackupType::Flash64KB,
            device_pattern: Rtc as u8,
        },

        // Legendz - Sign of Nekuromu
        "BLVJ" => CartridgeConfig {
            backup_type: BackupType::Flash64KB,
            device_pattern: Rtc as u8,
        },

        // Pokemon Ruby
        "AXVJ" | "AXVE" | "AXVP" | "AXVI" | "AXVS" | "AXVD" | "AXVF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: Rtc as u8,
        },

        // Pokemon Sapphire
        "AXPJ" | "AXPE" | "AXPP" | "AXPI" | "AXPS" | "AXPD" | "AXPF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: Rtc as u8,
        },

        // Pokemon Emerald
        "BPEJ" | "BPEE" | "BPEP" | "BPEI" | "BPES" | "BPED" | "BPEF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: Rtc as u8,
        },

        // Pokemon FireRed
        "BPRJ" | "BPRE" | "BPRP" | "BPRI" | "BPRS" | "BPRD" | "BPRF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: 0,
        },

        // Pokemon LeafGreen
        "BPGJ" | "BPGE" | "BPGP" | "BPGI" | "BPGS" | "BPGD" | "BPGF" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: 0,
        },

        // RockMan EXE 4.5 - Real Operation
        "BR4J" => CartridgeConfig {
            backup_type: BackupType::Flash64KB,
            device_pattern: Rtc as u8,
        },

        // Sennen Kazoku
        "BKAJ" => CartridgeConfig {
            backup_type: BackupType::Flash128KB,
            device_pattern: Rtc as u8,
        },

        // Shin Bokura no Taiyou - Gyakushuu no Sabata (Boktai 3)
        "U33J" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: Rtc | SolarSensor,
        },

        // Wario Ware Twisted!
        "RZWJ" | "RZWE" | "RZWP" => CartridgeConfig {
            backup_type: BackupType::Sram,
            device_pattern: Rumble | Gyro,
        },

        // Yoshi - Topsy-Turvy / Yoshi's Universal Gravitation
        "KYGJ" | "KYGE" | "KYGP" => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: Tilt as u8,
        },

        // FORCE_NONE — carts whose ROM contains stray bytes that false-positive the scan
        // Iridion II
        "AI2E" | "AI2P" => CartridgeConfig {
            backup_type: BackupType::None,
            device_pattern: 0,
        },

        // Stuart Little 2
        "ASLE" | "ASLF" => CartridgeConfig {
            backup_type: BackupType::None,
            device_pattern: 0,
        },

        // Top Gun - Combat Zones
        "A2YE" => CartridgeConfig {
            backup_type: BackupType::None,
            device_pattern: 0,
        },

        // Classic NES Series — covers all F-prefix titles (~30 games)
        _ if game_code.starts_with('F') => CartridgeConfig {
            backup_type: BackupType::Eeprom,
            device_pattern: 0,
        },

        _ => return None,
    };
    Some(config)
}
