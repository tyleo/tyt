use crate::{VoxjBackend, VoxjCodecObject};

/// The codec [`VoxjBackend`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxjCodecBackend;

impl VoxjBackend for VoxjCodecBackend {
    type Object = VoxjCodecObject;
}
