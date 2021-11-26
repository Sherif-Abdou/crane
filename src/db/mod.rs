mod item_tree;
mod data_writer;
mod data_reader;
mod data_manager;

#[derive(Debug, PartialEq)]
pub enum DataError {
    OutOfStorage,
}