use crate::{
    GltfExtAnimation, GltfExtAsset, GltfExtImage, GltfExtMaterial, GltfExtMesh, GltfExtNode,
    GltfExtPrimitive, GltfExtSampler, GltfExtScene, GltfExtSkin, GltfExtTexture,
};
use branded_id::{U32Id, soa::IdRemap};
use meshdoc::{
    BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshPrimitive, BMeshTexture,
    Error as MeshError, MeshExt, MeshGcRemap, MeshState, Result as MeshResult,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// The `gltf` ext payload stashed on a [`MeshMain`](meshdoc::MeshMain): the
/// glTF state with no native meshdoc home, kept so a document loaded from a
/// glTF file can be written back exactly.
///
/// The meshes, materials, textures, images, and nodes become native
/// entities. This holds the rest, with one entry per entity keyed by the
/// entity's id. The entries follow the state through the
/// [`MeshExt`](meshdoc::MeshExt) hooks:
///
/// 1. an entity retained after the load gets the entry the synthesizer
///    would build
/// 2. a released entity drops its entry and every reference to it
/// 3. a gc rekeys the entries
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GltfExt {
    /// The `asset` block.
    #[cfg_attr(feature = "serde", serde(default))]
    pub asset: GltfExtAsset,

    /// The extensions the document declares it uses, beyond those the writer
    /// declares for what it writes.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "extensions-used",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub extensions_used: Vec<String>,

    /// The extensions the document declares it requires.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "extensions-required",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub extensions_required: Vec<String>,

    /// The root's `extras`, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub extras: Option<Value>,

    /// The root's extensions, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Map::is_empty")
    )]
    pub extensions: Map<String, Value>,

    /// The relative URI the geometry buffer was loaded from as a loose file,
    /// or `None` when it was embedded. The JSON container writes it back the
    /// same way.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "buffer-uri",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub buffer_uri: Option<String>,

    /// The scenes, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub scenes: Vec<GltfExtScene>,

    /// Index into [`scenes`](Self::scenes) of the default scene, if declared.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "default-scene",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub default_scene: Option<u32>,

    /// The cameras, in stored order, each as its JSON.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub cameras: Vec<Value>,

    /// The skins, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub skins: Vec<GltfExtSkin>,

    /// The animations, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub animations: Vec<GltfExtAnimation>,

    /// Per-node state, keyed by node id.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub nodes: BTreeMap<U32Id<BMeshHierarchyNode>, GltfExtNode>,

    /// Per-mesh state, keyed by the object's id.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub meshes: BTreeMap<U32Id<BMeshObject>, GltfExtMesh>,

    /// Per-material state, keyed by material id.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub materials: BTreeMap<U32Id<BMeshMaterial>, GltfExtMaterial>,

    /// Per-texture state, keyed by texture id.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub textures: BTreeMap<U32Id<BMeshTexture>, GltfExtTexture>,

    /// Per-image state, keyed by image id.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub images: BTreeMap<U32Id<BMeshImage>, GltfExtImage>,
}

impl MeshExt for GltfExt {
    fn hierarchy_node_did_retain(
        &mut self,
        _state: &MeshState,
        node_id: U32Id<BMeshHierarchyNode>,
    ) -> MeshResult<()> {
        self.nodes.insert(node_id, GltfExtNode::default());
        Ok(())
    }

    /// The node leaves every scene, skin, and animation that referenced it.
    fn hierarchy_node_will_release(
        &mut self,
        _state: &MeshState,
        node_id: U32Id<BMeshHierarchyNode>,
    ) -> MeshResult<()> {
        if self.nodes.remove(&node_id).is_none() {
            return Err(ext_error(format!(
                "gltf ext has no node entry for node {}",
                node_id.to_u32()
            )));
        }

        for scene in &mut self.scenes {
            scene.node_ids.retain(|id| *id != node_id);
        }

        for skin in &mut self.skins {
            skin.joint_node_ids.retain(|id| *id != node_id);
            if skin.skeleton_node_id == Some(node_id) {
                skin.skeleton_node_id = None;
            }
        }

        for animation in &mut self.animations {
            animation
                .channels
                .retain(|channel| channel.target_node_id != node_id);
        }

        Ok(())
    }

