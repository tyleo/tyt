use crate::{ByteReader, Result};

/// One `.vox` chunk's framing: its id and the two byte regions that follow the
/// `id` / content-length / child-length header.
pub struct Chunk<'a> {
    /// The four-byte chunk id.
    pub id: [u8; 4],

    /// The chunk's content region (`N` bytes).
    pub content: &'a [u8],

    /// The chunk's child region (`M` bytes).
    pub children: &'a [u8],
}

/// Reads one chunk header and its content / child regions from `reader`,
/// advancing past the whole chunk.
pub fn read_chunk<'a>(reader: &mut ByteReader<'a>) -> Result<Chunk<'a>> {
    let id = reader.read_array::<4>()?;
    let content_len = reader.read_u32()? as usize;
    let children_len = reader.read_u32()? as usize;
    let content = reader.read_bytes(content_len)?;
    let children = reader.read_bytes(children_len)?;
    Ok(Chunk {
        id,
        content,
        children,
    })
}
