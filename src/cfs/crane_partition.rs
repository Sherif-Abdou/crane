use std::cmp::max;

use super::{reader::Reader, writer::Writer};

pub struct CranePartition {
    id: u64,
    offset: u64,
    total_len: u64,
    initialized_len: u64,
    writer: Box<dyn Writer>,
    reader: Box<dyn Reader>
}

impl CranePartition {

}

impl Writer for CranePartition {
    fn sector_length(&self) -> u64 {
        self.writer.sector_length()
    }

    fn write_sectors(&mut self, start: u64, end: u64, bytes: &[u8]) -> Result<(), super::writer::WriteError> {
        let s = start + self.offset;
        let e = end + self.offset;
        self.initialized_len = max(e, self.initialized_len);
        self.writer.write_sectors(start + self.offset, end + self.offset, bytes)
    }

    fn capacity(&self) -> u64 {
        self.writer.capacity()
    }
}