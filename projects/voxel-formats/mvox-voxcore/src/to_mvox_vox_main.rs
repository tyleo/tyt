use crate::{
    Error, MVoxExt, MVoxExtFrame, MVoxExtNode, MVoxExtNodeBody, MVoxExtShapeModel, MVoxVoxMain,
    Result,
};
use branded_id::U32Id;
use mvox::MVoxRotation;
use std::collections::{HashMap, HashSet};
use ty_math::{
    TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32,
};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxVoxel, VoxHierarchyNode, VoxMain, VoxPalette,
    VoxValuePool,
    color::{lin_srgba_f64_from_srgba_u8, resolve_cell_color, resolve_cell_color_or_transparent},
    material::BASE_COLOR,
};

/// The format version a synthesized file carries.
const VERSION: u32 = 150;

/// Colors a MagicaVoxel palette holds. Slot 0 is the reserved empty color.
const PALETTE_COLORS: usize = 256;

/// Voxels per axis a model can hold, since a voxel coordinate is a byte.
const MODEL_AXIS_LIMIT: u32 = 256;

/// An object and the merged-palette slot each of its live voxels samples.
type ObjectSlots = (U32Id<BVoxObject>, Vec<(U32Id<BVoxVoxel>, u32)>);

/// Gives a bare state a synthesized [`MVoxExt`], the state
/// [`to_mvox_file`](crate::to_mvox_file) writes as a file synthesized from
/// the scene. The state takes MagicaVoxel's shape first. Every palette merges
/// into one 256-color table, each object sampling the slot of its color on
/// one layer. The hierarchy rebuilds with one node per scene node: every node
/// becomes a transform node carrying its translation over a group of its
/// child nodes and one transform-over-shape placement per object, all under
/// one synthetic root. Each node then takes its entry, and the ext takes the
/// synthesized version and no materials, layers, cameras, or notes.
///
/// Lossy where MagicaVoxel cannot represent the source. Node rotation and
/// scale drop because a scene transform carries only translation. A node's
/// name and an object's name ride their transform nodes as `_name`.
///
/// Errors when an object grid exceeds 256 voxels per axis, past which the
/// byte voxel coordinates cannot address it, or when the objects use more
/// than 255 distinct colors, past which the palette is full.
pub fn to_mvox_vox_main(mut state: VoxMain<()>) -> Result<MVoxVoxMain> {
    check_grids(&state)?;

    merge_palettes(&mut state)?;

    let scene_nodes = rebuild_hierarchy(&mut state)?;

    let ext = MVoxExt {
        version: VERSION,
        palette_present: true,
        scene_nodes,
        ..Default::default()
    };

    Ok(state.put_ext(ext))
}

/// Errors on a grid past the per-axis limit.
fn check_grids(state: &VoxMain<()>) -> Result<()> {
    for (_, object) in state.iter_objects() {
        let bounds = object.bounds();
        if bounds.x > MODEL_AXIS_LIMIT || bounds.y > MODEL_AXIS_LIMIT || bounds.z > MODEL_AXIS_LIMIT
        {
            return Err(Error::Invalid(format!(
                "object grid {}x{}x{} exceeds MagicaVoxel's limit of {MODEL_AXIS_LIMIT} voxels \
                 per axis",
                bounds.x, bounds.y, bounds.z
            )));
        }
    }

    Ok(())
}

