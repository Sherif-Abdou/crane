use std::{convert::TryInto, vec};
use lazy_static::lazy_static;

use crate::{cfs::{buffer::Buffer, reader::Reader, schema::DataValue}};

use super::{crane_partition::CranePartition, schema::CraneSchema, writer::Writer};

lazy_static! {
    static ref PARTITION_SCHEMA: CraneSchema = CraneSchema::new(vec![
        DataValue::UInt64(0),
        DataValue::UInt64(0),
        DataValue::UInt64(0),
        DataValue::UInt64(0),
    ]);
}


pub struct RootPartition {
    partition: CranePartition,
    pub partition_starts: Vec<u64>,
    pub partition_ends: Vec<u64>,
    pub init_lens: Vec<u64>,
    pub partition_types: Vec<u64>,
}

impl RootPartition {
    pub fn import_from(partition: CranePartition) -> Self {
        let mut root = RootPartition {
            partition,
            partition_starts: vec![],
            partition_ends: vec![],
            init_lens: vec![],
            partition_types: vec![],
        };

        root.read();

        root
    }

    pub fn new(partition: CranePartition) -> Self {
        RootPartition {
            partition,
            partition_starts: vec![],
            partition_ends: vec![],
            init_lens: vec![],
            partition_types: vec![],
        }
    }

    pub fn compute_lens(&self) -> Vec<u64> {
        self.partition_starts.iter().zip(self.partition_ends.iter()).map(|(s, e)| *e-*s).collect()
    }

    pub fn read(&mut self) {
        let mut new_starts: Vec<u64> = vec![];
        let mut new_ends: Vec<u64> = vec![];
        let mut init_lens: Vec<u64> = vec![];
        let mut partition_types: Vec<u64> = vec![];
        let mut bytes = Buffer::new(self.partition.read_sectors(0, 12).expect("Couldn't read partition map"));

        while !bytes.empty() {
            let values = PARTITION_SCHEMA.parse_bytes(&mut bytes);
            
            if let (DataValue::UInt64(start), DataValue::UInt64(end), DataValue::UInt64(init_len), DataValue::UInt64(p_type)) 
                = (&values[0], &values[1], &values[2], &values[3]) {
                if *start == 0 && *end == 0 {
                    break;
                }
                new_starts.push(*start);
                new_ends.push(*end);
                init_lens.push(*init_len);
                partition_types.push(*p_type);
            }
       }

        self.partition_starts = new_starts;
        self.partition_ends = new_ends;
        self.init_lens = init_lens;
        self.partition_types = partition_types;
    }

    pub fn write(&mut self) {
        assert_eq!(self.partition_starts.len(), self.partition_ends.len());
        assert_eq!(self.partition_starts.len(), self.init_lens.len());
        let len = PARTITION_SCHEMA.len();

        for i in 0..(self.partition_starts.len()) {
            let bytes = PARTITION_SCHEMA.produce_bytes(&vec![
                DataValue::UInt64(self.partition_starts[i]),
                DataValue::UInt64(self.partition_ends[i]),
                DataValue::UInt64(self.init_lens[i]),
                DataValue::UInt64(0),
            ]);
            self.partition.write_sectors(0, len*(i as u64), &bytes).unwrap();
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

        let partition = CranePartition::new(1, 0, 24, 0, Rc::downgrade(&file), Rc::downgrade(&file));

        let mut root_partition = RootPartition::new(partition);

        root_partition.partition_starts.append(&mut vec![20, 120, 282]);
        root_partition.partition_ends.append(&mut vec![119, 281, 300]);
        root_partition.init_lens.append(&mut vec![40, 100, 18]);

        root_partition.write();
    }

    #[test]
    fn test_partition_read() {
        let file = Rc::new(RefCell::new(File::open("./test/partitions/read.db").unwrap()));

        let partition = CranePartition::new(1, 0, 12, 0, Rc::downgrade(&file), Rc::downgrade(&file));

        let root_partition = RootPartition::import_from(partition);

        assert_eq!(root_partition.partition_starts.len(), 3);
        assert_eq!(root_partition.partition_starts, vec![20, 120, 282]);
        assert_eq!(root_partition.partition_ends, vec![119, 281, 300]);
        assert_eq!(root_partition.init_lens, vec![40, 100, 18]);
    }
}