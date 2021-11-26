use std::{cell::RefCell, cmp::max, fs::File, rc::Weak};

use crate::SECTOR_LENGTH;

use super::{FSError, crane_reader::CraneReader, crane_writer::CraneWriter, reader::Reader, writer::Writer};

pub struct CranePartition {
    id: u64,
    offset: u64,
    total_len: u64,
    pub partition_type: u64,
    pub initialized_len: u64,
    writer: Box<dyn Writer>,
    reader: Box<dyn Reader>
}

impl CranePartition {
    //TODO: Create a builder for this, too many constructor arguments
    /// Creates a partition while assigning a type.
    /// # Arguments
    /// * `id` - The partition id.
    /// * `offset` - How offset the partition is from the start of the file in sectors.
    /// * `total_len` - How many sectors the partition is.
    /// * `initialized_len` - How many sectors in the partition that have been initialized.
    /// * `partition_type` - The type of the partition.
    /// * `rfile` - The file to read from.
    /// * `wfile` - The file to write to.
    pub fn with_type(id: u64, offset: u64, total_len: u64, initialized_len: u64, partition_type: u64, rfile: Weak<RefCell<File>>, wfile: Weak<RefCell<File>>) 
        -> Self {
        let s = offset;
        let e = total_len + offset;
        let reader = CraneReader::new(s, e, rfile.clone());
        let writer = CraneWriter::new(s, e, wfile.clone());
        CranePartition {
            id,
            offset,
            total_len,
            initialized_len,
            partition_type,
            reader: Box::new(reader),
            writer: Box::new(writer),
        }

    }

    /// Creates a partition without assigning a type.
    /// # Arguments
    /// * `id` - The partition id.
    /// * `offset` - How offset the partition is from the start of the file in sectors.
    /// * `total_len` - How many sectors the partition is.
    /// * `initialized_len` - How many sectors in the partition that have been initialized.
    /// * `rfile` - The file to read from.
    /// * `wfile` - The file to write to.
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
            partition_type: 0,
            reader: Box::new(reader),
            writer: Box::new(writer),
        }
    }

    /// Returns the offset of the partition.
    pub fn offset(&self) -> u64 {
        self.offset
    }

    // Returns the total length of the partition in sectors.
    pub fn total_len(&self) -> u64 {
        self.total_len
    }

    /// Returns the total length of the partition in bytes.
    pub fn total_bytes(&self) -> u64 {
        self.total_len * (SECTOR_LENGTH as u64)
    }

    /// Returns the id of the partition.
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
        self.initialized_len = max(start*(SECTOR_LENGTH as u64) + offset + (bytes.len() as u64), self.initialized_len);
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