/// Replaces every palette with one holding each distinct color the objects
/// use, in first-seen order, material `k` at color `k`. Each object keeps
/// one layer on the merged palette, sampling the material of its color, and
/// keeps its id. The old palettes release. Their value pools stay for the
/// caller's prune.
fn merge_palettes(state: &mut VoxMain<()>) -> Result<()> {
    let mut colors: Vec<[u8; 4]> = vec![[0, 0, 0, 0]];
    let mut index_of: HashMap<[u8; 4], u32> = HashMap::new();
    for (_, object) in state.iter_objects() {
        let Some(cell_color) = resolve_cell_color(state, object)? else {
            continue;
        };
        for voxel_id in object.iter_live() {
            let rgba = cell_color.color(voxel_id);
            if index_of.contains_key(&rgba) {
                continue;
            }
            if colors.len() == PALETTE_COLORS {
                return Err(Error::Invalid(format!(
                    "the objects use more than {} distinct colors, past which a \
                     MagicaVoxel palette is full",
                    PALETTE_COLORS - 1
                )));
            }
            index_of.insert(rgba, colors.len() as u32);
            colors.push(rgba);
        }
    }

    // Each voxel's slot, read while the old palettes still resolve. A
    // colorless object samples the empty color at slot 0. When a colored
    // object used transparent, that color's slot serves instead.
    let mut samples: Vec<ObjectSlots> = Vec::new();
    for (object_id, object) in state.iter_objects() {
        let cell_color = resolve_cell_color_or_transparent(state, object)?;
        let object_samples = object
            .iter_live()
            .map(|voxel_id| {
                let slot = index_of
                    .get(&cell_color.color(voxel_id))
                    .copied()
                    .unwrap_or(0);
                (voxel_id, slot)
            })
            .collect();
        samples.push((object_id, object_samples));
    }

    for (object_id, _) in &samples {
        let layer_ids: Vec<_> = state
            .object(*object_id)
            .expect("a listed object")
            .iter_layers()
            .map(|(layer_id, _)| layer_id)
            .collect();
        for layer_id in layer_ids {
            state.release_layer(*object_id, layer_id)?;
        }
    }

    let palette_ids: Vec<_> = state
        .iter_palettes()
        .map(|(palette_id, _)| palette_id)
        .collect();
    for palette_id in palette_ids {
        state.release_palette(palette_id)?;
    }

    let value_pool_id = state.retain_value_pool(
        VoxValuePool::vec_4_float(
            colors
                .iter()
                .map(|&color| <[f64; 4]>::from(lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(color))))
                .collect(),
        )
        .expect("byte-derived components are finite and slot 0 is present"),
    );
    let mut palette = VoxPalette::default();
    palette
        .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
        .expect("the one property name is distinct");
    for index in 0..colors.len() {
        palette
            .retain_material(vec![U32Id::from_u32(index as u32)])
            .expect("one value id for the one property");
    }
    let palette_id = state.retain_palette(palette)?;

    for (object_id, object_samples) in samples {
        state.retain_layer(object_id, palette_id, U32Id::from_u32(0))?;
        for (voxel_id, slot) in object_samples {
            state.retain_voxel(
                object_id,
                voxel_id,
                &[U32Id::<BVoxMaterial>::from_u32(slot)],
            )?;
        }
    }

    Ok(())
}

/// Rebuilds the hierarchy in MagicaVoxel's shape, one node per scene node,
/// and returns the entries aligned with the new listing. Every node becomes
/// a transform node carrying its translation over a group of its child nodes
/// and one transform-over-shape placement per object. All roots hang under
/// one synthetic root, the single root MagicaVoxel requires. An object no
/// node places hangs there too, at the origin, so geometry is never dropped.
/// A node reached along several paths is rebuilt per path, and a node no root
/// reaches drops. Entry ids count up in emission order, parents before
/// children.
fn rebuild_hierarchy(state: &mut VoxMain<()>) -> Result<Vec<Option<MVoxExtNode>>> {
    let old: HashMap<U32Id<BVoxHierarchyNode>, VoxHierarchyNode> = state
        .iter_hierarchy_nodes()
        .map(|(node_id, node)| (node_id, node.clone()))
        .collect();
    let root_ids = state.root_hierarchy_node_ids().to_vec();
    let object_indices: HashMap<U32Id<BVoxObject>, u32> = state
        .iter_objects()
        .enumerate()
        .map(|(index, (object_id, _))| (object_id, index as u32))
        .collect();

    let mut builder = Builder {
        state,
        old: &old,
        object_indices,
        ids: Vec::new(),
        entries: Vec::new(),
        placed: HashSet::new(),
    };
    let root_transform = builder.allocate(transform_vox_node([0, 0, 0], ""))?;
    let root_group = builder.allocate(VoxHierarchyNode::default())?;
    let mut children = Vec::new();
    for root_id in root_ids {
        children.push(builder.emit_node(root_id)?);
    }
    let unplaced: Vec<_> = builder
        .state
        .iter_objects()
        .filter(|(object_id, _)| !builder.placed.contains(object_id))
        .map(|(object_id, object)| (object_id, object.bounds()))
        .collect();
    for (object_id, bounds) in unplaced {
        children.push(builder.emit_object(object_id, bounds)?);
    }
    builder.link(
        root_transform,
        &[root_group],
        transform_entry(root_transform, [0, 0, 0], root_group, None),
    )?;
    builder.link(
        root_group,
        &children,
        group_entry(root_group, children.clone()),
    )?;
    let Builder { ids, entries, .. } = builder;

    // The old nodes release once nothing lists them.
    state.set_root_hierarchy_node_ids(vec![ids[root_transform as usize]])?;
    for (&old_id, node) in &old {
        let mut node = node.clone();
        node.child_node_ids.clear();
        state.set_hierarchy_node(old_id, node)?;
    }
    for &old_id in old.keys() {
        state.release_hierarchy_node(old_id)?;
    }

    Ok(entries
        .into_iter()
        .map(|entry| Some(entry.expect("every emitted node is linked")))
        .collect())
}

