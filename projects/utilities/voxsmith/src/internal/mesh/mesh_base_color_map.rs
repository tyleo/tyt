use crate::MeshSampler;
use ty_math::TyLinearRgbaColorF64;

/// A material's base-color texture binding: the sampler and the linear
/// base-color factor the sampled texel multiplies. The texture is sRGB, decoded
/// to linear at the sample site before the tint.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshBaseColorMap {
    /// The image and wrap modes to sample.
    pub sampler: MeshSampler,

    /// The linear base-color factor, multiplied component-wise into the sampled
    /// texel.
    pub factor: TyLinearRgbaColorF64,
}
