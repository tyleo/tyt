use crate::GltfExt;
use branded_id::U32Id;
use meshdoc::{
    BMeshHierarchyNode, MeshFile, MeshHierarchyNode, MeshImage, MeshMaterial, MeshPrimitive,
    MeshProperty, MeshTexture,
};

/// Everything a main holds, in listing order, for an equality check across a
/// round trip.
#[derive(Debug, PartialEq)]
pub struct DocumentSnapshot {
    pub files: Vec<MeshFile>,

    pub images: Vec<MeshImage>,

    pub textures: Vec<MeshTexture>,

    pub materials: Vec<MeshMaterial>,

    pub objects: Vec<(String, Vec<MeshProperty>, Vec<MeshPrimitive>)>,

    pub nodes: Vec<MeshHierarchyNode>,

    pub roots: Vec<U32Id<BMeshHierarchyNode>>,

    pub ext: GltfExt,
}
