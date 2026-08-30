/// A little-endian byte sink, the write counterpart of
/// [`ByteReader`](crate::ByteReader).
#[derive(Default)]
pub struct ByteWriter {
    bytes: Vec<u8>,
}

impl ByteWriter {
    /// An empty writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// The accumulated bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Appends one byte.
    pub fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    /// Appends a little-endian `u16`.
    #[allow(dead_code)]
    pub fn write_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Appends a little-endian `u32`.
    pub fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Appends a length or count as a little-endian `u32` (the formats store
    /// these in a `u32`). No real file exceeds `u32::MAX`; the `debug_assert!`
    /// catches a violation instead of truncating.
    pub fn write_len(&mut self, len: usize) {
        debug_assert!(
            len <= u32::MAX as usize,
            "length {len} exceeds the u32 the Qubicle formats store it in"
        );
        self.write_u32(len as u32);
    }

    /// Appends a little-endian `i32`.
    pub fn write_i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Appends a little-endian `f32`.
    #[allow(dead_code)]
    pub fn write_f32(&mut self, value: f32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Appends raw bytes.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }
}
