use std::convert::TryInto;

use crate::cfs::{buffer::Buffer, reader::Reader};

use super::crane_partition::CranePartition;

pub struct RootPartition {
    partition: CranePartition,
    partition_starts: Vec<u64>,
}

impl RootPartition {
    pub fn import_from(partition: CranePartition) {
        
    }

    pub fn read(&mut self) {
        let mut new_vec: Vec<u64> = vec![];
        let mut bytes = Buffer::new(self.partition.read_sectors(0, 4).unwrap());

        while !bytes.empty() {
            let b = bytes.consume(8);
            let rnumber = b.as_slice();

            let value = u64::from_be_bytes(rnumber.try_into().unwrap());

            new_vec.push(value);
        }
    }

    pub fn write(&mut self) {
    }
}