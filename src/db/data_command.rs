use std::{cell::RefCell, rc::Rc};

use crate::{SECTOR_LENGTH, cfs::{Buffer, CranePartition, CraneSchema, DataValue, Reader, Writer}};

use super::{DataError, item_tree::{ItemTree, Position}};

type Partition = Rc<RefCell<CranePartition>>;

pub struct DataState<'a> {
    pub data_partitions: Vec<&'a Partition>,
    pub schema: &'a CraneSchema,
    pub tree: &'a Rc<RefCell<ItemTree>>
}

pub trait DataCommand {
    fn execute(&mut self, state: &mut DataState) -> Result<(), DataError>;
}

pub struct GetKeyCommand {
    key: u64,
    res: Option<Vec<DataValue>>,
}

impl GetKeyCommand {
    pub fn new(key: u64) -> Self {
        Self {
            key,
            res: None
        }
    }

    pub fn get_result(&self) -> Option<Vec<DataValue>> {
        self.res.clone()
    }
}

impl DataCommand for GetKeyCommand {
    fn execute(&mut self, state: &mut DataState) -> Result<(), DataError> {
        let rdiff: f64 = (state.schema.len() as f64)/(SECTOR_LENGTH as f64);
        let diff = f64::ceil(rdiff) as u64;
        if let Some(position) = state.tree.borrow().get(self.key) {
            let value = state.data_partitions.iter().filter(|p| p.borrow().id() == position.partition).next().unwrap();
            
            let s = SECTOR_LENGTH as u64;

            let start_sector =  position.offset / s;
            let start_offset = position.offset % s;

            let mut buf = Buffer::new(value.borrow_mut().read_sectors(start_sector, start_sector+diff).unwrap());

            buf.consume(start_offset);
            self.res = Some(state.schema.parse_bytes(&mut buf));

            return Ok(());
        }

        Ok(())
    }
}

pub struct InsertValueCommand {
    value: Vec<DataValue>,
}

impl InsertValueCommand {
    pub fn new(value: Vec<DataValue>) -> Self {
        Self {
            value
        }
    }

    fn get_position_for_new(&self, state: &mut DataState) -> Result<(usize, u64), DataError> {
        if let Some(res) = self.find_replace_slot(state) {
            return Ok(res);
        }
        let i = self.find_fresh_slot(state)?;
        let off = state.data_partitions[i].borrow().initialized_len;
        Ok((i, off))
    }

    fn find_fresh_slot(&self, state: &mut DataState) -> Result<usize, DataError> {
        let mut i: usize = 0;
        // dbg!(self.partitions[i as usize].borrow().total_len() , self.partitions[i as usize].borrow().initialized_len);
        while (state.data_partitions[i as usize].borrow().total_len()*(SECTOR_LENGTH as u64) - state.data_partitions[i as usize].borrow().initialized_len)*(SECTOR_LENGTH as u64) < state.schema.len() {
            i += 1;
            if i >= state.data_partitions.len() {
                return Err(DataError::OutOfStorage);
            }
        }
        Ok(i)
    }

    fn find_replace_slot(&self, state: &mut DataState) -> Option<(usize, u64)> {
        let ids: Vec<_> = state.data_partitions.iter().map(|v| v.borrow().id()).collect();
        let tree = state.tree.borrow();
        let positions = tree.position_set();
        let jump = state.schema.len();
        for (i, id) in ids.iter().enumerate() {
            let mut curr_offset = 0u64;

            while curr_offset + state.schema.len() < state.data_partitions[i].borrow().total_bytes() {
                curr_offset += jump;

                let pos = Position::new(*id, curr_offset);

                if !positions.contains(&pos) {
                    return Some((i, curr_offset));
                }
            }
        }
        None
    }
}

impl DataCommand for InsertValueCommand {
    fn execute(&mut self, state: &mut DataState) -> Result<(), DataError> {
        let (i, off) = self.get_position_for_new(state)?;
        let m = state.tree.borrow().max_key();
        state.tree.borrow_mut().insert(m+1,         state.data_partitions[i].borrow().id(), off);
        state.data_partitions[i].borrow_mut().write_sectors(0, off, &state.schema.produce_bytes(&self.value))
            .unwrap();
        return Ok(());
    }
}

pub struct UpdateValueCommand {
    pub value: Vec<DataValue>,
    pub key: u64
}

impl UpdateValueCommand {
    pub fn new(key: u64, value: Vec<DataValue>) -> Self {
        Self {
            key,
            value,
        }
    }
}

impl DataCommand for UpdateValueCommand {
    fn execute(&mut self, state: &mut DataState) -> Result<(), DataError> {
        let pos = state.tree.borrow().get(self.key).ok_or(DataError::UnknownKey)?;
        let off = pos.offset;
        let p = state.data_partitions.iter()
            .filter(|v| v.borrow().id() == pos.partition)
            .next()
            .ok_or(DataError::UnknownKey)?;
        let m = state.tree.borrow().max_key();
        state.tree.borrow_mut().insert(m+1,         p.borrow().id(), off);
        p.borrow_mut().write_sectors(0, off, &state.schema.produce_bytes(&self.value))
            .unwrap();
        return Ok(());
    }
}

pub struct RemoveValueCommand {
    pub key: u64,
}

impl RemoveValueCommand {
    pub fn new(key: u64) -> Self {
        Self {
            key,
        }
    }
}

impl DataCommand for RemoveValueCommand {
    fn execute(&mut self, state: &mut DataState) -> Result<(), DataError> {
        state.tree.borrow_mut().remove(self.key);
        Ok(())
    }
}