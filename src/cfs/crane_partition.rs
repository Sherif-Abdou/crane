use std::{cell::RefCell, cmp::max, fs::File, rc::Weak};

use super::{FSError, crane_reader::CraneReader, crane_writer::CraneWriter, reader::Reader, writer::Writer};

pub struct CranePartition {
    id: u64,
    offset: u64,
    total_len: u64,
    pub initialized_len: u64,
    writer: Box<dyn Writer>,
    reader: Box<dyn Reader>
}

impl CranePartition {
    pub fn new(id: u64, offset: u64, total_len: u64, initialized_len: u64, rfile: Weak<RefCell<File>>, wfile: Weak<RefCell<File>>) -> Self {
        let s = offset;
        let e = total_len + offset;
        let reader = CraneReader::new(s, e, rfile.clone());
        let writer = CraneWriter::new(s, e, wfile.clone());
        CranePartition {
            id,
            offset,
            total_len,
            initialized_len,
            reader: Box::new(reader),
            writer: Box::new(writer),
        }
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn total_len(&self) -> u64 {
        self.total_len
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl Writer for CranePartition {
    fn sector_length(&self) -> u64 {
        self.writer.sector_length()
    }

    fn write_sectors(&mut self, start: u64, offset: u64, bytes: &[u8]) -> Result<(), FSError> {
        let s = start + self.offset;
        let e = s + offset + (bytes.len() as u64);
        self.initialized_len = max(start*256 + offset + (bytes.len() as u64), self.initialized_len);
        self.writer.write_sectors(s,offset, bytes)
    }

    fn capacity(&self) -> u64 {
        self.writer.capacity()
    }
}

impl Reader for CranePartition {
    fn sector_length(&self) -> u64 {
        self.reader.sector_length()
    }

    fn read_sectors(&mut self, start: u64, end: u64) -> Result<Vec<u8>, FSError> {
        let s = start + self.offset;
        let e = end + self.offset;

        self.reader.read_sectors(s, e)
    }

    fn capacity(&self) -> u64 {
        self.reader.capacity()
    }
}