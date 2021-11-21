use std::{cell::RefCell, rc::Rc};

use crate::cfs::{CranePartition, CraneSchema, DataValue, Writer};

use super::item_tree::ItemTree;


type Partition = Rc<RefCell<CranePartition>>;

pub struct DataWriter {
    pub partitions: Vec<Partition>,
    pub tree: ItemTree,
    schema: CraneSchema
}

impl DataWriter {
    pub fn new(partitions: Vec<Partition>, schema: CraneSchema, tree: ItemTree) -> Self {
        DataWriter {
            partitions,
            schema,
            tree
        }
    }

    pub fn write_value(&mut self, values: Vec<DataValue>) {
        let mut i: usize = 0;
        while (self.partitions[i as usize].borrow().total_len() - self.partitions[i as usize].borrow().initialized_len) < self.schema.len() {
            i += 1;
        }

        let off = self.partitions[i].borrow().initialized_len;
        self.tree.insert(self.tree.max_key()+1,         self.partitions[i].borrow().id(), off);
        self.partitions[i].borrow_mut().write_sectors(0, off, &self.schema.produce_bytes(&values))
            .unwrap();
    }

    pub fn save_tree(&self, partition: &mut CranePartition) {
        self.tree.to_partition(partition);
    }
}