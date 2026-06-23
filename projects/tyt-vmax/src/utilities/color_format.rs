use clap::ValueEnum;

/// How `from-voxj` stores an object's colors in the rebuilt `.vmax` package.
/// Voxel Max accepts either a color image or a color table inside the material
/// settings sidecar, but uses only one at a time.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ColorFormat {
    /// Write the colors as a 256x1 `palette*.png` image and leave the material
    /// `palette*.settings.vmaxpsb` without a `colors` table. This matches a
    /// Voxel Max package that ships a color image.
    #[default]
    #[value(name = "png")]
    Png,
    /// Write the colors into the `palette*.settings.vmaxpsb` `colors` table and
    /// emit no `palette*.png`. The `pal` reference still names the (absent)
    /// image. This matches a Voxel Max package that keeps its colors in the
    /// plist.
    #[value(name = "plist")]
    Plist,
    /// Write the colors into both the `palette*.png` image and the material
    /// `palette*.settings.vmaxpsb` `colors` table.
    #[value(name = "all")]
    All,
}
