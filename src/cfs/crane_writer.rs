use std::io::{Seek, SeekFrom, Write};
use std::{cell::RefCell, fs::File, rc::Weak};

use super::writer::{WriteError, Writer};

const SECTOR_LENGTH: u64 = 512;

pub struct CraneWriter {
    sector_length: u64,
    start_byte: u64,
    end_byte: u64,
    file: Weak<RefCell<File>>
}

impl CraneWriter {
    pub fn new(start: u64, end: u64, file: Weak<RefCell<File>>) -> Self {
        CraneWriter {
            sector_length: SECTOR_LENGTH,
            start_byte: start,
            end_byte: end,
            file
        }
    }
}

impl Writer for CraneWriter {
    fn sector_length(&self) -> u64 {
        self.sector_length
    }

    fn write_sectors(&mut self, start: u64, end: u64, bytes: &[u8]) -> Result<(), super::writer::WriteError> {
        let start_byte = start*self.sector_length;
        let end_byte = end*self.sector_length;
        assert_eq!(end_byte-start_byte, bytes.len() as u64);

        if let Some(filerc) = self.file.upgrade() {
            let mut f = filerc.borrow_mut();

            f.seek(SeekFrom::Start(start_byte)).unwrap();

            f.write(bytes).unwrap();
            return Ok(());
        }
        Err(WriteError {})
    }

    fn capacity(&self) -> u64 {
        (self.end_byte-self.start_byte)/self.sector_length
    }
}