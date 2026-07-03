use crate::MeshSampler;

/// A material's metallic-roughness texture binding, the glTF packing with
/// metallic in the blue channel and roughness in the green. The texture is linear
/// data, sampled straight (no sRGB decode); each channel scales by its factor.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshMetallicRoughnessMap {
    /// The image and wrap modes to sample.
    pub sampler: MeshSampler,

    /// The metallic factor scaling the texel's blue channel.
    pub metallic: f64,

    /// The roughness factor scaling the texel's green channel.
    pub roughness: f64,
}
