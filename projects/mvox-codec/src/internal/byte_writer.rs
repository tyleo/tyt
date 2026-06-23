/// A little-endian byte sink mirroring [`ByteReader`](crate::ByteReader), used to
/// build chunk content and assemble the file.
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

    /// Appends a little-endian `u32`.
    pub fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Appends a little-endian `i32`.
    pub fn write_i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// Appends raw bytes.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    /// Appends a `.vox` `STRING`: a `u32` byte length then the UTF-8 bytes.
    pub fn write_string(&mut self, value: &str) {
        self.write_u32(value.len() as u32);
        self.write_bytes(value.as_bytes());
    }

    /// Appends a `.vox` `DICT`: a `u32` pair count then each key/value `STRING`.
    pub fn write_dict(&mut self, pairs: &[(String, String)]) {
        self.write_u32(pairs.len() as u32);
        for (key, value) in pairs {
            self.write_string(key);
            self.write_string(value);
        }
    }

    /// Appends a full chunk: the four-byte `id`, the content length, the child
    /// length, then the `content` and `children` bytes.
    pub fn write_chunk(&mut self, id: &[u8; 4], content: &[u8], children: &[u8]) {
        self.write_bytes(id);
        self.write_u32(content.len() as u32);
        self.write_u32(children.len() as u32);
        self.write_bytes(content);
        self.write_bytes(children);
    }
}
