use crate::{VoxjCodecBackend, VoxjMain};

/// A [`VoxjMain`] whose objects carry decoded geometry
/// ([`VoxjCodecObject`](crate::VoxjCodecObject)); the body of a
/// [`VoxjCodecFile`](crate::VoxjCodecFile).
pub type VoxjCodecMain = VoxjMain<VoxjCodecBackend>;
