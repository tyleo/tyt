use crate::{ByteReader, Result};

impl ByteReader<'_> {
    /// Reads a little-endian `f32`.
    pub fn read_f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.read_array()?))
    }
}
