use std::io::{Error as IOError, ErrorKind, Result};

/// A bounds-checked little-endian cursor over a byte slice. Every read that
/// would run past the end returns an [`ErrorKind::UnexpectedEof`] error rather
/// than panicking, so truncated input is rejected instead of masked.
pub struct ByteReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    /// A reader positioned at the start of `bytes`.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    /// The bytes not yet consumed.
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

    /// Reads one byte.
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

    /// Reads a fixed-size array of bytes.
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self
            .read_bytes(N)?
            .try_into()
            .expect("read_bytes returns exactly N bytes"))
    }

    /// Reads a `.vox` `STRING`: a `u32` byte length then that many UTF-8 bytes.
    pub fn read_string(&mut self) -> Result<String> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|error| IOError::new(ErrorKind::InvalidData, error))
    }

    /// Reads a `.vox` `DICT`: a `u32` pair count then that many key/value
    /// `STRING` pairs, returned in stored order.
    pub fn read_dict(&mut self) -> Result<Vec<(String, String)>> {
        let count = self.read_u32()? as usize;
        // A pair occupies at least eight bytes (two length-prefixed strings), so
        // a count larger than the remaining bytes allow is malformed; cap the
        // pre-allocation at that bound so a bogus count cannot demand a huge one.
        let mut pairs = Vec::with_capacity(count.min(self.remaining().len() / 8));
        for _ in 0..count {
            let key = self.read_string()?;
            let value = self.read_string()?;
            pairs.push((key, value));
        }
        Ok(pairs)
    }
}

/// An unexpected-end-of-input error carrying `message`.
fn eof(message: String) -> IOError {
    IOError::new(ErrorKind::UnexpectedEof, message)
}
