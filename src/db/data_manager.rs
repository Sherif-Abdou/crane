use std::any::Any;
use std::rc::{Rc};
use std::cell::{RefCell};

use crate::cfs::{CranePartition, CraneSchema};

use super::data_reader::DataReader;
use super::data_writer::{self, DataWriter};
use super::item_tree::ItemTree;


type Partition = Rc<RefCell<CranePartition>>;

const OFFSET: u64 = 1;

struct DataManager {
    schema: CraneSchema,
    data_partitions: Vec<Partition>,
    tree_partition: Partition,
    data_writer: DataWriter,
    data_reader: DataReader,
}

impl DataManager {
    pub fn new(schema: CraneSchema, data_partitions: Vec<Partition>, tree_partition: Partition) -> Self {
        let tree = Rc::new(RefCell::new(ItemTree::from_partition(&mut *tree_partition.borrow_mut(), Some(OFFSET))));
        let data_writer = DataWriter::new(data_partitions.clone(), schema.clone(), tree.clone());
        let data_reader = DataReader::new(data_partitions.clone(), schema.clone(), tree.clone());
        Self {
            schema,
            data_partitions,
            tree_partition,
            data_writer,
            data_reader
        }
    }

    pub fn save_schema(&mut self) {
        let ids = self.schema.types.iter().map(|t| t.id()).collect::<Vec<u16>>();
        let mut vals = ids.iter().map(|id| id.to_be_bytes().to_vec()).flatten().collect::<Vec<u8>>();

    }

    pub fn get_schema(&self) -> &CraneSchema {
        &self.schema
    }

    pub fn get_data_partitions(&self) -> &Vec<Partition> {
        &self.data_partitions
    }

    pub fn get_tree_partition(&self) -> &Partition {
        &self.tree_partition
    }
}