    fn object_did_retain(
        &mut self,
        state: &MeshState,
        object_id: U32Id<BMeshObject>,
    ) -> MeshResult<()> {
        let object = state.object(object_id).expect("a retained object is live");

        let primitives = object
            .iter_primitives()
            .map(|(primitive_id, _)| (primitive_id, GltfExtPrimitive::default()))
            .collect();

        self.meshes.insert(
            object_id,
            GltfExtMesh {
                primitives,
                ..Default::default()
            },
        );
        Ok(())
    }

    fn object_will_release(
        &mut self,
        _state: &MeshState,
        object_id: U32Id<BMeshObject>,
    ) -> MeshResult<()> {
        if self.meshes.remove(&object_id).is_none() {
            return Err(ext_error(format!(
                "gltf ext has no mesh entry for object {}",
                object_id.to_u32()
            )));
        }
        Ok(())
    }

    fn primitive_did_retain(
        &mut self,
        _state: &MeshState,
        object_id: U32Id<BMeshObject>,
        primitive_id: U32Id<BMeshPrimitive>,
    ) -> MeshResult<()> {
        let Some(mesh) = self.meshes.get_mut(&object_id) else {
            return Err(ext_error(format!(
                "gltf ext has no mesh entry for object {}",
                object_id.to_u32()
            )));
        };

        mesh.primitives
            .insert(primitive_id, GltfExtPrimitive::default());
        Ok(())
    }

    fn primitive_will_release(
        &mut self,
        _state: &MeshState,
        object_id: U32Id<BMeshObject>,
        primitive_id: U32Id<BMeshPrimitive>,
    ) -> MeshResult<()> {
        let removed = self
            .meshes
            .get_mut(&object_id)
            .and_then(|mesh| mesh.primitives.remove(&primitive_id));

        if removed.is_none() {
            return Err(ext_error(format!(
                "gltf ext has no primitive entry for object {} primitive {}",
                object_id.to_u32(),
                primitive_id.to_u32()
            )));
        }
        Ok(())
    }

    fn material_did_retain(
        &mut self,
        _state: &MeshState,
        material_id: U32Id<BMeshMaterial>,
    ) -> MeshResult<()> {
        self.materials
            .insert(material_id, GltfExtMaterial::default());
        Ok(())
    }

    fn material_will_release(
        &mut self,
        _state: &MeshState,
        material_id: U32Id<BMeshMaterial>,
    ) -> MeshResult<()> {
        if self.materials.remove(&material_id).is_none() {
            return Err(ext_error(format!(
                "gltf ext has no material entry for material {}",
                material_id.to_u32()
            )));
        }
        Ok(())
    }

    /// A texture retained after the load gets its own sampler.
    fn texture_did_retain(
        &mut self,
        _state: &MeshState,
        texture_id: U32Id<BMeshTexture>,
    ) -> MeshResult<()> {
        self.textures.insert(
            texture_id,
            GltfExtTexture {
                sampler: Some(GltfExtSampler::default()),
                ..Default::default()
            },
        );
        Ok(())
    }

    fn texture_will_release(
        &mut self,
        _state: &MeshState,
        texture_id: U32Id<BMeshTexture>,
    ) -> MeshResult<()> {
        if self.textures.remove(&texture_id).is_none() {
            return Err(ext_error(format!(
                "gltf ext has no texture entry for texture {}",
                texture_id.to_u32()
            )));
        }
        Ok(())
    }

    fn image_did_retain(
        &mut self,
        _state: &MeshState,
        image_id: U32Id<BMeshImage>,
    ) -> MeshResult<()> {
        self.images.insert(image_id, GltfExtImage::default());
        Ok(())
    }

