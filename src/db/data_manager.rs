use std::convert::TryInto;
use std::rc::{Rc};
use std::cell::{RefCell};

use crate::cfs::{Buffer, CraneDisk, CranePartition, CraneSchema, DataValue, Reader, Writer};

use super::DataError;
use super::data_command::{DataCommand, DataState};
use super::data_reader::DataReader;
use super::data_writer::{self, DataWriter};
use super::item_tree::ItemTree;


type Partition = Rc<RefCell<CranePartition>>;
struct DataManager {
    schema: CraneSchema,
    data_partitions: Vec<Partition>,
    tree_partition: Partition,
    schema_partition: Partition,
    tree: Rc<RefCell<ItemTree>>,
    pub data_writer: DataWriter,
    pub data_reader: DataReader,
}

impl DataManager {
    pub fn new(schema: CraneSchema, data_partitions: Vec<Partition>, schema_partition: Partition, tree_partition: Partition) -> Self {
        let tree = Rc::new(RefCell::new(ItemTree::from_partition(&mut *tree_partition.borrow_mut(), None)));
        let data_writer = DataWriter::new(data_partitions.clone(), schema.clone(), tree.clone());
        let data_reader = DataReader::new(data_partitions.clone(), schema.clone(), tree.clone());
        Self {
            schema,
            data_partitions,
            tree_partition,
            data_writer,
            tree,
            data_reader,
            schema_partition,
        }
    }

    pub fn create_to_disk(disk: &mut CraneDisk, schema_slot: u64, schema: CraneSchema) -> Self {
        let schema_type = schema_slot*3 + 1;
        let tree_type = schema_slot*3 + 2;
        let data_type = schema_slot*3 + 3;

        let schema_id = disk.append_partition(2, schema_type);
        let tree_id = disk.append_partition(8, tree_type);
        let data_id = disk.append_partition(16, data_type);


        // Code here is almost entirely copy and pasted from `from_disk`
        //TODO: Make this code segment not copy and pasted and dry

        let spartitions = disk.get_partition_by_type(schema_type);
        let tpartitions = disk.get_partition_by_type(tree_type);
        let dpartitions = disk.get_partition_by_type(data_type);

        let schema_partition = spartitions.get(0).expect("Missing schema partition");
        let tree_partition = tpartitions.get(0).expect("Missing btree partition");
        let data_partitions: Vec<Partition> = dpartitions.iter()
            .map(|v| (*v).clone())
            .collect();

        let tree = Rc::new(RefCell::new(ItemTree::from_partition(&mut *tree_partition.borrow_mut(), None)));

        let data_writer = DataWriter::new(data_partitions.clone(), schema.clone(), tree.clone());
        let data_reader = DataReader::new(data_partitions.clone(), schema.clone(), tree.clone());

        Self {
            schema,
            data_partitions,
            schema_partition: (*schema_partition).clone(),
            tree_partition: (*tree_partition).clone(),
            data_writer,
            tree,
            data_reader,
        }
    }

    pub fn from_disk(disk: &CraneDisk, schema_slot: u64) -> Self {
        let schema_type = schema_slot*3 + 1;
        let tree_type = schema_slot*3 + 2;
        let data_type = schema_slot*3 + 3;
        
        let spartitions = disk.get_partition_by_type(schema_type);
        let tpartitions = disk.get_partition_by_type(tree_type);
        let dpartitions = disk.get_partition_by_type(data_type);

        let schema_partition = spartitions.get(0).expect("Missing schema partition");
        let tree_partition = tpartitions.get(0).expect("Missing btree partition");
        let data_partitions: Vec<Partition> = dpartitions.iter()
            .map(|v| (*v).clone())
            .collect();
        let schema = Self::load_schema(*schema_partition);
        let tree = Rc::new(RefCell::new(ItemTree::from_partition(&mut *tree_partition.borrow_mut(), None)));

        let data_writer = DataWriter::new(data_partitions.clone(), schema.clone(), tree.clone());
        let data_reader = DataReader::new(data_partitions.clone(), schema.clone(), tree.clone());

        Self {
            schema,
            data_partitions,
            schema_partition: (*schema_partition).clone(),
            tree_partition: (*tree_partition).clone(),
            tree,
            data_writer,
            data_reader,
        }
    }

