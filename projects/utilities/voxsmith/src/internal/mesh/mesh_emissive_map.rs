use crate::MeshSampler;

/// A material's emissive texture binding. The texture is sRGB, decoded to linear
/// and tinted by the emissive RGB factor; the voxelized emissive color is the
/// resulting linear color, re-encoded to sRGB. The emissive strength is a
/// per-material scalar the texture does not carry, so the map overrides only the
/// color. The `KHR_materials_emissive_strength` multiplier is not read.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshEmissiveMap {
    /// The image and wrap modes to sample.
    pub sampler: MeshSampler,

    /// The linear emissive RGB factor, multiplied component-wise into the sampled
    /// texel.
    pub factor: [f64; 3],
}
