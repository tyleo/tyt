use crate::{VoxjCodecBackend, VoxjMain};

/// A [`VoxjMain`] whose objects carry decoded geometry; the body of a
/// [`VoxjCodecFile`](crate::VoxjCodecFile).
pub type VoxjCodecMain = VoxjMain<VoxjCodecBackend>;
