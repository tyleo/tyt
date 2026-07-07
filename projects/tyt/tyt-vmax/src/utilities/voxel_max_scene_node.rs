/// One node of a Voxel Max scene hierarchy, flattened from `scene.json` for the
/// codec-free command surface: the identity, parentage, local transform, and
/// authored bounds the `hierarchy` and `rename-node` commands need, without
/// exposing the `vmax` scene model.
#[derive(Clone, Debug)]
pub struct VoxelMaxSceneNode {
    /// Node UUID (`id`).
    pub id: String,
    /// Display name (object `n` / group `name`).
    pub name: String,
    /// Parent node UUID (`pid`), or `None` at the root.
    pub parent_id: Option<String>,
    /// Whether this node is a group (folder) rather than an object.
    pub is_group: bool,
    /// Local translation (`t_p`).
    pub position: [f64; 3],
    /// Local rotation (`t_r`) as axis-angle: axis `x, y, z` then angle in
    /// radians.
    pub rotation: [f64; 4],
    /// Local scale (`t_s`).
    pub scale: [f64; 3],
    /// Authored voxel box as `(min, max)` corners in the node's local space, or
    /// `None` when the object has no edited bounds.
    pub bounds: Option<([f64; 3], [f64; 3])>,
}
