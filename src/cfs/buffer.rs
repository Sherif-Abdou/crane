
pub struct Buffer {
    raw: Vec<u8>
}

impl Buffer {
    pub fn new(raw: Vec<u8>) -> Buffer {
        Buffer {
            raw
        }
    }

    pub fn consume(&mut self, bytes: u64) -> Vec<u8> {
        (0..bytes).map(|_i| self.raw.pop().unwrap()).collect()
    }

    pub fn empty(&self) -> bool {
        self.raw.is_empty()
    }
}