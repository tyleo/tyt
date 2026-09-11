use crate::voxj::{VoxjSerialization, VoxjWriteOptions};

/// A Voxel Json write target: the container the document takes and the
/// writer's options.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxjWriteFormat {
    /// The container.
    pub serialization: VoxjSerialization,

    /// The writer's options.
    pub options: VoxjWriteOptions,
}
