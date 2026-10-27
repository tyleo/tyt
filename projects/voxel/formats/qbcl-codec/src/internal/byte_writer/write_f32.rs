use crate::ByteWriter;

impl ByteWriter {
    /// Appends a little-endian `f32`.
    pub fn write_f32(&mut self, value: f32) {
        self.write_bytes(&value.to_le_bytes());
    }
}
