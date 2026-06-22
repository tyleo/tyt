use crate::{VoxjCodecBackend, VoxjFile};

/// A [`VoxjFile`] whose objects carry decoded geometry
/// ([`VoxjCodecObject`](crate::VoxjCodecObject)); the codec's top-level type,
/// encoded into a [`VoxjSerdeFile`](crate::VoxjSerdeFile) for serialization.
pub type VoxjCodecFile = VoxjFile<VoxjCodecBackend>;
