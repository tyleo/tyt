use crate::{GltfExt, GltfMeshMain};
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

/// The snapshot of `main`.
pub fn snapshot(main: &GltfMeshMain) -> DocumentSnapshot {
    DocumentSnapshot {
        files: main.iter_files().map(|(_, file)| file.clone()).collect(),
        images: main.iter_images().map(|(_, image)| image.clone()).collect(),
        textures: main
            .iter_textures()
            .map(|(_, texture)| texture.clone())
            .collect(),
        materials: main
            .iter_materials()
            .map(|(_, material)| material.clone())
            .collect(),
        objects: main
            .iter_objects()
            .map(|(_, object)| {
                (
                    object.name().to_owned(),
                    object.properties().to_vec(),
                    object
                        .iter_primitives()
                        .map(|(_, primitive)| primitive.clone())
                        .collect(),
                )
            })
            .collect(),
        nodes: main
            .iter_hierarchy_nodes()
            .map(|(_, node)| node.clone())
            .collect(),
        roots: main.root_hierarchy_node_ids().to_vec(),
        ext: main.ext().clone(),
    }
}
