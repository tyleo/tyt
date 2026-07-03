use crate::MeshWrap;

/// The image and wrap modes a material's texture map samples: which decoded image
/// in the mesh texture table, and how a coordinate outside `[0, 1]` maps back
/// onto it. Each PBR map binding carries one. The color-space decode is chosen at
/// the sample site, not stored here, so one image sampled as sRGB by one map and
/// as linear data by another decodes correctly for each.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshSampler {
    /// Index into the mesh's texture table.
    pub image: usize,

    /// Wrap mode along the texture's `u` axis.
    pub wrap_s: MeshWrap,

    /// Wrap mode along the texture's `v` axis.
    pub wrap_t: MeshWrap,
}
