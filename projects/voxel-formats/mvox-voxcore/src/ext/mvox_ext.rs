use crate::{
    MVoxExtCamera, MVoxExtFrame, MVoxExtLayer, MVoxExtMaterial, MVoxExtNode, MVoxExtNodeBody,
    MVoxExtUnknownChunk, SceneNodeKind, frame_translation, insert_synthesized_scene_node,
    synthesized_node_body, synthesized_shape_model,
};
use branded_id::U32Id;
use mvox::MVoxRotation;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, mem};
use ty_math::{TyMatrix4x4F64, TyTransformF64};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, Error, Result, VoxExt, VoxGcRemap,
    VoxHierarchyNode, VoxState,
};

/// How far a turned axis may sit from a whole `-1`, `0`, or `1` and still
/// read as one.
const SIGNED_TOLERANCE: f64 = 1e-6;

/// The `mvox` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// MagicaVoxel `.vox` state with no native voxcore home, kept so a file loaded
/// from a MagicaVoxel package can be written back exactly.
///
/// Geometry, colors, and the scene graph become native voxcore entities. This
/// holds the rest. The scene nodes are keyed by hierarchy node and the
/// materials by material of palette `palette_id`. Both follow the state through
/// the [`VoxExt`] hooks.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MVoxExt {
    /// The format version from the header (`version`).
    pub version: u32,

    /// Whether the file carried an `RGBA` palette chunk. When false the colors
    /// came from the built-in palette and no chunk is written back.
    #[cfg_attr(feature = "serde", serde(rename = "palette-present"))]
    pub palette_present: bool,

    /// The palette `materials` and `index_map` describe, or `None` once that
    /// palette is released. Releasing another palette or its materials leaves
    /// the entries alone.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "palette-id",
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub palette_id: Option<U32Id<BVoxPalette>>,

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

