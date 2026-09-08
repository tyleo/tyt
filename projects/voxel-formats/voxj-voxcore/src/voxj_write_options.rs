use crate::EditStateMode;
use voxj::objects::{PositionEncoding, SampleEncoding};

/// Options for writing a Voxel Json document. The default searches each
/// object's block encodings for the lowest cost, keeps the state's `ext`
/// block, and records the edit state only when an object carries margin
/// around its live voxels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VoxjWriteOptions {
    /// The position-block encoding, or `None` to search for the lowest cost
    /// paired with the sample encoding.
    pub position_encoding: Option<PositionEncoding>,

    /// The sample-block encoding, or `None` to search for the lowest cost
    /// paired with the position encoding.
    pub sample_encoding: Option<SampleEncoding>,

    /// Whether the state's `ext` block is written.
    pub ext: bool,

    /// When each object's editor build volume is recorded in the document's
    /// edit state.
    pub edit_state: EditStateMode,
}

impl Default for VoxjWriteOptions {
    fn default() -> Self {
        Self {
            position_encoding: None,
            sample_encoding: None,
            ext: true,
            edit_state: EditStateMode::Auto,
        }
    }
}