/// Retains the rebuilt nodes in emission order, parents before children, so
/// the listing follows the scene-node ids.
struct Builder<'a> {
    state: &'a mut VoxMain<()>,
    old: &'a HashMap<U32Id<BVoxHierarchyNode>, VoxHierarchyNode>,
    object_indices: HashMap<U32Id<BVoxObject>, u32>,
    ids: Vec<U32Id<BVoxHierarchyNode>>,
    entries: Vec<Option<MVoxExtNode>>,
    placed: HashSet<U32Id<BVoxObject>>,
}

impl<'a> Builder<'a> {
    /// Retains `node` with no child nodes yet and returns its scene-node id,
    /// its index in emission order.
    fn allocate(&mut self, node: VoxHierarchyNode) -> Result<i32> {
        let node_id = self.state.retain_hierarchy_node(node)?;
        self.ids.push(node_id);
        self.entries.push(None);
        Ok((self.ids.len() - 1) as i32)
    }

    /// Links the node with scene-node id `id` to its child nodes and records
    /// its entry.
    fn link(&mut self, id: i32, children: &[i32], entry: MVoxExtNode) -> Result<()> {
        let node_id = self.ids[id as usize];
        let mut node = self
            .state
            .hierarchy_node(node_id)
            .expect("an allocated node is listed")
            .clone();
        node.child_node_ids = children
            .iter()
            .map(|&child| self.ids[child as usize])
            .collect();
        self.state.set_hierarchy_node(node_id, node)?;
        self.entries[id as usize] = Some(entry);
        Ok(())
    }

    /// Emits the transform over group for the old node `old_id` and its whole
    /// subtree, returning the transform's scene-node id for a parent group to
    /// list.
    fn emit_node(&mut self, old_id: U32Id<BVoxHierarchyNode>) -> Result<i32> {
        let old: &'a HashMap<_, _> = self.old;
        let node = old.get(&old_id).expect("a root or child is a listed node");
        let translation = translation_of(&node.transform.position);
        let name = named(&node.name);
        let transform = self.allocate(transform_vox_node(translation, &node.name))?;
        let group = self.allocate(VoxHierarchyNode::default())?;

        let mut children = Vec::new();
        for &child_id in &node.child_node_ids {
            children.push(self.emit_node(child_id)?);
        }
        for &object_id in &node.child_object_ids {
            let bounds = self
                .state
                .object(object_id)
                .expect("a placed object is listed")
                .bounds();
            children.push(self.emit_object(object_id, bounds)?);
        }

        self.link(
            transform,
            &[group],
            transform_entry(transform, translation, group, name),
        )?;
        self.link(group, &children, group_entry(group, children.clone()))?;
        Ok(transform)
    }

    /// Emits the transform over shape placing one object, offsetting the
    /// transform by the model's pivot so MagicaVoxel centers it where
    /// voxcore's min corner sits.
    fn emit_object(&mut self, object_id: U32Id<BVoxObject>, bounds: TyVector3U32) -> Result<i32> {
        self.placed.insert(object_id);
        let pivot = (bounds / 2).as_ivec3().to_array();
        let name = self
            .state
            .object(object_id)
            .expect("a placed object is listed")
            .name()
            .to_owned();
        let transform = self.allocate(transform_vox_node(pivot, &name))?;
        let shape = self.allocate(VoxHierarchyNode {
            child_object_ids: vec![object_id],
            ..Default::default()
        })?;

        self.link(
            transform,
            &[shape],
            transform_entry(transform, pivot, shape, named(&name)),
        )?;
        self.link(
            shape,
            &[],
            shape_entry(shape, self.object_indices[&object_id]),
        )?;
        Ok(transform)
    }
}

