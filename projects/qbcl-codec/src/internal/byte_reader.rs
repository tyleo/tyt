use crate::{Error, Result};

/// A bounds-checked little-endian cursor over a byte slice. A read past the end
/// returns [`Error::UnexpectedEof`] rather than panicking, so truncated input is
/// rejected instead of masked.
pub struct ByteReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    /// A reader positioned at the start of `bytes`.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    /// Bytes not yet consumed.
    pub fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.pos..]
    }

    /// Whether every byte has been consumed.
    pub fn is_empty(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    /// Reads `len` bytes, advancing past them.
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        match self.pos.checked_add(len) {
            Some(end) if end <= self.bytes.len() => {
                let slice = &self.bytes[self.pos..end];
                self.pos = end;
                Ok(slice)
            }
            _ => Err(eof(format!(
                "need {len} more bytes, only {} remain",
                self.bytes.len() - self.pos
            ))),
        }
    }

    /// Reads a single byte.
    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_array::<1>()?[0])
    }

    /// Reads a little-endian `u32`.
    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_array()?))
    }

    /// Reads a little-endian `i32`.
    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_array()?))
    }

    /// Reads a little-endian `f32`.
    #[cfg(feature = "qbt")]
    pub fn read_f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.read_array()?))
    }

    /// Reads a fixed-size array of bytes.
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self
            .read_bytes(N)?
            .try_into()
            .expect("read_bytes returns exactly N bytes"))
    }

    /// Reads `len` bytes as a UTF-8 Qubicle name string.
    pub fn read_string(&mut self, len: usize) -> Result<String> {
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|error| Error::Invalid(error.to_string()))
    }
}

/// An unexpected-end-of-input error carrying `message`.
fn eof(message: String) -> Error {
    Error::UnexpectedEof(message)
}
