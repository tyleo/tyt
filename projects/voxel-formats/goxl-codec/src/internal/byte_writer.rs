/// A little-endian byte sink, the write counterpart of
/// [`ByteReader`](crate::ByteReader), used to build chunk content and assemble
/// the file.
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

    /// Appends a little-endian `u32`.
    pub fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Appends a little-endian `i32`.
    pub fn write_i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Appends a length or count as a little-endian `u32` (the format stores
    /// these in a 32-bit int). No real file exceeds `u32::MAX`; the
    /// `debug_assert!` catches a violation instead of truncating.
    pub fn write_len(&mut self, len: usize) {
        debug_assert!(
            len <= u32::MAX as usize,
            "length {len} exceeds the 32-bit int the .gox format stores it in"
        );
        self.write_u32(len as u32);
    }

    /// Appends raw bytes.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    /// Appends a chunk's trailing `DICT`: each entry is a `u32` key length, the
    /// key bytes, a `u32` value length, then the value bytes. Goxel reads
    /// entries until the chunk ends, so no terminating zero-length key is
    /// written.
    pub fn write_dict(&mut self, pairs: &[(String, Vec<u8>)]) {
        for (key, value) in pairs {
            self.write_len(key.len());
            self.write_bytes(key.as_bytes());
            self.write_len(value.len());
            self.write_bytes(value);
        }
    }

    /// Appends a full chunk: the four-byte `id`, the content length, the
    /// `content` bytes, then a zero `CRC` word. Goxel writes the CRC as zero and
    /// does not validate it on read.
    pub fn write_chunk(&mut self, id: &[u8; 4], content: &[u8]) {
        self.write_bytes(id);
        self.write_len(content.len());
        self.write_bytes(content);
        self.write_u32(0);
    }
}