    pub fn save_schema(&mut self) {
        let ids = self.schema.types.iter().map(|t| t.id()).collect::<Vec<u16>>();
        let mut vals = ids.iter().enumerate().map(|(i, id)| {
            let mut v = id.to_be_bytes().to_vec();

            if let DataValue::Fixchar(_, j) = self.schema.types[i] {
                v.append(&mut j.to_be_bytes().to_vec());
            }

            v
        }).flatten().collect::<Vec<u8>>();

        self.schema_partition.borrow_mut().write_sectors(0, 0, &vals[..]).expect("Error writing schema to disk");
    }

    fn load_schema(schema_partition: &RefCell<CranePartition>) -> CraneSchema {
        let len = schema_partition.borrow().total_len();
        let bytes = schema_partition.borrow_mut().read_sectors(0, len).unwrap();
        let mut buffer = Buffer::new(bytes);

        let mut curr = buffer.consume(2);
        let mut value = u16::from_be_bytes(curr.try_into().unwrap());
        let mut ids = Vec::new();
        while value != 0 && !buffer.empty() {
            let mut meta_data: u64 = 0;
            if value == 6 {
                let c = buffer.consume(8);
                let v = u64::from_be_bytes(c.try_into().unwrap());
                meta_data = v;
            }
            ids.push(DataValue::from_id(value, meta_data));
            curr = buffer.consume(2);
            value = u16::from_be_bytes(curr.try_into().unwrap());
        }

        let schema = CraneSchema::new(ids);
        schema
    }

    pub fn save(&mut self) {
        self.save_schema();
        self.data_writer.save_tree(&mut *self.tree_partition.borrow_mut());
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

    pub fn run_command(&mut self, command: &mut dyn DataCommand) -> Result<(), DataError> {
        let mut state = DataState {
            schema: &self.schema,
            tree: &self.tree,
            data_partitions: self.data_partitions.iter().map(|p| p).collect(),
        };

        command.execute(&mut state)
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Borrow, fs::{File, OpenOptions}};

    use super::*;

    fn generate_disk() -> CraneDisk {
        let write = File::create("test/data/db.cdb").unwrap();
        let read = File::open("test/data/db.cdb").unwrap();

        let crane = CraneDisk::init_file(read, write);

        crane
    }

    fn load_disk() -> CraneDisk {
        let read = File::open("test/data/db.cdb").unwrap();
        let write = OpenOptions::new().write(true).open("test/data/db.cdb").unwrap();

        let crane = CraneDisk::from_file(read, write);

        crane
    }

    fn get_schema() -> CraneSchema {
       CraneSchema::new(vec![
            DataValue::UInt64(0),
            DataValue::UInt64(0),
            DataValue::UInt64(0),
            DataValue::Fixchar(String::new(), 32)
        ])
    }

    #[test]
    pub fn test_create_manager() {

        let mut disk = generate_disk();
        
        let schema = get_schema();
        let mut manager = DataManager::create_to_disk(&mut disk, 1, schema);

        let values = vec![
            DataValue::UInt64(1),
            DataValue::UInt64(5),
            DataValue::UInt64(2),
            DataValue::Fixchar("hello world".to_owned(), 32),
        ];

        manager.data_writer.write_value(values.clone()).unwrap();
        manager.data_writer.write_value(values.clone()).unwrap();
        manager.data_writer.write_value(values.clone()).unwrap();
        manager.data_writer.write_value(values.clone()).unwrap();

        manager.save();
        disk.save();
    }

    #[test]
    pub fn test_load_manager() {
        test_create_manager();

        let disk = load_disk();

        let manager = DataManager::from_disk(&disk, 1);


        let value = manager.data_reader.get_value(3);

        assert_ne!(value, None);

        let stuff = value.unwrap();
        assert_eq!(*stuff.get(0).unwrap(), DataValue::UInt64(1));
        assert_eq!(*stuff.get(1).unwrap(), DataValue::UInt64(5));
        assert_eq!(*stuff.get(2).unwrap(), DataValue::UInt64(2));
        assert_eq!(*stuff.get(3).unwrap(), DataValue::Fixchar("hello world".to_owned(), 32));
    }
}