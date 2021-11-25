use std::{cell::RefCell, rc::Rc};

use crate::{SECTOR_LENGTH, cfs::{Buffer, CranePartition, CraneSchema, DataValue, Reader}};

use super::item_tree::ItemTree;


type Partition = Rc<RefCell<CranePartition>>;

pub struct DataReader {
    pub partitions: Vec<Partition>,
    pub tree: Rc<RefCell<ItemTree>>,
    schema: CraneSchema,
}

impl DataReader {
    pub fn new(partitions: Vec<Partition>, schema: CraneSchema, tree: Rc<RefCell<ItemTree>>) -> Self {
        Self {
            partitions,
            tree,
            schema,
        }
    }

    /// Retrieves the value at the key in whatever partition it is in.
    pub fn get_value(&self, key: u64) -> Option<Vec<DataValue>> {
        if let Some(position) = self.tree.borrow().get(key) {
            let value = self.partitions.iter().filter(|p| p.borrow().id() == position.partition).next().unwrap();
            
            let s = SECTOR_LENGTH as u64;

            let start_sector =  position.offset / s;
            let start_offset = position.offset % s;

            let mut buf = Buffer::new(value.borrow_mut().read_sectors(start_sector, start_sector+1).unwrap());

            buf.consume(start_offset);
            return Some(self.schema.parse_bytes(&mut buf));
        }

        None
    }
}

#[cfg(test)]
mod test {
    use std::fs::{File, OpenOptions};
    use super::*;

    use crate::cfs::CraneDisk;


    fn generate_disk() -> CraneDisk {
        let read = File::open("test/data/read.db").unwrap();
        let write = OpenOptions::new().write(true).open("test/data/read.db").unwrap();

        let crane = CraneDisk::from_file(read, write);

        crane
    }

    #[test]
    pub fn test_data_read() {
        let disk = generate_disk();
        
        let schema = CraneSchema::new(vec![
            DataValue::UInt64(0),
            DataValue::UInt64(0),
        ]);

        let partitions = vec![
            disk.get_partition_with_id(2).clone()
        ];

        let tree = ItemTree::from_partition(&mut *disk.get_partition_with_id(1).borrow_mut(), None);

        let reader = DataReader::new(partitions, schema, Rc::new(RefCell::new(tree)));

        let res = reader.get_value(1);

        assert_ne!(res, None, "Not equal");
        let val = res.unwrap();
        assert_eq!(val[0], DataValue::UInt64(5));
        assert_eq!(val[1], DataValue::UInt64(6));
    }
}