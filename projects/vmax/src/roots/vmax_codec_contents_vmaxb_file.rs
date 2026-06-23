use crate::{VMaxBrush, VMaxCamera, VMaxCodecVoxel, VMaxTools};

/// The decoded form of a `contents*.vmaxb` object: its voxels plus the editor
/// state Voxel Max preserves. The codec backend's contents representation (see
/// [`VMaxCodecBackend`](crate::VMaxCodecBackend)); `vmax-codec` decodes a
/// [`VMaxSerdeContentsVmaxbFile`](crate::VMaxSerdeContentsVmaxbFile) into this and encodes
/// it back. The snapshot edit-log collapses to its final voxels, but every
/// editor-state field round-trips verbatim.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxCodecContentsVmaxbFile {
    /// Decoded voxels in model space (the final state of the snapshot replay).
    pub voxels: Vec<VMaxCodecVoxel>,

    /// Object content UUID.
    pub uuid: String,

    /// Codable version.
    pub v: i64,

    /// Tool state; separate from the voxel geometry.
    pub tools: Option<VMaxTools>,

    /// Brush palette.
    pub brush: Option<VMaxBrush>,

    /// Per-object camera.
    pub cam: Option<VMaxCamera>,
}
