use std::{cell::RefCell, fs::File, io::{Read, Seek, SeekFrom}, rc::Weak, slice};

use super::reader::{ReadError, Reader};

const SECTOR_LENGTH: u64 = 256;

pub struct CraneReader {
    sector_length: u64,
    start_byte: u64,
    end_byte: u64,
    file: Weak<RefCell<File>>
}

impl CraneReader {
    pub fn new(start: u64, end: u64, file: Weak<RefCell<File>>) -> Self {
        CraneReader {
            sector_length: SECTOR_LENGTH,
            start_byte: start,
            end_byte: end,
            file
        }
    }
}

impl Reader for CraneReader {
    fn sector_length(&self) -> u64 {
        self.sector_length
    }

    fn read_sectors(&mut self, start: u64, end: u64) -> Result<Vec<u8>, super::reader::ReadError> {
        let start_byte = start*self.sector_length;
        let end_byte = end*self.sector_length;

        let len = end_byte - start_byte;

        if let Some(raw_file) = self.file.upgrade() {
            let mut f = raw_file.borrow_mut();

            let mut buffer = vec![0u8; len as usize];

            f.seek(SeekFrom::Start(start_byte)).unwrap();

            f.read_exact(buffer.as_mut_slice()).unwrap();

            return Ok(buffer);
        }

        Err(ReadError {})
    }

    fn capacity(&self) -> u64 {
        (self.end_byte-self.start_byte)/self.sector_length
    }
}