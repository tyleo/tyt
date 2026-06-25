use crate::qbcl::QbclNode;

/// A `.qbcl` model node: an inner scene-tree node grouping child nodes. The file
/// root is conventionally a model.
#[derive(Clone, Debug, PartialEq)]
pub struct QbclModel {
    /// A 36-byte chunk Qubicle writes after every model node's header. Its
    /// meaning is unconfirmed (it may encode a 3x3 transform); it is preserved
    /// verbatim so the file round-trips. The reference exporter writes
    /// [`DEFAULT_TRANSFORM`](Self::DEFAULT_TRANSFORM).
    pub transform: [u8; 36],

    /// Child nodes, in stored order.
    pub children: Vec<QbclNode>,
}

impl QbclModel {
    /// The 36-byte [`transform`](Self::transform) chunk the reference exporter
    /// writes: little-endian `1` at the first three `u32` slots, zero elsewhere.
    pub const DEFAULT_TRANSFORM: [u8; 36] = {
        let mut bytes = [0u8; 36];
        bytes[0] = 1;
        bytes[4] = 1;
        bytes[8] = 1;
        bytes
    };
}

impl Default for QbclModel {
    fn default() -> Self {
        Self {
            transform: Self::DEFAULT_TRANSFORM,
            children: Vec::new(),
        }
    }
}
