/// A chunk this crate does not model, preserved verbatim so an unrecognized or
/// future chunk survives a decode / encode round trip rather than being dropped.
/// The legacy `MATT` material chunk is kept here rather than decoded; `MATL`
/// supersedes it and is modeled.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxUnknownChunk {
    /// The four-byte chunk id, as stored.
    pub id: [u8; 4],

    /// The chunk's content bytes (its `N`-byte region).
    pub content: Vec<u8>,

    /// The chunk's child bytes (its `M`-byte region); empty for the leaf chunks
    /// the `.vox` format defines.
    pub children: Vec<u8>,
}
