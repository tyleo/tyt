use crate::{VoxjBackend, VoxjCodecObject};

/// The codec [`VoxjBackend`]: objects are [`VoxjCodecObject`]s carrying decoded
/// geometry (raw positions and samples), the form the codec encodes into and
/// decodes out of the serde form.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxjCodecBackend;

impl VoxjBackend for VoxjCodecBackend {
    type Object = VoxjCodecObject;
}
