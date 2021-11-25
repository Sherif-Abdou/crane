use std::{cell::RefCell, fs::File, io::{Seek, SeekFrom, Write}, rc::Rc};

use crate::SECTOR_LENGTH;

use super::{crane_partition::CranePartition, root_partition::{self, RootPartition}};


pub struct CraneDisk {
    root_partition: RootPartition,
    pub partitions: Vec<Rc<RefCell<CranePartition>>>,
    read_file: Rc<RefCell<File>>,
    write_file: Rc<RefCell<File>>,
}

impl CraneDisk {
    pub fn from_file(read_file: File, write_file: File) -> Self {
        let read_rc = Rc::new(RefCell::new(read_file));
        let write_rc = Rc::new(RefCell::new(write_file));

        let rpartition = CranePartition::new(0, 0, 12, 12, 
            Rc::downgrade(&read_rc), Rc::downgrade(&write_rc));

        let root_partition = RootPartition::import_from(rpartition);
        let total_lens = &root_partition.compute_lens();
        let init_lens = &root_partition.init_lens;

        let partition_map: Vec<Rc<RefCell<CranePartition>>> = root_partition.partition_starts.iter()
            .filter(|x| **x != 0)
            .enumerate()
            .map(|(i, v)| CranePartition::new((i+1) as u64, *v, total_lens[i], init_lens[i], Rc::downgrade(&read_rc), Rc::downgrade(&write_rc)))
            .map(|v| Rc::new(RefCell::new(v)))
            .collect();

        CraneDisk {
            root_partition,
            partitions: partition_map,
            read_file: read_rc,
            write_file: write_rc,
        }
    }

    pub fn init_file(read_file: File, write_file: File) -> Self {
        let read_rc = Rc::new(RefCell::new(read_file));
        let write_rc = Rc::new(RefCell::new(write_file));

        let root_partition = RootPartition::new(CranePartition::new(0, 0, 12, 12, 
            Rc::downgrade(&read_rc), Rc::downgrade(&write_rc)));
        
        let mut disk = CraneDisk {
            root_partition,
            partitions: vec![],
            read_file: read_rc,
            write_file: write_rc,
        };
        
        disk.add_sectors(8);

        disk
    }

    /// Adds `sectors` sectors of empty bytes, returns the new sector length of the file
    pub fn add_sectors(&mut self, sectors: u64) -> u64 {
        let mut f = self.write_file.borrow_mut();
        f.seek(SeekFrom::End(0)).unwrap();

        for _ in 0..sectors {
            f.write(&[0u8; SECTOR_LENGTH]).unwrap();
        }

        self.len()
    }

    pub fn save(&mut self) {
        self.update_root();
    }

    pub fn append_partition(&mut self, sector_length: u64) -> u64 {
        let old_len = self.len();
        let new_len = self.add_sectors(sector_length);
        let id = (self.partitions.len() as u64) + 1;
        let partition = CranePartition::new(id, old_len, new_len-old_len, 0, 
        Rc::downgrade(&self.read_file), Rc::downgrade(&self.write_file));

        self.add_to_root(&partition);
        self.partitions.push(Rc::new(RefCell::new(partition)));
        
        id
    }

    pub fn get_partition_with_id(&self, id: u64) -> &Rc<RefCell<CranePartition>> {
        &self.partitions[id as usize - 1]
    }

    fn add_to_root(&mut self, partition: &CranePartition) {
        self.root_partition.partition_starts.push(partition.offset());
        self.root_partition.partition_ends.push(partition.total_len() + partition.offset());
        self.root_partition.init_lens.push(partition.initialized_len);

        self.root_partition.write();
    }

    fn update_root(&mut self) {
        self.root_partition.init_lens = self.partitions.iter().map(|x| x.borrow().initialized_len).collect();

        self.root_partition.write();
    }

    pub fn len(&self) -> u64 {
        (self.read_file.borrow().metadata().unwrap().len() as u64)/(SECTOR_LENGTH as u64)
    }
}


#[cfg(test)]
mod test {
    use std::fs::OpenOptions;

    use crate::cfs::writer::Writer;

    use super::*;

    #[test]
    fn test_new_disk() {
        let write_file = File::create("./test/disk/disk.db").unwrap();
        let read_file = File::open("./test/disk/disk.db").unwrap();

        let mut disk = CraneDisk::init_file(read_file, write_file);

        disk.append_partition(8);
        assert_eq!(disk.partitions.len(), 1);

        disk.partitions[0].borrow_mut().write_sectors(0, 0, &25u64.to_be_bytes()).unwrap();
        disk.save();
    }

    #[test]
    fn read_disk() {
        test_new_disk();
        let write_file = OpenOptions::new().write(true).open("./test/disk/disk.db").unwrap();
        let read_file = File::open("./test/disk/disk.db").unwrap();

        let disk = CraneDisk::from_file(read_file, write_file);

        assert_eq!(disk.partitions.len(), 1);
        assert_eq!(disk.partitions[0].borrow().initialized_len, 8);
    }
}