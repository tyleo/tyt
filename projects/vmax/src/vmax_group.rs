/// A group node in a Voxel Max scene hierarchy.
#[derive(Clone, Debug, PartialEq)]
pub struct VMaxGroup {
    pub name: String,
    pub id: String,
    pub parent_id: Option<String>,
    pub position: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
}
