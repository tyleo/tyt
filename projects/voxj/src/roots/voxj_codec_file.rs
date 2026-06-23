use crate::{VoxjCodecBackend, VoxjFile};

/// A [`VoxjFile`] whose objects carry decoded geometry; the codec's top-level
/// type, encoded into a [`VoxjSerdeFile`](crate::VoxjSerdeFile) for
/// serialization.
pub type VoxjCodecFile = VoxjFile<VoxjCodecBackend>;
