use crate::MeshWrap;
use ty_math::TyLinearRgbaColorF64;

/// A material's base-color texture binding: which decoded image in the mesh's
/// texture table, the sampler wrap modes, and the linear base-color factor the
/// sampled texel multiplies.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshBaseColorMap {
    /// Index into the mesh's texture table.
    pub image: usize,

    /// The linear base-color factor, multiplied component-wise into the sampled
    /// texel.
    pub factor: TyLinearRgbaColorF64,

    /// Wrap mode along the texture's `u` axis.
    pub wrap_s: MeshWrap,

    /// Wrap mode along the texture's `v` axis.
    pub wrap_t: MeshWrap,
}
