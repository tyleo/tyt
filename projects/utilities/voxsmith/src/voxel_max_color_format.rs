/// Where a palette's colors are stored when building a Voxel Max document.
///
/// Voxel Max can read an object's colors from a `palette*.png` image or from the
/// `colors` table of its material `palette*.settings.vmaxpsb` sidecar. The `pal`
/// reference always names the image; this selects where the bytes live.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VoxelMaxColorFormat {
    /// A 256x1 RGBA `palette*.png` image, with no `colors` table in the sidecar.
    #[default]
    Png,

    /// The material sidecar's `colors` table, with no image bytes.
    Plist,

    /// Both the image and the sidecar `colors` table.
    All,
}
