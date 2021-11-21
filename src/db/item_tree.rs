use std::{collections::BTreeMap, convert::TryInto};
use crate::cfs::{Buffer, CranePartition, Reader, Writer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub partition: u64,
    pub offset: u64,
}

impl Position {
    pub fn new(partition: u64, offset: u64) -> Self {
        Self { partition, offset }
    }  

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.append(&mut self.partition.to_be_bytes().to_vec());
        buf.append(&mut self.offset.to_be_bytes().to_vec());

        buf
    }

    pub fn from_bytes(bytes: &mut Buffer) -> Self {
        let partition = u64::from_be_bytes(bytes.consume(8)[..].try_into().unwrap());
        let offset = u64::from_be_bytes(bytes.consume(8)[..].try_into().unwrap());

        Self::new(partition, offset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemTree {
    pub tree: BTreeMap<u64, Position>,
    max_key: u64,
}

impl ItemTree {
    pub fn new() -> Self {
        ItemTree {
            tree: BTreeMap::new(),
            max_key: 0,
        }
    }

    pub fn max_key(&self) -> u64 {
        self.max_key
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.tree.iter().map(|(key, value)| {
            let mut key_bytes = key.to_be_bytes().to_vec();
            let mut value_bytes = value.to_bytes().to_vec();

            key_bytes.append(&mut value_bytes);
            key_bytes
        }).flatten().collect()
    }

    pub fn from_bytes(bytes: &mut Buffer) -> Self {
        let mut tree = BTreeMap::new();
        let mut m = 0u64;
        while !bytes.empty() {
            let key = u64::from_be_bytes(bytes.consume(8)[..].try_into().unwrap());
            let value = Position::from_bytes(bytes);
            if key == 0 {
                break;
            }

            tree.insert(key, value);
            m = u64::max(m, key);
        }

        Self {
            tree,
            max_key: m,
        }
    }

    pub fn to_partition(&self, partition: &mut CranePartition) {
        partition.write_sectors(0, 0, self.to_bytes()[..].try_into().unwrap()).unwrap();
    }

    pub fn from_partition(partition: &mut CranePartition) -> Self {
        let mut buffer = Buffer::new(partition.read_sectors(0, partition.total_len()).unwrap());

        Self::from_bytes(&mut buffer)
    }

    pub fn get(&self, key: u64) -> Option<Position> {
        self.tree.get(&key).cloned()
    }

    pub fn insert(&mut self, key: u64, partition: u64, offset: u64) {
        self.max_key = u64::max(self.max_key, key);
        self.tree.insert(key, Position::new(partition, offset));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_bytes() {
        let mut tree = ItemTree::new();

        tree.insert(1, 1, 0);
        tree.insert(1, 1, 8);

        let bytes = tree.to_bytes();
        let new_tree = ItemTree::from_bytes(&mut Buffer::new(bytes));

        assert_eq!(tree, new_tree);
    }
}