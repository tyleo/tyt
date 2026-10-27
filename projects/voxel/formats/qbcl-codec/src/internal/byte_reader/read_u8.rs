use crate::{ByteReader, Result};

impl ByteReader<'_> {
    /// Reads a single byte.
    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_array::<1>()?[0])
    }
}
