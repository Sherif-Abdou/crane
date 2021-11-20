use std::{cell::RefCell, fs::File, rc::Rc};

use super::{crane_partition::CranePartition, root_partition::{self, RootPartition}};




pub struct CraneDisk {
    root_partition: RootPartition,
    partitions: Vec<CranePartition>,
    file: Rc<RefCell<File>>,
}

impl CraneDisk {
    pub fn from_file(file: File) -> Self {
        let rc = Rc::new(RefCell::new(file));

        let rpartition = CranePartition::new(0, 0, 4, 4, Rc::downgrade(&rc));
        let root_partition = RootPartition::import_from(rpartition);

        let partition_map: Vec<CranePartition> = root_partition.partition_starts.iter()
            .filter(|x| **x != 0)
            .enumerate()
            .map(|(i, v)| CranePartition::new((i+1) as u64, *v, 512, 0, Rc::downgrade(&rc)))
            .collect();

        CraneDisk {
            root_partition,
            partitions: partition_map,
            file: rc
        }
    }
}