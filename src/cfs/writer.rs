pub struct WriteError {

}

pub trait Writer {
    /// The number of bytes a sector takes up
    fn sector_length(&self) -> u64;
    fn write_sectors(&mut self, start: u64, end: u64, bytes: &[u8]) -> Result<(), WriteError>;
    /// The maximum sector capacity of the writer
    fn capacity(&self) -> u64;
}