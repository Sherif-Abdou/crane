mod item_tree;
mod data_writer;
mod data_reader;
mod data_manager;
mod data_command;

#[derive(Debug, PartialEq)]
pub enum DataError {
    OutOfStorage,
}