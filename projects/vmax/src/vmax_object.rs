/// A single object in a Voxel Max scene.
#[derive(Clone, Debug, PartialEq)]
pub struct VMaxObject {
    pub name: String,
    pub data: String,
    pub palette: String,
    pub history: String,
    pub id: String,
    pub parent_id: Option<String>,
    pub position: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
    /// Center of the object's voxel bounds in model space (Voxel Max `e_c`).
    pub center: [f64; 3],
}
