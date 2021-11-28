use super::FSError;


/// A writer is an object who can write bytes to sectors inside of it
pub trait Writer {
    /// The number of bytes a sector takes up
    fn sector_length(&self) -> u64;
    /// Writes bytes
    /// # Arguments
    /// * `start` - The sector to start writing at
    /// * `offset` - The offset within(or beyond) the sector to start writing at
    /// * `bytes` - The bytes to write
    fn write_sectors(&mut self, start: u64, offset: u64, bytes: &[u8]) -> Result<(), FSError>;
    /// The maximum sector capacity of the writer
    fn capacity(&self) -> u64;
}