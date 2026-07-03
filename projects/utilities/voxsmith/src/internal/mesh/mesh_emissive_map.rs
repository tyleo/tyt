use crate::MeshSampler;

/// A material's emissive texture binding. The texture is sRGB, decoded to linear
/// and tinted by the emissive RGB factor; the voxelized emissive strength is the
/// strongest resulting channel, matching how the flat path collapses the emissive
/// factor. The `KHR_materials_emissive_strength` multiplier is not applied.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshEmissiveMap {
    /// The image and wrap modes to sample.
    pub sampler: MeshSampler,

    /// The linear emissive RGB factor, multiplied component-wise into the sampled
    /// texel before the strongest channel is taken.
    pub factor: [f64; 3],
}
