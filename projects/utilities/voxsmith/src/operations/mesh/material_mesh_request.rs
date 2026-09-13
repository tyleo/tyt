use crate::operations::mesh::{AtlasShape, MaterialMap, MeshMethod};

/// A request to mesh an object and bake its flattened layer materials into
/// textures.
#[derive(Clone, Debug)]
pub struct MaterialMeshRequest {
    /// The meshing strategy. Greedy merges only same-material faces, so each
    /// face samples one atlas texel.
    pub method: MeshMethod,

    /// Meters per voxel, applied as a uniform scale to every vertex.
    pub scale: f64,

    /// The maps to bake, each its own image, in order.
    pub maps: Vec<MaterialMap>,

    /// How the atlas canvas is shaped around the material texels.
    pub shape: AtlasShape,
}
