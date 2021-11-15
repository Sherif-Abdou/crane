use std::io::{Seek, SeekFrom, Write};
use std::{cell::RefCell, fs::File, rc::Weak};

use super::FSError;
use super::writer::{Writer};

const SECTOR_LENGTH: u64 = 256;

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

    fn write_sectors(&mut self, start: u64, offset: u64, bytes: &[u8]) -> Result<(), FSError> {
        let start_byte = start*self.sector_length;

        if let Some(filerc) = self.file.upgrade() {
            let mut f = filerc.borrow_mut();

            f.seek(SeekFrom::Start(start_byte + offset)).unwrap();

            f.write(bytes).unwrap();
            return Ok(());
        }
        Err(FSError {})
    }

    fn capacity(&self) -> u64 {
        (self.end_byte-self.start_byte)/self.sector_length
    }
}


#[cfg(test)]
mod test {
    use std::{fs::OpenOptions, path::PathBuf, rc::Rc};
    use super::Writer;

    use super::*;

    pub fn get_db_file() -> File {
        let path = PathBuf::from("./test/write.db");

        match &path.exists() {
            true => OpenOptions::new().write(true).open(path).unwrap(),
            false => File::create(path).unwrap(),
        }
    }

    #[test]
    pub fn test_writer() {
        let file = Rc::new(RefCell::new(get_db_file()));

        let mut writer = CraneWriter::new(0, 16, Rc::downgrade(&file));

        let bytes = (24u64).to_be_bytes();

        writer.write_sectors(0, 0, &bytes).unwrap();
    }
}