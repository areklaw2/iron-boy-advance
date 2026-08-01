use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use tracing::warn;

use crate::cartridge::CartridgeError;

pub struct BackupFile {
    buffer: Vec<u8>,
    file: Option<File>,
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
            length if length == size => {
                let mut buffer = vec![0; size];
                file.read_exact(&mut buffer)?;
                buffer
            }
            _ => return Err(CartridgeError::SaveSizeMismatch),
        };

        Ok(Self {
            buffer,
            file: Some(file),
        })
    }

    pub fn memory(size: usize, fill: u8) -> Self {
        Self {
            buffer: vec![fill; size],
            file: None,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn read(&self, offset: usize) -> u8 {
        self.buffer[offset]
    }

    pub fn write(&mut self, offset: usize, value: u8) {
        self.buffer[offset] = value;

        let Some(file) = self.file.as_mut() else {
            return;
        };

        if let Err(error) = file
            .seek(SeekFrom::Start(offset as u64))
            .and_then(|_| file.write_all(&[value]))
        {
            warn!("backup write failed at offset {:#06X}: {}", offset, error);
        }
    }
}
