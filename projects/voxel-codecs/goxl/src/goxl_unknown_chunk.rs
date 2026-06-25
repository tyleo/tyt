/// A chunk this crate does not model, preserved verbatim so an unrecognized or
/// future chunk survives a decode / encode round trip rather than being dropped.
/// The chunk header's data-length field bounds the bytes kept here; the trailing
/// CRC word is not part of [`data`](Self::data).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GoxlUnknownChunk {
    /// The four-byte chunk type, as stored.
    pub id: [u8; 4],

    /// The chunk's data bytes (the region its header's data length spans).
    pub data: Vec<u8>,
}
