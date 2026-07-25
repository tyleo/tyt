use clap::ValueEnum;

/// Where each voxel's color and material come from when `voxelize` rasterizes a
/// mesh. Independent of [`FillMode`](crate::commands::FillMode), which sets
/// the geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum MaterialMode {
    /// Sample per-texel when the mesh carries textures, else per-primitive.
    #[value(name = "auto")]
    Auto,

    /// One material per glTF material, read from its flat PBR factors.
    #[value(name = "per-primitive")]
    PerPrimitive,

    /// Sample the material maps at each voxel's surface point.
    #[value(name = "per-texel")]
    PerTexel,

    /// Ignore the mesh's materials and paint the one `--fill-color`.
    #[value(name = "flat")]
    Flat,
}