/// A node named `name` at `translation` with no children yet.
fn transform_vox_node(translation: [i32; 3], name: &str) -> VoxHierarchyNode {
    VoxHierarchyNode {
        name: name.to_owned(),
        transform: TyTransformF64::new(
            TyVector3I32::from_array(translation).as_dvec3(),
            TyQuaternionF64::IDENTITY,
            TyVector3F64::ONE,
        ),
        ..Default::default()
    }
}

/// The `_name` attribute for `name`. An empty name gets none.
fn named(name: &str) -> Option<String> {
    (!name.is_empty()).then(|| name.to_owned())
}

/// The entry of an identity-rotation transform node at `translation` over
/// its single child, named `name`.
fn transform_entry(
    id: i32,
    translation: [i32; 3],
    child: i32,
    name: Option<String>,
) -> MVoxExtNode {
    MVoxExtNode {
        id,
        name,
        hidden: None,
        attr_extra: Vec::new(),
        body: MVoxExtNodeBody::Transform {
            child,
            layer: -1,
            frames: vec![MVoxExtFrame {
                rotation: MVoxRotation::IDENTITY.0,
                translation,
                frame_index: None,
                extra: Vec::new(),
            }],
        },
    }
}

/// The entry of a group node over `children`.
fn group_entry(id: i32, children: Vec<i32>) -> MVoxExtNode {
    MVoxExtNode {
        id,
        name: None,
        hidden: None,
        attr_extra: Vec::new(),
        body: MVoxExtNodeBody::Group { children },
    }
}

/// The entry of a shape node drawing `model` on its first frame.
fn shape_entry(id: i32, model: u32) -> MVoxExtNode {
    MVoxExtNode {
        id,
        name: None,
        hidden: None,
        attr_extra: Vec::new(),
        body: MVoxExtNodeBody::Shape {
            models: vec![MVoxExtShapeModel {
                model,
                frame_index: Some(0),
                extra: Vec::new(),
            }],
        },
    }
}

/// One scene-frame translation rounded from a node's local position.
fn translation_of(position: &TyVector3F64) -> [i32; 3] {
    position.round().as_ivec3().to_array()
}

