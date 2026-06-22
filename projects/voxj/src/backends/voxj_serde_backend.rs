use crate::{VoxjBackend, VoxjSerdeObject};

/// The serde [`VoxjBackend`]: objects are [`VoxjSerdeObject`]s carrying encoded
/// position and sample blocks, the form read from and written to `.voxj` JSON.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxjSerdeBackend;

impl VoxjBackend for VoxjSerdeBackend {
    type Object = VoxjSerdeObject;
}
