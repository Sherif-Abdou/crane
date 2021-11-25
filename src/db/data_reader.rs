use std::{cell::RefCell, rc::Rc};

use crate::cfs::{Buffer, CranePartition, CraneSchema, DataValue, Reader};

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

            let start_sector =  position.offset / 256;
            let start_offset = position.offset % 256;

            let mut buf = Buffer::new(value.borrow_mut().read_sectors(start_sector, start_sector+1).unwrap());

            buf.consume(start_offset);
            return Some(self.schema.parse_bytes(&mut buf));
        }

        None
    }
}