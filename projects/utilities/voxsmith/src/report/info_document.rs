use crate::VoxelFormat;

/// What [`render_info`](crate::render_info) reports about a document beyond
/// its scene: where it came from and what the source carried that the loaded
/// state drops.
#[derive(Clone, Copy, Debug)]
pub struct InfoDocument<'a> {
    /// The file name, printed as the report title.
    pub name: &'a str,

    /// The format the document was read as.
    pub format: VoxelFormat,

    /// The stamped format version, for a Voxel Json source.
    pub voxj_version: Option<u32>,

    /// Whether the source carried an `ext` block.
    pub has_ext: bool,
}