#[cfg(test)]
mod tests {
    use crate::{MVoxExtNodeBody, from_mvox_file, to_mvox_file, to_mvox_vox_main};
    use branded_id::U32Id;
    use mvox::{MVoxModel, MVoxPalette};
    use std::collections::BTreeSet;
    use ty_math::{
        TyHexColor, TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3U32,
    };
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxMain,
        VoxObject, VoxPalette, VoxValuePool,
        color::{lin_srgba_f64_from_srgba_u8, resolve_cell_color},
        material::BASE_COLOR,
    };

    /// The linear-light components of a `#RRGGBBAA` hex string.
    fn linear_rgba(hex: &str) -> [f64; 4] {
        lin_srgba_f64_from_srgba_u8(TySrgbaU8::from_hex(hex).expect("a valid hex color")).into()
    }

    /// A resolved voxel: `x`, `y`, `z`, and an `rgba` color.
    type ResolvedVoxel = (u8, u8, u8, (u8, u8, u8, u8));

    /// A model's voxels as `(x, y, z, (r, g, b, a))`, order-independent so a
    /// synthesized model compares without depending on raster order.
    fn resolved_voxels(model: &MVoxModel, palette: &MVoxPalette) -> BTreeSet<ResolvedVoxel> {
        model
            .voxels
            .iter()
            .map(|voxel| {
                let color = palette.colors[voxel.color_index as usize];
                (
                    voxel.x,
                    voxel.y,
                    voxel.z,
                    (color.r, color.g, color.b, color.a),
                )
            })
            .collect()
    }

    /// A bare state built straight from voxcore: a red-green object and a
    /// blue object sharing one `rgba` palette, placed by a hierarchy of a
    /// nested group and two roots. This is the cross-format synthesis input.
    fn source_state() -> VoxMain<()> {
        let mut state = VoxMain::default();

        // One baseColor palette: a transparent placeholder, then red,
        // green, blue.
        let value_pool_id = state.retain_value_pool(
            VoxValuePool::vec_4_float(
                ["#00000000", "#FF0000FF", "#00FF00FF", "#0000FFFF"]
                    .iter()
                    .map(|hex| linear_rgba(hex))
                    .collect(),
            )
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        for index in 0..4 {
            palette
                .retain_material(vec![U32Id::from_u32(index)])
                .expect("one value-index for the one binding");
        }
        let palette_id = state.retain_palette(palette).unwrap();
        let material_id = |index: u32| U32Id::<BVoxMaterial>::from_u32(index);

        // Object 0: a red then a green voxel along x.
        let mut wide = VoxObject::new(String::new(), TyVector3U32::new(2, 1, 1))
            .expect("a 2x1x1 grid is within the dense limit");
        wide.retain_layer(palette_id, material_id(0));
        for (x, material_index) in [(0u32, 1u32), (1, 2)] {
            let voxel_id = wide
                .voxel_id(TyVector3U32::new(x, 0, 0))
                .expect("a position within the grid");
            wide.retain_voxel(voxel_id, &[material_id(material_index)])
                .expect("one sample for the one layer");
        }
        state.retain_object(wide).unwrap();

        // Object 1: a single blue voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.retain_layer(palette_id, material_id(0));
        let voxel_id = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel_id, &[material_id(3)])
            .expect("one sample for the one layer");
        state.retain_object(unit).unwrap();

        let object_id = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node_id = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at = |x: f64, y: f64, z: f64| {
            TyTransformF64::new(
                TyVector3F64::new(x, y, z),
                TyQuaternionF64::IDENTITY,
                TyVector3F64::ONE,
            )
        };

        // node 0 groups node 1, which places object 0 at +5x; node 2 places
        // object 1 at +3y. Nodes 0 and 2 are the roots.
        state
            .retain_hierarchy_nodes(vec![
                VoxHierarchyNode {
                    name: "group".to_owned(),
                    child_node_ids: vec![node_id(1)],
                    child_object_ids: Vec::new(),
                    transform: TyTransformF64::default(),
                },
                VoxHierarchyNode {
                    name: "wide".to_owned(),
                    child_node_ids: Vec::new(),
                    child_object_ids: vec![object_id(0)],
                    transform: placed_at(5.0, 0.0, 0.0),
                },
                VoxHierarchyNode {
                    name: "unit".to_owned(),
                    child_node_ids: Vec::new(),
                    child_object_ids: vec![object_id(1)],
                    transform: placed_at(0.0, 3.0, 0.0),
                },
            ])
            .unwrap();
        state
            .set_root_hierarchy_node_ids(vec![node_id(0), node_id(2)])
            .unwrap();

        state.validate().expect("a well-formed source state");
        state
    }

    /// A default state has no objects, so the synthesized file is empty.
    #[test]
    fn synthesizes_an_empty_state() {
        let state = to_mvox_vox_main(VoxMain::default()).unwrap();

        let file = to_mvox_file(&state).unwrap();

        assert!(file.models.is_empty());

        // The synthetic root alone.
        assert_eq!(file.scene_nodes.len(), 2);
    }

    /// A bare state, such as one cross-loaded from another format,
    /// synthesizes a file: one model per object, a global palette gathering
    /// every used color, and a scene graph the decoder reads back.
    #[test]
    fn synthesizes_a_file_from_a_bare_state() {
        let state = to_mvox_vox_main(source_state()).unwrap();
        let file = to_mvox_file(&state).unwrap();
        let palette = file.palette.as_ref().expect("synthesis writes a palette");

        let red = (0xFF, 0, 0, 0xFF);
        let green = (0, 0xFF, 0, 0xFF);
        let blue = (0, 0, 0xFF, 0xFF);

        assert_eq!(file.models.len(), 2);
        assert_eq!(file.models[0].size, [2, 1, 1]);
        assert_eq!(
            resolved_voxels(&file.models[0], palette),
            BTreeSet::from([(0, 0, 0, red), (1, 0, 0, green)])
        );
        assert_eq!(file.models[1].size, [1, 1, 1]);
        assert_eq!(
            resolved_voxels(&file.models[1], palette),
            BTreeSet::from([(0, 0, 0, blue)])
        );

        // The synthesized palette and scene graph read back into a valid state.
        let reloaded = from_mvox_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 2);
    }

    /// The ext holds one entry per rebuilt node, ids counting up in emission
    /// order: the synthetic root, then each node's transform and group, then
    /// each placement's transform and shape.
    #[test]
    fn synthesizes_an_entry_per_scene_node() {
        let state = to_mvox_vox_main(source_state()).unwrap();

        let ext = state.ext();

        assert_eq!(ext.version, 150);

        assert!(ext.palette_present);

        assert!(ext.materials.is_empty());

        assert_eq!(state.hierarchy_node_count(), 12);

        assert_eq!(ext.scene_nodes.len(), 12);

        let entry = |index: usize| {
            ext.scene_nodes[index]
                .as_ref()
                .expect("a synthesized entry")
        };

        for index in 0..12 {
            assert_eq!(entry(index).id, index as i32);
        }

        // A node's name rides its transform. The root, the groups, the
        // placements of nameless objects, and the shapes carry none.
        let names: Vec<_> = (0..12).map(|index| entry(index).name.as_deref()).collect();

        assert_eq!(
            names,
            [
                None,
                None,
                Some("group"),
                None,
                Some("wide"),
                None,
                None,
                None,
                Some("unit"),
                None,
                None,
                None,
            ]
        );

        let (_, transform) = state.iter_hierarchy_nodes().nth(2).unwrap();

        assert_eq!(transform.name, "group");

        assert!(matches!(
            entry(0).body,
            MVoxExtNodeBody::Transform { child: 1, .. }
        ));

        assert_eq!(
            entry(1).body,
            MVoxExtNodeBody::Group {
                children: vec![2, 8]
            }
        );

        // Node "wide" at +5x places object 0, whose pivot is +1x.
        let MVoxExtNodeBody::Transform { child, frames, .. } = &entry(4).body else {
            panic!("a transform entry");
        };

        assert_eq!(*child, 5);

        assert_eq!(frames[0].translation, [5, 0, 0]);

        let MVoxExtNodeBody::Transform { frames, .. } = &entry(6).body else {
            panic!("a transform entry");
        };

        assert_eq!(frames[0].translation, [1, 0, 0]);

        let MVoxExtNodeBody::Shape { models } = &entry(7).body else {
            panic!("a shape entry");
        };

        assert_eq!(models[0].model, 0);

        let MVoxExtNodeBody::Shape { models } = &entry(11).body else {
            panic!("a shape entry");
        };

        assert_eq!(models[0].model, 1);

        // The rebuilt hierarchy mirrors the entries.
        let root = state.root_hierarchy_node_ids();

        assert_eq!(root.len(), 1);

        let (_, group) = state.iter_hierarchy_nodes().nth(1).unwrap();

        assert_eq!(group.child_node_ids.len(), 2);
    }

    /// Two palettes merge into one whose materials are the distinct colors
    /// in first-seen order after the empty slot. Every voxel samples the slot
    /// of its color, and the objects keep their ids and colors.
    #[test]
    fn merges_palettes_into_one_with_samples_remapped() {
        let mut state = VoxMain::default();

        let red_palette_id = single_color_palette(&mut state, "#FF0000FF");

        let blue_palette_id = single_color_palette(&mut state, "#0000FFFF");

        let red_id = state.retain_object(unit_object(red_palette_id)).unwrap();

        let blue_id = state.retain_object(unit_object(blue_palette_id)).unwrap();

        let before: Vec<[u8; 4]> = [red_id, blue_id]
            .iter()
            .map(|&object_id| color_of(&state, object_id))
            .collect();

        let state = to_mvox_vox_main(state).unwrap();

        assert_eq!(state.palette_count(), 1);

        let (palette_id, palette) = state.iter_palettes().next().unwrap();

        assert_eq!(palette.material_count(), 3);

        assert_eq!(state.object_count(), 2);

        for (object_id, slot) in [(red_id, 1), (blue_id, 2)] {
            let object = state.object(object_id).unwrap();

            let (layer_id, layer_palette_id) = object.iter_layers().next().unwrap();

            assert_eq!(object.layer_count(), 1);

            assert_eq!(layer_palette_id, palette_id);

            let voxel_id = object.voxel_id(TyVector3U32::ZERO).unwrap();

            assert_eq!(
                object.voxel_material(voxel_id, layer_id),
                Some(U32Id::<BVoxMaterial>::from_u32(slot))
            );
        }

        let after: Vec<[u8; 4]> = [red_id, blue_id]
            .iter()
            .map(|&object_id| color_of(&state, object_id))
            .collect();

        assert_eq!(after, before);

        let file = to_mvox_file(&state).unwrap();

        let colors = file.palette.unwrap().colors;

        assert_eq!((colors[1].r, colors[1].g, colors[1].b), (0xFF, 0, 0));

        assert_eq!((colors[2].r, colors[2].g, colors[2].b), (0, 0, 0xFF));
    }

    /// An object no node places hangs under the synthetic root, so geometry
    /// is never dropped.
    #[test]
    fn places_an_unplaced_object_under_the_root() {
        let mut state = VoxMain::default();

        let palette_id = single_color_palette(&mut state, "#FF0000FF");

        state.retain_object(unit_object(palette_id)).unwrap();

        let state = to_mvox_vox_main(state).unwrap();

        assert_eq!(state.hierarchy_node_count(), 4);

        let (_, group) = state.iter_hierarchy_nodes().nth(1).unwrap();

        assert_eq!(group.child_node_ids.len(), 1);

        let (_, shape) = state.iter_hierarchy_nodes().nth(3).unwrap();

        assert_eq!(shape.child_object_ids.len(), 1);
    }

    /// A grid past 256 voxels on an axis has no byte coordinate for its far
    /// cells, so the state is refused.
    #[test]
    fn errors_on_a_grid_over_the_axis_limit() {
        let mut state = VoxMain::default();

        state
            .retain_object(VoxObject::new(String::new(), TyVector3U32::new(257, 1, 1)).unwrap())
            .unwrap();

        assert!(to_mvox_vox_main(state).is_err());
    }

    #[test]
    fn errors_past_the_palette_limit() {
        let mut state = VoxMain::default();

        let colors: Vec<_> = (0..=255u8)
            .map(|red| lin_srgba_f64_from_srgba_u8(TySrgbaU8::from([red, 0, 0, 255])).into())
            .collect();

        let value_pool_id = state.retain_value_pool(VoxValuePool::vec_4_float(colors).unwrap());

        let mut palette = VoxPalette::default();

        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();

        for color in 0..256u32 {
            palette
                .retain_material(vec![U32Id::from_u32(color)])
                .unwrap();
        }

        let palette_id = state.retain_palette(palette).unwrap();

        let mut object = VoxObject::new(String::new(), TyVector3U32::new(256, 1, 1)).unwrap();

        object.retain_layer(palette_id, U32Id::from_u32(0));

        for color in 0..256u32 {
            let voxel_id = object.voxel_id(TyVector3U32::new(color, 0, 0)).unwrap();

            object
                .retain_voxel(voxel_id, &[U32Id::from_u32(color)])
                .unwrap();
        }

        state.retain_object(object).unwrap();

        assert!(to_mvox_vox_main(state).is_err());
    }

    /// One palette with one `baseColor` material of `hex`.
    fn single_color_palette(state: &mut VoxMain<()>, hex: &str) -> U32Id<BVoxPalette> {
        let value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_4_float(vec![linear_rgba(hex)]).unwrap());

        let mut palette = VoxPalette::default();

        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();

        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

        state.retain_palette(palette).unwrap()
    }

    /// A one-voxel object sampling material 0 of `palette_id`.
    fn unit_object(palette_id: U32Id<BVoxPalette>) -> VoxObject {
        let mut object = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1)).unwrap();

        object.retain_layer(palette_id, U32Id::from_u32(0));

        let voxel_id = object.voxel_id(TyVector3U32::ZERO).unwrap();

        object
            .retain_voxel(voxel_id, &[U32Id::from_u32(0)])
            .unwrap();

        object
    }

    /// The color of the one voxel of object `object_id`.
    fn color_of<T>(state: &VoxMain<T>, object_id: U32Id<BVoxObject>) -> [u8; 4] {
        let object = state.object(object_id).unwrap();

        let voxel_id = object.voxel_id(TyVector3U32::ZERO).unwrap();

        resolve_cell_color(state, object)
            .unwrap()
            .expect("a colored object")
            .color(voxel_id)
    }
}
