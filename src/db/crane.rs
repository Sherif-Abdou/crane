use std::{cell::RefCell, process::Command, rc::Rc, vec};

use crate::cfs::{CraneDisk, CranePartition, CraneSchema};

use super::{DataError, data_command::DataCommand, data_manager::{DataManager, OFFSET}};

type Partition = Rc<RefCell<CranePartition>>;

pub struct Crane {
    disk: CraneDisk,
    managers: Vec<DataManager>
}

impl Crane {
    pub fn from_disk(disk: CraneDisk) -> Self {
        let mut res = Self::new(disk);
        Self::generate_schemas(&mut res);

        res
    }

    pub fn add_schema(&mut self, schema: CraneSchema) -> u64 {
        let slot = self.schema_count();
        self.managers.push(
            DataManager::create_to_disk(&mut self.disk, slot, schema)
        );

        self.save();

        slot
    }

    pub fn save(&mut self) {
        for manager in &mut self.managers {
            manager.save();
        }
        self.disk.save();
    }

    pub fn schema_count(&self) -> u64 {
        Self::count_schemas(&self.disk.partitions)
    }

    pub fn new(disk: CraneDisk) -> Self {
        Self {
            disk,
            managers: vec![],
        }
    }

    fn count_schemas(partitions: &Vec<Partition>) -> u64 {
        let max_type: u64 = partitions.iter().map(|v| v.borrow().partition_type).max().unwrap_or(0);

        (max_type-OFFSET)/3
    }

    pub fn execute(&mut self, schema_slot: u64, command: &mut dyn DataCommand) -> Result<(), DataError> {
        let res = self.managers[schema_slot as usize].execute(command);
        match res {
            Ok(()) => {
                self.save();
                Ok(())
            },
            Err(DataError::OutOfStorage) => {
                let data_type = schema_slot*3 + 3;
                self.disk.append_partition(16, data_type);
                Self::generate_schemas(self);
                self.execute_no_recur(schema_slot, command)
            }
        }
    }

    fn execute_no_recur(&mut self, schema_slot: u64, command: &mut dyn DataCommand) -> Result<(), DataError> {
        let res = self.managers[schema_slot as usize].execute(command);
        if res.is_ok() {
            self.save();
        }
        res
    }

    fn generate_schemas(res: &mut Crane) {
        let schemas = Self::count_schemas(&res.disk.partitions);
        for i in 0..schemas {
            res.managers.push(
                DataManager::from_disk(&res.disk, i)
            );
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::{File, OpenOptions};

    use crate::{cfs::{CraneDisk, DataValue}, db::data_command::{GetKeyCommand, InsertValueCommand}};

    use super::*;

    fn generate_disk() -> CraneDisk {
        let write = File::create("test/crane/db.cdb").unwrap();
        let read = File::open("test/crane/db.cdb").unwrap();

        let crane = CraneDisk::init_file(read, write);

        crane
    }

    fn load_disk() -> CraneDisk {
        let read = File::open("test/crane/db.cdb").unwrap();
        let write = OpenOptions::new().write(true).open("test/crane/db.cdb").unwrap();

        let crane = CraneDisk::from_file(read, write);

        crane
    }

    fn gen_schema() -> CraneSchema {
        let schema = CraneSchema::new(vec![
            DataValue::UInt64(0),
            DataValue::Int16(0),
            DataValue::Fixchar("".to_owned(), 64),
        ]);

        schema
    }

    #[test]
    fn test_create_crane() {
        let disk = generate_disk();

        let mut crane = Crane::new(disk);

        let slot = crane.add_schema(gen_schema());

        let mut command = InsertValueCommand::new(vec![
            DataValue::UInt64(21),
            DataValue::Int16(-5),
            DataValue::Fixchar("Hello world".to_owned(), 64),
        ]);

        crane.execute(slot, &mut command).unwrap();
        crane.execute(slot, &mut command).unwrap();
        crane.execute(slot, &mut command).unwrap();
        crane.execute(slot, &mut command).unwrap();

        crane.save();
    }

    #[test]
    fn test_load_crane() {
        test_create_crane();
        let disk = load_disk();

        let mut crane = Crane::from_disk(disk);

        let mut command = GetKeyCommand::new(1);

        crane.execute(0, &mut command).unwrap();

        let res = command.get_result();

        assert_ne!(res, None);
    }
}