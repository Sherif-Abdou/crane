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
        let bytes = Buffer::new(self.partition.read_sectors(0, 4).unwrap());

        while !bytes.empty() {
            let rnumber = bytes.consume(8);

            // new_vec.push(rnumber.try_into().unwrap());
        }
    }

    pub fn write(&mut self) {
    }
}