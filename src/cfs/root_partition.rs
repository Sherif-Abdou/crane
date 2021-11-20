use std::convert::TryInto;
use lazy_static::lazy_static;

use crate::{Crane, cfs::{buffer::Buffer, reader::Reader, schema::DataValue}};

use super::{crane_partition::CranePartition, schema::CraneSchema, writer::Writer};

lazy_static! {
    static ref PARTITION_SCHEMA: CraneSchema = CraneSchema::new(vec![
        DataValue::UInt64(0),
        DataValue::UInt64(0),
    ]);
}


pub struct RootPartition {
    partition: CranePartition,
    pub partition_starts: Vec<u64>,
    pub partition_ends: Vec<u64>,
}

impl RootPartition {
    pub fn import_from(partition: CranePartition) -> Self {
        let mut root = RootPartition {
            partition,
            partition_starts: vec![],
            partition_ends: vec![],
        };

        root.read();

        root
    }

    pub fn new(partition: CranePartition) -> Self {
        RootPartition {
            partition,
            partition_starts: vec![],
            partition_ends: vec![],
        }
    }

    pub fn read(&mut self) {
        let mut new_starts: Vec<u64> = vec![];
        let mut new_ends: Vec<u64> = vec![];
        let mut bytes = Buffer::new(self.partition.read_sectors(0, 4).unwrap());

        let mut i: u64  = 0;

        while !bytes.empty() {
            let mut b = bytes.consume(8);
            b.reverse();
            let rnumber = b.as_slice();

            let bytes = rnumber.try_into().unwrap();

            let value = u64::from_be_bytes(bytes);

            match i%2 {
                0 => new_ends.push(value),
                1 => new_starts.push(value),
                _ => panic!()
            };
            
            i += 1;
        }

        new_starts.reverse();
        self.partition_starts = new_starts;
    }

    pub fn write(&mut self) {
        for i in 0..(self.partition_starts.len()*2) {
            let bytes = match i%2 {
                0 => self.partition_starts[i/2].to_be_bytes(),
                1 => self.partition_ends[i/2].to_be_bytes(),
                _ => panic!("mod 2 produced number besides 1 or 0")
            };

            self.partition.write_sectors(0, 8*(i as u64), &bytes).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, fs::File, rc::Rc};

    use super::*;

    #[test]
    fn test_partition_write() {
        let file = Rc::new(RefCell::new(File::create("./test/partitions/write.db").unwrap()));

        let partition = CranePartition::new(1, 0, 24, 0, Rc::downgrade(&file));

        let mut root_partition = RootPartition::new(partition);

        root_partition.partition_starts.append(&mut vec![20, 120, 282]);
        root_partition.partition_ends.append(&mut vec![119, 281, 300]);

        root_partition.write();
    }

    #[test]
    fn test_partition_read() {
        let file = Rc::new(RefCell::new(File::open("./test/partitions/read.db").unwrap()));

        let partition = CranePartition::new(1, 0, 24, 0, Rc::downgrade(&file));

        let root_partition = RootPartition::import_from(partition);

        assert_eq!(root_partition.partition_starts.len(), 3);
        assert_eq!(root_partition.partition_starts, vec![20, 120, 282]);
    }
}