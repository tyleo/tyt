use crate::{
    BMeshFile, BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshPrimitive,
    BMeshTexture,
};
use branded_id::{IdVec, soa::IdRemap};

/// The id relabelings from a [`MeshMain::gc`](crate::MeshMain::gc), one per
/// id pool, for translating ids held across the call.
pub struct MeshGcRemap {
    /// The file id-pool relabeling.
    pub files: IdRemap<BMeshFile, u32>,

    /// The image id-pool relabeling.
    pub images: IdRemap<BMeshImage, u32>,

    /// The texture id-pool relabeling.
    pub textures: IdRemap<BMeshTexture, u32>,

    /// The material id-pool relabeling.
    pub materials: IdRemap<BMeshMaterial, u32>,

    /// The object id-pool relabeling.
    pub objects: IdRemap<BMeshObject, u32>,

    /// Each object's primitive relabeling, indexed by the object's old id,
    /// empty where an object was released.
    pub primitives: IdVec<BMeshObject, IdRemap<BMeshPrimitive, u32>>,

    /// The hierarchy node id-pool relabeling.
    pub hierarchy_nodes: IdRemap<BMeshHierarchyNode, u32>,
}
