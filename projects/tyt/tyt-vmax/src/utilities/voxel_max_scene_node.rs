/// One node of a Voxel Max scene hierarchy, flattened from `scene.json` for the
/// codec-free command surface: just the identity and parentage the `hierarchy`
/// and `rename-node` commands need, without exposing the `vmax` scene model.
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
}
