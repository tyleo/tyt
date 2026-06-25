/// A scene-tree node whose type id this crate does not model, preserved verbatim
/// so an unrecognized or future node type survives a round trip instead of being
/// dropped. The node header's data-size field bounds the bytes kept here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QbtUnknownNode {
    /// Node type id, as stored.
    pub type_id: u32,

    /// Node data bytes (the region its header's data size spans).
    pub data: Vec<u8>,
}
