use crate::qbt::{QbtColor, QbtNode};

/// A Qubicle Binary Tree `.qbt` file: header, optional color map, and scene
/// tree.
#[derive(Clone, Debug, PartialEq)]
pub struct QbtFile {
    /// `(major, minor)` version from the header; the reference exporter writes
    /// `(1, 0)`.
    pub version: (u8, u8),

    /// `[x, y, z]` global scale applied to the whole model.
    pub global_scale: [f32; 3],

    /// The `COLORMAP` palette, in stored order; empty when voxels store colors
    /// directly. When non-empty, a voxel's red byte indexes this map (see
    /// [`QbtVoxel`](crate::qbt::QbtVoxel)).
    pub color_map: Vec<QbtColor>,

    /// The `DATATREE` root node, conventionally a [`QbtNode::Model`].
    pub root: QbtNode,
}

impl Default for QbtFile {
    fn default() -> Self {
        Self {
            version: (1, 0),
            global_scale: [1.0, 1.0, 1.0],
            color_map: Vec::new(),
            root: QbtNode::default(),
        }
    }
}
