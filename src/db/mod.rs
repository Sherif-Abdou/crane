mod item_tree;
mod data_manager;
mod data_command;
mod crane;

#[derive(Debug, PartialEq)]
pub(crate) enum DataError {
    OutOfStorage,
}