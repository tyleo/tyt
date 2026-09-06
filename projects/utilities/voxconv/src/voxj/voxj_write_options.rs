use crate::voxj::VoxjSerialization;
use voxj::objects::{PositionEncoding, SampleEncoding};
use voxj_voxcore::EditStateMode;

/// Writer options for a Voxel Json document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VoxjWriteOptions {
    /// The encoder and container the document is written with.
    pub serialization: VoxjSerialization,

    /// The position-block encoding, or `None` to search for the smallest
    /// paired with the sample encoding.
    pub position_encoding: Option<PositionEncoding>,

    /// The sample-block encoding, or `None` to search for the smallest
    /// paired with the position encoding.
    pub sample_encoding: Option<SampleEncoding>,

    /// Whether the state's ext is persisted as the document's `ext` block.
    pub ext: bool,

    /// When each object's editor build volume is recorded in the document's
    /// edit state.
    pub edit_state: EditStateMode,
}

/// Compact JSON, the smallest block encodings, the ext block kept, and the
/// edit state recorded when an object has margin around its live voxels.
impl Default for VoxjWriteOptions {
    fn default() -> Self {
        Self {
            serialization: VoxjSerialization::default(),
            position_encoding: None,
            sample_encoding: None,
            ext: true,
            edit_state: EditStateMode::Auto,
        }
    }
}
