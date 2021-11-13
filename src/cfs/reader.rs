pub struct ReadError {

}

pub trait Reader {
    /// The number of bytes a sector takes up
    fn sector_length(&self) -> u64;
    fn read_sectors(&mut self, start: u64, end: u64) -> Result<Vec<u8>, ReadError>;
    /// The maximum sector capacity of the writer
    fn capacity(&self) -> u64;

}