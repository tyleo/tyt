/// A single object in a Voxel Max scene.
#[derive(Clone, Debug, PartialEq)]
pub struct VMaxObject {
    pub name: String,
    pub data: String,
    pub palette: String,
    pub history: String,
    pub id: String,
    pub position: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
}
