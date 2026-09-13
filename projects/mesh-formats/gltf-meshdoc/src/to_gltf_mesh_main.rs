use crate::{GltfExt, GltfExtMesh, GltfExtPrimitive, GltfExtSampler, GltfExtScene, GltfMeshMain};
use meshdoc::MeshMain;

/// Gives a bare main a synthesized [`GltfExt`], the main
/// [`to_gltf_file`](crate::to_gltf_file) writes as a file synthesized from
/// the document. Each entity's entry is the one a retain after the load
/// also gets. An image over its own bytes embeds, and one over a file
/// references it. The document is not reshaped, because glTF holds
/// everything the document does.
pub fn to_gltf_mesh_main(main: MeshMain<()>) -> GltfMeshMain {
    let mut ext = GltfExt {
        scenes: vec![GltfExtScene {
            node_ids: main.root_hierarchy_node_ids().to_vec(),
            ..Default::default()
        }],
        default_scene: Some(0),
        ..Default::default()
    };

    ext.asset.generator = Some(concat!("gltf-meshdoc ", env!("CARGO_PKG_VERSION")).to_owned());

    for (node_id, _) in main.iter_hierarchy_nodes() {
        ext.nodes.insert(node_id, Default::default());
    }

    for (object_id, object) in main.iter_objects() {
        ext.meshes.insert(
            object_id,
            GltfExtMesh {
                primitives: object
                    .iter_primitives()
                    .map(|(primitive_id, _)| (primitive_id, GltfExtPrimitive::default()))
                    .collect(),
                ..Default::default()
            },
        );
    }

    for (material_id, _) in main.iter_materials() {
        ext.materials.insert(material_id, Default::default());
    }

    for (texture_id, _) in main.iter_textures() {
        ext.textures.insert(
            texture_id,
            crate::GltfExtTexture {
                sampler: Some(GltfExtSampler::default()),
                ..Default::default()
            },
        );
    }

    for (image_id, _) in main.iter_images() {
        ext.images.insert(image_id, Default::default());
    }

    main.put_ext(ext)
}
