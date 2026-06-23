use crate::{VoxjMain, VoxjSerdeBackend};

/// A [`VoxjMain`] whose objects carry encoded position and sample blocks; the
/// body of a [`VoxjSerdeFile`](crate::VoxjSerdeFile).
pub type VoxjSerdeMain = VoxjMain<VoxjSerdeBackend>;
