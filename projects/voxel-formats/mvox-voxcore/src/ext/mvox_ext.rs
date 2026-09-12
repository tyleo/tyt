use crate::{
    MVoxExtCamera, MVoxExtLayer, MVoxExtMaterial, MVoxExtNode, MVoxExtNodeBody,
    MVoxExtUnknownChunk, SceneNodeKind, insert_synthesized_scene_node,
};
use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, mem};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, Error, Result, VoxExt, VoxGcRemap,
    VoxState,
};

/// The `mvox` ext payload stashed on a [`VoxMain`](voxcore::VoxMain):
/// the MagicaVoxel `.vox` state with no native voxcore home, kept so a file
/// loaded from a MagicaVoxel package can be written back exactly.
///
/// Geometry, colors, and the scene graph become native voxcore entities. This
/// holds the rest. The scene nodes are keyed by hierarchy node and the
/// materials by material of the state's one palette. Both follow the state
/// through the [`VoxExt`](voxcore::VoxExt) hooks.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MVoxExt {
    /// The format version from the header (`version`).
    pub version: u32,

    /// Whether the file carried an `RGBA` palette chunk. When false the colors
    /// came from the built-in palette and no chunk is written back.
    #[cfg_attr(feature = "serde", serde(rename = "palette-present"))]
    pub palette_present: bool,

    /// Per-material provenance, keyed by material. A material with no entry
    /// writes no `MATL` chunk.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub materials: BTreeMap<U32Id<BVoxMaterial>, MVoxExtMaterial>,

    /// Per scene-node provenance, keyed by hierarchy node. Every node has an
    /// entry: the loader builds one per scene node and the retain hook builds
    /// one for a node retained after the load.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "scene-nodes",
            default,
            skip_serializing_if = "BTreeMap::is_empty"
        )
    )]
    pub scene_nodes: BTreeMap<U32Id<BVoxHierarchyNode>, MVoxExtNode>,

    /// The layer definitions (`LAYR`), preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub layers: Vec<MVoxExtLayer>,

    /// The render-settings chunks (`rOBJ`), each an ordered key/value list.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "render-objects",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub render_objects: Vec<Vec<(String, String)>>,

    /// The render cameras (`rCAM`), preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub cameras: Vec<MVoxExtCamera>,

    /// The palette color names (`NOTE`), in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "palette-notes",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub palette_notes: Vec<String>,

    /// The palette index map (`IMAP`) as its 256 bytes, each a material id,
    /// or `None` when the file omits it. Held as a `Vec` because serde does
    /// not derive for `[u8; 256]`.
    #[cfg_attr(
        feature = "serde",
        serde(rename = "index-map", default, skip_serializing_if = "Option::is_none")
    )]
    pub index_map: Option<Vec<u8>>,

    /// Chunks the mvox crate does not model, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "unknown-chunks",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub unknown_chunks: Vec<MVoxExtUnknownChunk>,
}

/// The index map follows a material release through `gc`: a byte for a
/// released material keeps its index, now an empty color, until the
/// compaction frees one, which keeps the map a permutation. An object release
/// is refused while a shape entry still draws the object, because the ext
/// was out of step.
impl VoxExt for MVoxExt {
    fn hierarchy_node_did_retain(
        &mut self,
        state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        let node = state
            .hierarchy_node(node_id)
            .expect("a retained node is live");
        let kind = if !node.child_object_ids.is_empty() {
            SceneNodeKind::Shape
        } else if node.child_node_ids.len() == 1 {
            SceneNodeKind::Transform
        } else {
            SceneNodeKind::Group
        };
        insert_synthesized_scene_node(&mut self.scene_nodes, node_id, kind, node);
        Ok(())
    }

