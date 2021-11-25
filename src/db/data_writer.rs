use std::{cell::RefCell, rc::Rc};

use crate::{SECTOR_LENGTH, cfs::{CranePartition, CraneSchema, DataValue, Writer}};

use super::item_tree::ItemTree;


type Partition = Rc<RefCell<CranePartition>>;

pub struct DataWriter {
    pub partitions: Vec<Partition>,
    pub tree: Rc<RefCell<ItemTree>>,
    schema: CraneSchema
}

impl DataWriter {
    pub fn new(partitions: Vec<Partition>, schema: CraneSchema, tree: Rc<RefCell<ItemTree>>) -> Self {
        DataWriter {
            partitions,
            schema,
            tree
        }
    }

    pub fn write_value(&mut self, values: Vec<DataValue>) {
        let mut i: usize = 0;
        while (self.partitions[i as usize].borrow().total_len() - self.partitions[i as usize].borrow().initialized_len)*(SECTOR_LENGTH as u64) < self.schema.len() {
            i += 1;
        }

        let off = self.partitions[i].borrow().initialized_len;
        let m = self.tree.borrow().max_key();
        self.tree.borrow_mut().insert(m+1,         self.partitions[i].borrow().id(), off);
        self.partitions[i].borrow_mut().write_sectors(0, off, &self.schema.produce_bytes(&values))
            .unwrap();
    }

    pub fn save_tree(&self, partition: &mut CranePartition) {
        // dbg!(self.tree.borrow().to_bytes());
        self.tree.borrow_mut().to_partition(partition);
    }
}


#[cfg(test)]
mod test {
    use std::{collections::HashSet, fs::{File, OpenOptions}};

    use crate::cfs::CraneDisk;

    use super::*;

    fn generate_disk() -> CraneDisk {
        let write = File::create("test/data/write.db").unwrap();
        let read = File::open("test/data/write.db").unwrap();

        let crane = CraneDisk::init_file(read, write);

        crane
    }

    #[test]
    fn test_write() {
        let mut disk = generate_disk();

        let tree_id = disk.append_partition(2);
        let data_id = disk.append_partition(8);
        assert!(tree_id != data_id);
        let tree = disk.get_partition_with_id(tree_id);

        let tree_partition = ItemTree::from_partition(&mut *tree.borrow_mut(), Some(0));
        let r = Rc::new(RefCell::new(tree_partition));

        let schema = CraneSchema::new(vec![
            DataValue::UInt64(0),
            DataValue::UInt64(0),
        ]);

        let mut writer = DataWriter::new(vec![
            disk.get_partition_with_id(data_id).clone(),
        ], schema.clone(), r.clone());

        writer.write_value(vec![
            DataValue::UInt64(5),
            DataValue::UInt64(6),
        ]);

        writer.save_tree(&mut *tree.borrow_mut());
        disk.save();
    }
}