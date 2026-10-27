/// One `.gox` chunk's framing: its four-byte id and the data region between the
/// `id` / length header and the trailing `CRC` word.
pub struct Chunk<'a> {
    /// The four-byte chunk id.
    pub id: [u8; 4],

    /// The chunk's data region (the `length` bytes the header announces).
    pub data: &'a [u8],
}
