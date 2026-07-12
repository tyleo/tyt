use crate::{AtlasShape, MaterialMap, MeshMethod, ResourceStorage};
use branded_id::U32Id;
use voxcore::BVoxLayer;

/// A request to mesh an object and bake its palette materials into glTF
/// textures: the meshing strategy and scale, the maps to bake, where their
/// images go, and how the atlas canvas is shaped.
#[derive(Clone, Debug)]
pub struct MaterialMeshRequest {
    /// The meshing strategy. Greedy merges only same-material faces, so each
    /// face samples one atlas texel.
    pub method: MeshMethod,

    /// The object layer to read materials from.
    pub layer: U32Id<BVoxLayer>,

    /// Meters per voxel, applied as a uniform scale to every vertex.
    pub scale: f64,

    /// The maps to bake, each its own image, in order.
    pub maps: Vec<MaterialMap>,

    /// Where the baked images go.
    pub storage: ResourceStorage,

    /// How the atlas canvas is shaped around the material texels.
    pub shape: AtlasShape,
}
