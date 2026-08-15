use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use tracing::warn;

use crate::cartridge::{CartridgeError, rtc::RTC_SAVE_BYTES};

pub struct BackupFile {
    buffer: Vec<u8>,
    file: File,
}

impl BackupFile {
    pub fn open(path: &Path, size: usize, fill: u8) -> Result<Self, CartridgeError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let buffer = match file.metadata()?.len() as usize {
            0 => {
                let buffer = vec![fill; size];
                file.write_all(&buffer)?;
                buffer
            }
            len if len == size || len == size + RTC_SAVE_BYTES => {
                let mut buffer = vec![0; size];
                file.read_exact(&mut buffer)?;
                buffer
            }
            _ => return Err(CartridgeError::SaveSizeMismatch),
        };

        Ok(Self { buffer, file })
    }

    pub fn read(&self, offset: usize) -> u8 {
        self.buffer[offset]
    }

    pub fn write(&mut self, offset: usize, value: u8) {
        self.buffer[offset] = value;
        if let Err(e) = self
            .file
            .seek(SeekFrom::Start(offset as u64))
            .and_then(|_| self.file.write_all(&[value]))
        {
            warn!("backup write failed at offset {:08X}: {}", offset, e);
        }
    }

    pub fn fill(&mut self, offset: usize, length: usize, value: u8) {
        let range = offset..offset + length;
        self.buffer[range.clone()].fill(value);
        let result = self
            .file
            .seek(SeekFrom::Start(offset as u64))
            .and_then(|_| self.file.write_all(&self.buffer[range]));
        if let Err(e) = result {
            warn!("backup fill failed at offset {:08X}: {}", offset, e);
        }
    }
}