    fn hierarchy_node_will_release(
        &mut self,
        _state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> Result<()> {
        self.scene_nodes.remove(&node_id);
        Ok(())
    }

    fn object_will_release(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
    ) -> Result<()> {
        let drawn_by: Vec<i32> = self
            .scene_nodes
            .values()
            .filter(|entry| match &entry.body {
                MVoxExtNodeBody::Shape { models } => {
                    models.iter().any(|model| model.object == object_id)
                }
                _ => false,
            })
            .map(|entry| entry.id)
            .collect();
        if drawn_by.is_empty() {
            return Ok(());
        }

        Err(Error::Ext {
            reason: format!(
                "mvox scene nodes {drawn_by:?} still draw object {}, which no hierarchy node \
                 places",
                object_id.to_u32()
            ),
        })
    }

    fn palette_will_release(
        &mut self,
        _state: &VoxState,
        _palette_id: U32Id<BVoxPalette>,
    ) -> Result<()> {
        self.materials.clear();
        self.index_map = None;
        Ok(())
    }

    fn materials_will_release(
        &mut self,
        _state: &VoxState,
        _palette_id: U32Id<BVoxPalette>,
        material_ids: &[U32Id<BVoxMaterial>],
    ) -> Result<()> {
        for material_id in material_ids {
            self.materials.remove(material_id);
        }
        Ok(())
    }

    fn did_gc(&mut self, state: &VoxState, remap: &VoxGcRemap) -> Result<()> {
        let scene_nodes = mem::take(&mut self.scene_nodes);
        for (old_id, mut entry) in scene_nodes {
            let node_id = remap
                .hierarchy_nodes
                .new_id(old_id)
                .ok_or_else(|| stale("hierarchy node", old_id.to_u32()))?;
            if let MVoxExtNodeBody::Shape { models } = &mut entry.body {
                for model in models {
                    model.object = remap
                        .objects
                        .new_id(model.object)
                        .ok_or_else(|| stale("object", model.object.to_u32()))?;
                }
            }
            self.scene_nodes.insert(node_id, entry);
        }

        let Some((palette_id, _)) = state.iter_palettes().next() else {
            return Ok(());
        };
        let old_palette_id = (0..remap.palettes.old_len() as u32)
            .map(U32Id::<BVoxPalette>::from_u32)
            .find(|&old_id| remap.palettes.new_id(old_id) == Some(palette_id))
            .expect("a live palette was relabeled from an old id");
        let material_remap = &remap.materials[old_palette_id.to_usize_id()];

        let materials = mem::take(&mut self.materials);
        for (old_id, entry) in materials {
            let material_id = material_remap
                .new_id(old_id)
                .ok_or_else(|| stale("material", old_id.to_u32()))?;
            self.materials.insert(material_id, entry);
        }

        if let Some(index_map) = &mut self.index_map {
            let mut relabeled: Vec<Option<u8>> = index_map
                .iter()
                .map(|&old| {
                    material_remap
                        .new_id(U32Id::<BVoxMaterial>::from_u32(old as u32))
                        .map(|new_id| new_id.to_u32() as u8)
                })
                .collect();
            let taken: Vec<bool> = (0..=u8::MAX)
                .map(|index| relabeled.contains(&Some(index)))
                .collect();
            let mut freed = taken
                .iter()
                .enumerate()
                .filter(|(_, taken)| !**taken)
                .map(|(index, _)| index as u8);
            for slot in relabeled.iter_mut().filter(|slot| slot.is_none()) {
                *slot = Some(
                    freed
                        .next()
                        .expect("a byte map has one free index per stale byte"),
                );
            }
            *index_map = relabeled
                .into_iter()
                .map(|slot| slot.expect("every slot was filled"))
                .collect();
        }

        Ok(())
    }
}

/// The error for an entry keyed by an id the gc found dead.
fn stale(entity: &str, id: u32) -> Error {
    Error::Ext {
        reason: format!("mvox ext keeps an entry for {entity} {id}, which was released before gc"),
    }
}

#[cfg(test)]
mod tests {
    use crate::{MVoxExt, MVoxExtMaterial, MVoxExtNode, MVoxExtNodeBody, MVoxExtShapeModel};
    use branded_id::U32Id;
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxMain,
        VoxObject, VoxPalette, VoxValuePool,
    };

    fn node(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn object(index: u32) -> U32Id<BVoxObject> {
        U32Id::from_u32(index)
    }

    fn material(index: u32) -> U32Id<BVoxMaterial> {
        U32Id::from_u32(index)
    }

    /// A one-cell object.
    fn unit_object() -> VoxObject {
        VoxObject::new(String::new(), TyVector3U32::splat(1)).unwrap()
    }

    /// A retained node takes the kind its shape implies and a fresh id past
    /// the entries it joins.
    #[test]
    fn a_retained_node_takes_a_synthesized_entry_by_its_shape() {
        let mut main: VoxMain<MVoxExt> = VoxMain::default();

        let object_id = main.retain_object(unit_object()).unwrap();

        let shape_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();

        let transform_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                child_node_ids: vec![shape_id],
                transform: TyTransformF64::from_translation(TyVector3F64::new(1.4, -2.6, 0.0)),
                ..Default::default()
            })
            .unwrap();

        let group_id = main
            .retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        let ext = main.ext();

        assert_eq!(
            ext.scene_nodes[&shape_id],
            MVoxExtNode {
                id: 0,
                hidden: None,
                attr_extra: Vec::new(),
                body: MVoxExtNodeBody::Shape {
                    models: vec![MVoxExtShapeModel {
                        object: object_id,
                        frame_index: Some(0),
                        extra: Vec::new(),
                    }],
                },
            }
        );

        let MVoxExtNodeBody::Transform { layer, frames } = &ext.scene_nodes[&transform_id].body
        else {
            panic!("one child node makes a transform");
        };

        assert_eq!(*layer, -1);

