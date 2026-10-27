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
