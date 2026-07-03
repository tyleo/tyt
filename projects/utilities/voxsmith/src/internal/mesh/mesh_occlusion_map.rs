use crate::MeshSampler;

/// A material's occlusion texture binding, the glTF packing with occlusion in the
/// red channel. The texture is linear data, sampled straight; the voxelized
/// occlusion is `1 + strength * (red - 1)`, so `strength` scales how far the map
/// darkens from full (`1`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshOcclusionMap {
    /// The image and wrap modes to sample.
    pub sampler: MeshSampler,

    /// The occlusion strength scaling the map's deviation from `1`.
    pub strength: f64,
}