/// A node keeps its kind while its children still fit it, and otherwise takes
/// the kind a retained node would. A transform node's first frame follows the
/// node's transform. A transform no frame can hold, such as a position off the
/// voxel grid, is left for the writer to report.
///
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
        insert_synthesized_scene_node(
            &mut self.scene_nodes,
            node_id,
            scene_node_kind_of(node),
            node,
        );
        Ok(())
    }

    fn hierarchy_node_transform_did_set(
        &mut self,
        state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        _old_transform: TyTransformF64,
    ) -> Result<()> {
        let node = state.hierarchy_node(node_id).expect("a set node is live");
        let MVoxExtNodeBody::Transform { frames, .. } = &mut scene_node_mut(self, node_id)?.body
        else {
            return Ok(());
        };

        if frames.is_empty() {
            frames.push(MVoxExtFrame {
                rotation: MVoxRotation::IDENTITY.0,
                ..Default::default()
            });
        }

        let frame = &mut frames[0];
        frame.translation = frame_translation(node.transform.position);
        if let Some(rotation) = frame_rotation(&node.transform) {
            frame.rotation = rotation;
        }
        Ok(())
    }

    fn hierarchy_node_children_did_set(
        &mut self,
        state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        _old_child_node_ids: &[U32Id<BVoxHierarchyNode>],
        _old_child_object_ids: &[U32Id<BVoxObject>],
    ) -> Result<()> {
        let node = state.hierarchy_node(node_id).expect("a set node is live");
        let entry = scene_node_mut(self, node_id)?;
        let fits = match &entry.body {
            MVoxExtNodeBody::Transform { .. } => {
                node.child_node_ids.len() == 1 && node.child_object_ids.is_empty()
            }

            MVoxExtNodeBody::Group => node.child_object_ids.is_empty(),

            MVoxExtNodeBody::Shape { .. } => node.child_node_ids.is_empty(),
        };
        if !fits {
            entry.body = synthesized_node_body(scene_node_kind_of(node), node);
            return Ok(());
        }

        if let MVoxExtNodeBody::Shape { models } = &mut entry.body {
            models.retain(|model| node.child_object_ids.contains(&model.object));
            for &object_id in &node.child_object_ids {
                if !models.iter().any(|model| model.object == object_id) {
                    models.push(synthesized_shape_model(object_id));
                }
            }
        }
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
        palette_id: U32Id<BVoxPalette>,
    ) -> Result<()> {
        if self.palette_id != Some(palette_id) {
            return Ok(());
        }

        self.palette_id = None;

        self.materials.clear();

        self.index_map = None;

        Ok(())
    }

    fn materials_will_release(
        &mut self,
        _state: &VoxState,
        palette_id: U32Id<BVoxPalette>,
        material_ids: &[U32Id<BVoxMaterial>],
    ) -> Result<()> {
        if self.palette_id != Some(palette_id) {
            return Ok(());
        }

        for material_id in material_ids {
            self.materials.remove(material_id);
        }

        Ok(())
    }

    fn did_gc(&mut self, _state: &VoxState, remap: &VoxGcRemap) -> Result<()> {
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

        let Some(old_palette_id) = self.palette_id else {
            return check_no_palette_entries(self);
        };

        let palette_id = remap
            .palettes
            .new_id(old_palette_id)
            .ok_or_else(|| stale("palette", old_palette_id.to_u32()))?;

        self.palette_id = Some(palette_id);

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

/// Errors when `ext` keeps material entries or an index map with no palette
/// for them to describe.
fn check_no_palette_entries(ext: &MVoxExt) -> Result<()> {
    if ext.materials.is_empty() && ext.index_map.is_none() {
        return Ok(());
    }

    Err(Error::Ext {
        reason: "mvox ext keeps material entries or an index map but no palette".to_owned(),
    })
}

fn scene_node_mut(
    ext: &mut MVoxExt,
    node_id: U32Id<BVoxHierarchyNode>,
) -> Result<&mut MVoxExtNode> {
    ext.scene_nodes.get_mut(&node_id).ok_or_else(|| Error::Ext {
        reason: format!(
            "mvox ext has no scene node for hierarchy node {}",
            node_id.to_u32()
        ),
    })
}

/// The error for an entry keyed by an id the gc found dead.
fn stale(entity: &str, id: u32) -> Error {
    Error::Ext {
        reason: format!("mvox ext keeps an entry for {entity} {id}, which was released before gc"),
    }
}

/// The packed rotation byte of a node's rotation and scale on MagicaVoxel's
/// Z-up axes, or `None` when they are not a signed permutation.
fn frame_rotation(transform: &TyTransformF64) -> Option<u8> {
    let turned = transform.yup_to_zup();
    let matrix = TyMatrix4x4F64::from_scale_rotation_translation(
        turned.scale,
        turned.rotation,
        Default::default(),
    );

    let mut signed = [[0i8; 3]; 3];
    for (row, entries) in signed.iter_mut().enumerate() {
        for (column, entry) in entries.iter_mut().enumerate() {
            let value = matrix.col(column)[row];
            let rounded = value.round();
            if (value - rounded).abs() > SIGNED_TOLERANCE || rounded.abs() > 1.0 {
                return None;
            }
            *entry = rounded as i8;
        }
    }

    MVoxRotation::from_matrix(signed).map(|rotation| rotation.0)
}

/// The kind a synthesized scene node takes from `node`'s children.
fn scene_node_kind_of(node: &VoxHierarchyNode) -> SceneNodeKind {
    if !node.child_object_ids.is_empty() {
        SceneNodeKind::Shape
    } else if node.child_node_ids.len() == 1 {
        SceneNodeKind::Transform
    } else {
        SceneNodeKind::Group
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        MVoxExt, MVoxExtMaterial, MVoxExtNode, MVoxExtNodeBody, MVoxExtShapeModel, material_id,
        node_id, object_id,
    };
    use branded_id::U32Id;
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool};

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

        // The translation turns to MagicaVoxel's Z-up axes before rounding.
        assert_eq!(frames[0].translation, [1, 0, -3]);

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
            node_id(7),
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

        main.ext_mut().palette_id = Some(palette_id);

        main.ext_mut().materials.insert(
            material_id(1),
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

        main.release_material(palette_id, material_id(1)).unwrap();

        assert!(main.ext().materials.is_empty());

        assert!(main.ext().index_map.is_some());

        main.release_palette(palette_id).unwrap();

        assert!(main.ext().index_map.is_none());
    }

    /// Releasing another palette's material or that palette leaves the
    /// entries alone, even where the material ids match.
    #[test]
    fn another_palettes_releases_leave_the_entries() {
        let (mut main, _) = palette_main();

        let ext = main.ext().clone();

        let mut other = VoxPalette::default();

        other.retain_material(Vec::new()).unwrap();

        let other_material_id = other.retain_material(Vec::new()).unwrap();

        assert!(ext.materials.contains_key(&other_material_id));

        let other_id = main.retain_palette(other).unwrap();

        main.release_material(other_id, other_material_id).unwrap();

        main.release_palette(other_id).unwrap();

        assert_eq!(main.ext(), &ext);
    }

    /// `gc` moves the entries' palette to its compacted id.
    #[test]
    fn gc_relabels_the_entries_palette() {
        let mut main: VoxMain<MVoxExt> = VoxMain::default();

        let spare_id = main.retain_palette(VoxPalette::default()).unwrap();

        let palette_id = main.retain_palette(VoxPalette::default()).unwrap();

        main.ext_mut().palette_id = Some(palette_id);

        main.release_palette(spare_id).unwrap();

        let remap = main.gc().unwrap();

        assert_eq!(main.ext().palette_id, remap.palettes.new_id(palette_id));
    }

    /// `gc` refuses entries with no palette to describe.
    #[test]
    fn gc_refuses_entries_without_a_palette() {
        let (mut main, _) = palette_main();

        main.ext_mut().palette_id = None;

        assert!(main.gc().is_err());
    }

    /// `gc` refuses an entry keyed by an id it found dead, since the ext was
    /// out of step before the gc.
    #[test]
    fn gc_refuses_an_entry_for_a_released_entity() {
        let (mut main, _) = palette_main();

        main.ext_mut()
            .materials
            .insert(material_id(9), MVoxExtMaterial::default());

        assert!(main.gc().is_err());

        let mut main: VoxMain<MVoxExt> = VoxMain::default();

        main.ext_mut().scene_nodes.insert(
            node_id(4),
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
                object: object_id(2),
                frame_index: None,
                extra: Vec::new(),
            }],
        };

        assert!(main.gc().is_err());
    }

    #[cfg(feature = "serde")]
    mod serde {
        use crate::{MVoxExt, ext::mvox_ext::tests::palette_main};
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
