use clap::ValueEnum;

/// Where `to vmax` stores object colors in the rebuilt `.vmax` package. Voxel
/// Max uses either a color image or a color table in the material settings
/// sidecar, but not both.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ColorFormat {
    /// Store colors as a `palette*.png` image.
    #[default]
    #[value(name = "png")]
    Png,
    /// Store colors in the `palette*.settings.vmaxpsb` sidecar.
    #[value(name = "plist")]
    Plist,
    /// Store colors in both the image and the sidecar.
    #[value(name = "all")]
    All,
}
