use crate::{VoxjBackend, VoxjSerdeObject};

/// The serde [`VoxjBackend`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxjSerdeBackend;

impl VoxjBackend for VoxjSerdeBackend {
    type Object = VoxjSerdeObject;
}
