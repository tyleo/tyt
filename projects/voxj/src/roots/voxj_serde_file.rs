use crate::{VoxjFile, VoxjSerdeBackend};

/// A [`VoxjFile`] whose objects carry encoded position and sample blocks
/// ([`VoxjSerdeObject`](crate::VoxjSerdeObject)); the form serialized to and from `.voxj`
/// JSON.
pub type VoxjSerdeFile = VoxjFile<VoxjSerdeBackend>;