    fn image_will_release(
        &mut self,
        _state: &MeshState,
        image_id: U32Id<BMeshImage>,
    ) -> MeshResult<()> {
        if self.images.remove(&image_id).is_none() {
            return Err(ext_error(format!(
                "gltf ext has no image entry for image {}",
                image_id.to_u32()
            )));
        }
        Ok(())
    }

    fn did_gc(&mut self, _state: &MeshState, remap: &MeshGcRemap) -> MeshResult<()> {
        let nodes = rekey(&self.nodes, &remap.hierarchy_nodes, "node")?;
        let materials = rekey(&self.materials, &remap.materials, "material")?;
        let textures = rekey(&self.textures, &remap.textures, "texture")?;
        let images = rekey(&self.images, &remap.images, "image")?;

        let mut meshes = BTreeMap::new();
        for (&old_object_id, mesh) in &self.meshes {
            let Some(object_id) = remap.objects.new_id(old_object_id) else {
                return Err(gc_error("object", old_object_id.to_u32()));
            };

            let mut mesh = mesh.clone();
            mesh.primitives = rekey(
                &mesh.primitives,
                &remap.primitives[old_object_id.to_usize_id()],
                "primitive",
            )?;
            meshes.insert(object_id, mesh);
        }

        let node = |old_id: U32Id<BMeshHierarchyNode>| {
            remap
                .hierarchy_nodes
                .new_id(old_id)
                .ok_or_else(|| gc_error("node", old_id.to_u32()))
        };

        for scene in &mut self.scenes {
            for node_id in &mut scene.node_ids {
                *node_id = node(*node_id)?;
            }
        }

        for skin in &mut self.skins {
            for node_id in &mut skin.joint_node_ids {
                *node_id = node(*node_id)?;
            }
            if let Some(node_id) = &mut skin.skeleton_node_id {
                *node_id = node(*node_id)?;
            }
        }

        for animation in &mut self.animations {
            for channel in &mut animation.channels {
                channel.target_node_id = node(channel.target_node_id)?;
            }
        }

        self.nodes = nodes;
        self.meshes = meshes;
        self.materials = materials;
        self.textures = textures;
        self.images = images;
        Ok(())
    }
}

/// `entries` rekeyed through `remap`. Errors when an entry's entity was
/// released without its entry.
fn rekey<Brand, V: Clone>(
    entries: &BTreeMap<U32Id<Brand>, V>,
    remap: &IdRemap<Brand, u32>,
    what: &str,
) -> MeshResult<BTreeMap<U32Id<Brand>, V>> {
    let mut rekeyed = BTreeMap::new();

    for (&old_id, entry) in entries {
        let Some(id) = remap.new_id(old_id) else {
            return Err(gc_error(what, old_id.to_u32()));
        };

        rekeyed.insert(id, entry.clone());
    }

    Ok(rekeyed)
}

/// The error for an entry whose entity a gc released.
fn gc_error(what: &str, id: u32) -> MeshError {
    ext_error(format!(
        "gc released {what} {id}, which still has a gltf ext entry"
    ))
}

/// The ext's refusal or inconsistency as meshdoc reports it.
fn ext_error(reason: String) -> MeshError {
    MeshError::Ext { reason }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::{GltfExt, GltfExtNode, GltfExtScene};
    use branded_id::U32Id;

    /// The node map keys and the scene node ids ride as bare `u32`s and come
    /// back branded.
    #[test]
    fn round_trips_the_id_keys_through_json() {
        let mut ext = GltfExt::default();

        ext.nodes.insert(U32Id::from_u32(7), GltfExtNode::default());

        ext.scenes.push(GltfExtScene {
            node_ids: vec![U32Id::from_u32(7)],
            ..Default::default()
        });

        let json = serde_json::to_string(&ext).unwrap();

        assert!(json.contains(r#""nodes":{"7":"#));

        assert!(json.contains(r#""node-ids":[7]"#));

        assert_eq!(serde_json::from_str::<GltfExt>(&json).unwrap(), ext);
    }
}
