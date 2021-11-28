use super::FSError;


/// A reader is a trait that can read sectors inside of itself
pub(crate) trait Reader {
    /// The number of bytes a sector takes up
    fn sector_length(&self) -> u64;
    /// Reads bytes from sector start to end
    /// # Arguments
    /// * `start` - The sector to start reading from
    /// * `end` - The sector to end reading at
    fn read_sectors(&mut self, start: u64, end: u64) -> Result<Vec<u8>, FSError>;
    /// The maximum sector capacity of the writer
    fn capacity(&self) -> u64;
}