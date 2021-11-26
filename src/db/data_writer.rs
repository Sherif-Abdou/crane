use std::{cell::RefCell, io::Error, rc::Rc};

use crate::{SECTOR_LENGTH, cfs::{CranePartition, CraneSchema, DataValue, FSError, Writer}};

use super::{DataError, item_tree::{ItemTree, Position}};


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

    pub fn write_value(&mut self, values: Vec<DataValue>) -> Result<(), DataError> {
        let (i, off) = self.get_position_for_new()?;
        let m = self.tree.borrow().max_key();
        self.tree.borrow_mut().insert(m+1,         self.partitions[i].borrow().id(), off);
        self.partitions[i].borrow_mut().write_sectors(0, off, &self.schema.produce_bytes(&values))
            .unwrap();
        return Ok(());
    }

    fn get_position_for_new(&mut self) -> Result<(usize, u64), DataError> {
        if let Some(res) = self.find_replace_slot() {
            dbg!("used replace slot");
            return Ok(res);
        }
        dbg!("used fresh slot");
        let i = self.find_fresh_slot()?;
        let off = self.partitions[i].borrow().initialized_len;
        Ok((i, off))
    }

    fn find_fresh_slot(&mut self) -> Result<usize, DataError> {
        let mut i: usize = 0;
        // dbg!(self.partitions[i as usize].borrow().total_len() , self.partitions[i as usize].borrow().initialized_len);
        while (self.partitions[i as usize].borrow().total_len()*(SECTOR_LENGTH as u64) - self.partitions[i as usize].borrow().initialized_len)*(SECTOR_LENGTH as u64) < self.schema.len() {
            i += 1;
            if i >= self.partitions.len() {
                return Err(DataError::OutOfStorage);
            }
        dbg!(self.partitions[i as usize].borrow().total_len() - self.partitions[i as usize].borrow().initialized_len);
        }
        Ok(i)
    }

    fn find_replace_slot(&mut self) -> Option<(usize, u64)> {
        let ids: Vec<_> = self.partitions.iter().map(|v| v.borrow().id()).collect();
        let tree = self.tree.borrow();
        let positions = tree.position_set();
        let jump = self.schema.len();
        for (i, id) in ids.iter().enumerate() {
            let mut curr_offset = 0u64;

            while curr_offset + self.schema.len() < self.partitions[i].borrow().total_bytes() {
                curr_offset += jump;

                let pos = Position::new(*id, curr_offset);

                if !positions.contains(&pos) {
                    return Some((i, curr_offset));
                }
            }
        }
        None
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

        let tree_id = disk.append_partition(2, 0);
        let data_id = disk.append_partition(8, 0);
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
        ]).unwrap();

        writer.save_tree(&mut *tree.borrow_mut());
        disk.save();
    }
}