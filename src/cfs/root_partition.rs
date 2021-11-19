use std::convert::TryInto;

use crate::{Crane, cfs::{buffer::Buffer, reader::Reader}};

use super::{crane_partition::CranePartition, writer::Writer};

pub struct RootPartition {
    partition: CranePartition,
    pub partition_starts: Vec<u64>,
}

impl RootPartition {
    pub fn import_from(partition: CranePartition) -> Self {
        let mut root = RootPartition {
            partition,
            partition_starts: vec![]
        };

        root.read();

        root
    }

    pub fn new(partition: CranePartition) -> Self {
        RootPartition {
            partition,
            partition_starts: vec![]
        }
    }

    pub fn read(&mut self) {
        let mut new_vec: Vec<u64> = vec![];
        let mut bytes = Buffer::new(self.partition.read_sectors(0, 4).unwrap());

        while !bytes.empty() {
            let mut b = bytes.consume(8);
            b.reverse();
            let rnumber = b.as_slice();

            let bytes = rnumber.try_into().unwrap();

            let value = u64::from_be_bytes(bytes);

            new_vec.push(value);
        }

        new_vec.reverse();
        self.partition_starts = new_vec;
    }

    pub fn write(&mut self) {
        for i in 0..(self.partition_starts.len()) {
            let bytes = self.partition_starts[i].to_be_bytes();

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
        let mut file = Rc::new(RefCell::new(File::create("./test/partitions/write.db").unwrap()));

        let mut partition = CranePartition::new(1, 0, 24, 0, Rc::downgrade(&file));

        let mut root_partition = RootPartition::new(partition);

        root_partition.partition_starts.append(&mut vec![20, 120, 282]);

        root_partition.write();
    }

    #[test]
    fn test_partition_read() {
        let mut file = Rc::new(RefCell::new(File::open("./test/partitions/read.db").unwrap()));

        let mut partition = CranePartition::new(1, 0, 24, 0, Rc::downgrade(&file));

        let mut root_partition = RootPartition::import_from(partition);

        assert_eq!(root_partition.partition_starts.len(), 3);
        assert_eq!(root_partition.partition_starts, vec![20, 120, 282]);
    }
}