        assert_eq!(frames[0].translation, [1, -3, 0]);

        assert_eq!(ext.scene_nodes[&transform_id].id, 1);

        assert_eq!(ext.scene_nodes[&group_id].body, MVoxExtNodeBody::Group);

        assert_eq!(ext.scene_nodes[&group_id].id, 2);
    }

    /// A released node drops its entry, and the next retained node takes an
    /// id past the ids still held.
    #[test]
    fn a_released_node_drops_its_entry() {
        let mut main: VoxMain<MVoxExt> = VoxMain::default();

        let first_id = main
            .retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        let second_id = main
            .retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        main.release_hierarchy_node(first_id).unwrap();

        assert_eq!(main.ext().scene_nodes.len(), 1);

        let third_id = main
            .retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        assert_eq!(main.ext().scene_nodes[&second_id].id, 1);

        assert_eq!(main.ext().scene_nodes[&third_id].id, 2);
    }

    /// A shape entry still drawing an object no node places is out of step,
    /// so the release is refused and the object stays.
    #[test]
    fn an_object_a_stale_shape_entry_draws_cannot_release() {
        let mut main: VoxMain<MVoxExt> = VoxMain::default();

        let object_id = main.retain_object(unit_object()).unwrap();

        main.ext_mut().scene_nodes.insert(
            node(7),
            MVoxExtNode {
                id: 3,
                hidden: None,
                attr_extra: Vec::new(),
                body: MVoxExtNodeBody::Shape {
                    models: vec![MVoxExtShapeModel {
                        object: object_id,
                        frame_index: None,
                        extra: Vec::new(),
                    }],
                },
            },
        );

        assert!(main.release_object(object_id).is_err());

        assert!(main.object(object_id).is_some());

        main.ext_mut().scene_nodes.clear();

        main.release_object(object_id).unwrap();
    }

    /// A palette with two materials, the second carrying an entry, and an
    /// index map.
    fn palette_main() -> (VoxMain<MVoxExt>, U32Id<BVoxPalette>) {
        let mut main: VoxMain<MVoxExt> = VoxMain::default();

        let value_pool_id = main.retain_value_pool(VoxValuePool::int(vec![0]).unwrap());

        let mut palette = VoxPalette::default();

        palette
            .retain_property("v".to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();

        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

        let palette_id = main.retain_palette(palette).unwrap();

        main.ext_mut().materials.insert(
            material(1),
            MVoxExtMaterial {
                weight: Some(0.5),
                ..Default::default()
            },
        );

        main.ext_mut().index_map = Some((0..=255).collect());

        (main, palette_id)
    }

    /// Releasing a material drops its entry. Releasing the palette drops
    /// every entry and the index map.
    #[test]
    fn released_materials_and_palettes_drop_their_entries() {
        let (mut main, palette_id) = palette_main();

        main.release_material(palette_id, material(1)).unwrap();

        assert!(main.ext().materials.is_empty());

        assert!(main.ext().index_map.is_some());

        main.release_palette(palette_id).unwrap();

        assert!(main.ext().index_map.is_none());
    }

    /// `gc` refuses an entry keyed by an id it found dead, since the ext was
    /// out of step before the gc.
    #[test]
    fn gc_refuses_an_entry_for_a_released_entity() {
        let (mut main, _) = palette_main();

        main.ext_mut()
            .materials
            .insert(material(9), MVoxExtMaterial::default());

        assert!(main.gc().is_err());

        let mut main: VoxMain<MVoxExt> = VoxMain::default();

        main.ext_mut().scene_nodes.insert(
            node(4),
            MVoxExtNode {
                id: 0,
                hidden: None,
                attr_extra: Vec::new(),
                body: MVoxExtNodeBody::Group,
            },
        );

        assert!(main.gc().is_err());

        let mut main: VoxMain<MVoxExt> = VoxMain::default();

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        let MVoxExtNodeBody::Group = main.ext().scene_nodes[&node_id].body else {
            panic!("a node with no children is a group");
        };

        main.ext_mut().scene_nodes.get_mut(&node_id).unwrap().body = MVoxExtNodeBody::Shape {
            models: vec![MVoxExtShapeModel {
                object: object(2),
                frame_index: None,
                extra: Vec::new(),
            }],
        };

        assert!(main.gc().is_err());
    }

    #[cfg(feature = "serde")]
    mod serde {
        use super::*;
        use serde_json::{from_str, to_string};

        /// The maps serialize keyed by bare id and read back.
        #[test]
        fn round_trips_the_keyed_entries_through_serde() {
            let (main, _) = palette_main();

            let json = to_string(main.ext()).unwrap();

            assert!(json.contains("\"materials\":{\"1\":{"));

            let reloaded: MVoxExt = from_str(&json).unwrap();

            assert_eq!(&reloaded, main.ext());
        }
    }
}
