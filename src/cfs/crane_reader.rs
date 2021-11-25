use std::{cell::RefCell, fs::File, io::{Read, Seek, SeekFrom}, rc::Weak, slice};

use super::{FSError, reader::{Reader}};

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

    fn read_sectors(&mut self, start: u64, end: u64) -> Result<Vec<u8>, FSError> {
        let start_byte = start*self.sector_length;
        let end_byte = end*self.sector_length;

        let len = end_byte - start_byte;

        if let Some(raw_file) = self.file.upgrade() {
            let mut f = (&raw_file).borrow_mut(); 
                
            f.seek(SeekFrom::Start(start_byte)).expect("Unable to shift file");

            let buffer = (&*f).bytes().take(len as usize).map(|v| v.expect("Unable to read byte in file")).collect();

            return Ok(buffer);
        }

        Err(FSError {})
    }


    fn capacity(&self) -> u64 {
        (self.end_byte-self.start_byte)/self.sector_length
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;
    use std::{fs::OpenOptions, path::PathBuf, rc::Rc};
    use super::Reader;

    use super::*;

    pub fn get_db_file() -> File {
        let path = PathBuf::from("./test/read.db");

        match &path.exists() {
            true => OpenOptions::new().read(true).open(path).unwrap(),
            false => File::create(path).unwrap(),
        }
    }

    #[test]
    pub fn test_writer() {
        let file = Rc::new(RefCell::new(get_db_file()));

        let mut reader = CraneReader::new(0, 16, Rc::downgrade(&file));

        let data = reader.read_sectors(0, 1).unwrap();


        let number = u64::from_be_bytes(data[..].try_into().unwrap());

        assert_eq!(number, 2048);
    }
}