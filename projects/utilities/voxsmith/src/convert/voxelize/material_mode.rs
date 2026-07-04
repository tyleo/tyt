/// Where each voxelized voxel's color and material come from. Independent of
/// [`FillMode`](crate::FillMode), which chooses the geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialMode {
    /// Sample per-texel when the mesh carries textures, else per-primitive.
    Auto,

    /// One cell per glTF material, read from its flat PBR factors.
    PerPrimitive,

    /// Sample the material maps at each voxel's surface point.
    PerTexel,

    /// Ignore the mesh's materials and paint the one fill color.
    Flat,
}
