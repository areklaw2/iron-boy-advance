use std::{fs, io, path::PathBuf};

use etcetera::{BaseStrategy, choose_base_strategy};
use ironboyadvance::System;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const CONFIG_FILE_NAME: &str = "ironboyadvance.toml";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("could not resolve platform config directory: {0}")]
    BaseStrategy(#[from] etcetera::HomeDirError),
    #[error("failed to parse config: {0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bios_path: Option<String>,
    pub gb_boot_rom: Option<String>,
    pub gbc_boot_rom: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let path = config_path()?;
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.into()),
        };
        Ok(toml::from_str(&contents)?)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, toml::to_string(self)?)?;
        Ok(())
    }

    pub fn set_bios(&mut self, kind: System, path: &str) {
        let slot = match kind {
            System::Gba => &mut self.bios_path,
            System::Gb => &mut self.gb_boot_rom,
            System::Gbc => &mut self.gbc_boot_rom,
        };
        *slot = Some(path.to_string());
    }

    pub fn bios(&self, kind: System) -> Option<&str> {
        match kind {
            System::Gba => self.bios_path.as_deref(),
            System::Gb => self.gb_boot_rom.as_deref(),
            System::Gbc => self.gbc_boot_rom.as_deref(),
        }
    }
}

fn config_path() -> Result<PathBuf, ConfigError> {
    let strategy = choose_base_strategy()?;
    Ok(strategy.config_dir().join("ironboyadvance").join(CONFIG_FILE_NAME))
}
