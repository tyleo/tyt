/// The occlusion a render shades with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderOcclusion {
    /// No occlusion.
    None,

    /// The neighbor-occupancy rule, one value per face corner from the
    /// three adjacent cells, as `voxsurface` computes it.
    Corner,
}
