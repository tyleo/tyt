use crate::{ByteReader, Result};

/// One `.gox` chunk's framing: its four-byte id and the data region between the
/// `id` / length header and the trailing `CRC` word.
pub struct Chunk<'a> {
    /// The four-byte chunk id.
    pub id: [u8; 4],

    /// The chunk's data region (the `length` bytes the header announces).
    pub data: &'a [u8],
}

/// Reads one chunk from `reader`, advancing past the whole chunk including its
/// trailing `CRC` word. The CRC is read and discarded: Goxel writes it as zero
/// and does not validate it.
pub fn read_chunk<'a>(reader: &mut ByteReader<'a>) -> Result<Chunk<'a>> {
    let id = reader.read_array::<4>()?;
    let length = reader.read_u32()? as usize;
    let data = reader.read_bytes(length)?;
    let _crc = reader.read_u32()?;
    Ok(Chunk { id, data })
}
