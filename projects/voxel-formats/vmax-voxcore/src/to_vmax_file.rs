use crate::{
    PaletteAxes, PalettePlan, Result, VMaxVoxMain, VMaxWriteOptions, apply_scene_camera,
    contents_editor_state, ext_entry, ext_placements, extend_palette_plan, group_from_node,
    new_palette_plan, node_rotation, object_file_suffix, object_from_node, place_object,
    reconstruct_voxels, secondary_object_ext, subtree_box_local, write_palette_files,
};
use branded_id::U32Id;
use std::collections::{BTreeMap, HashMap, HashSet};
use vmax::{
    VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxObject, VMaxPalettePngFile,
    VMaxPaletteSettingsVmaxpsbFile, snapshots::encode_vmax_snapshots,
};
use voxcore::{BVoxObject, BVoxPalette};

/// Writes a [`VMaxVoxMain`] to a Voxel Max document, the inverse of
/// [`from_vmax_file`](crate::from_vmax_file). A loaded document writes back
/// exactly through its ext. A state
/// [`to_vmax_vox_main`](crate::to_vmax_vox_main) gave its ext writes as a
/// document synthesized from the scene. The ext supplies each node's,
/// palette's, and object's provenance and the scene-level state. The scene
/// supplies the rest: names, transforms, parents, and content boxes. Nodes
/// write in listing order, children before parents when the listing has them
/// so, as Voxel Max's documents do. Errors when an entity has no ext entry.
pub fn to_vmax_file(main: &VMaxVoxMain, options: &VMaxWriteOptions) -> Result<VMaxFile> {
    let placements = ext_placements(main)?;

    let mut objects: Vec<VMaxObject> = Vec::new();
    let mut groups: Vec<VMaxGroup> = Vec::new();
    // One palette plan per distinct ordered list of layer palettes, in
    // first-seen order, and each list's position in it.
    let mut plans: Vec<PalettePlan> = Vec::new();
    let mut plan_index_of: HashMap<Vec<U32Id<BVoxPalette>>, usize> = HashMap::new();
    // Object id -> its `data` filename, shared by every node that places it.
    let mut contents_by_object: BTreeMap<U32Id<BVoxObject>, String> = BTreeMap::new();

    let mut contents_files: BTreeMap<String, VMaxContentsVmaxbFile> = BTreeMap::new();
    let mut palette_settings_files: BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile> =
        BTreeMap::new();
    let mut palette_png_files: BTreeMap<String, VMaxPalettePngFile> = BTreeMap::new();

    // Every node's `ind` is distinct through its entry. An extra object on a
    // node placing several is emitted without an entry, so it takes a triplet
    // no entry uses.
    let mut used_indices: HashSet<[i64; 3]> = placements
        .iter()
        .map(|placement| placement.ext.index)
        .collect();

    // A group's content box is derived from its subtree, the same box for every
    // path to a shared node, so it is memoized by node id.
    let mut box_memo: HashMap<u32, ([f64; 3], [f64; 3])> = HashMap::new();

    for placement in &placements {
        let node = placement.node;
        let ext_node = placement.ext;
        let rotation = node_rotation(ext_node, node);

        if node.child_object_ids.is_empty() {
            let (center, half) = subtree_box_local(main, placement.node_id, &mut box_memo);
            groups.push(group_from_node(placement, rotation, center, half));
            continue;
        }

        // One scene object per child object. A vmax-origin node always carries
        // a single object, so this loops once and matches the lossless path
        // exactly. A synthesized node may carry several, such as a Goxel
        // layer's blocks; the extra objects become sibling object-nodes under
        // the same parent.
        for (slot, object_id) in node.child_object_ids.iter().enumerate() {
            let object_id = *object_id;
            let object = main.object(object_id).expect("a valid node child object");
            let axes = PaletteAxes::resolve(main, object)?;
            let layer_palette_ids: Vec<_> = object.iter_layers().map(|(_, id)| id).collect();
            let plan_index = match plan_index_of.get(&layer_palette_ids) {
                Some(&plan_index) => plan_index,
                None => {
                    let colored_count = plans.iter().filter(|p| p.color_table.is_some()).count();
                    plans.push(new_palette_plan(main, &axes, object, colored_count)?);
                    plan_index_of.insert(layer_palette_ids, plans.len() - 1);
                    plans.len() - 1
                }
            };
            let plan = &mut plans[plan_index];
            extend_palette_plan(plan, &axes, object)?;
            let suffix = object_file_suffix(object_id);
            // The node's ext places its first object. An extra object gets a
            // per-object variant with a distinct id and index and its own
            // bounds.
            let object_ext = if slot == 0 {
                ext_node.clone()
            } else {
                secondary_object_ext(ext_node, slot, &mut used_indices)
            };

            // Re-derive the object's internal-grid placement by convention:
            // center the canvas in the 256-wide workspace, then seat the
            // runtime grid inside it by the runtime/edit origin offset. The
            // runtime grid is the live voxels' tight extent within the object's
            // build volume; the content box follows from it and the build
            // volume, the scene placement from the node transform.
            let (tight, object_placement) = place_object(object);
            let object_state = ext_entry(
                main.ext().object_states.get(&object_id),
                "object",
                object_id.to_u32(),
            )?;

            // Instances share one contents file: rebuild it once.
            let data = match contents_by_object.get(&object_id) {
                Some(data) => data.clone(),
                None => {
                    let voxels = reconstruct_voxels(&tight, plan, object_placement.box_min);
                    let data = format!("contents{suffix}.vmaxb");
                    // Voxels re-encode into snapshots; serde-only state the
                    // decoded voxcore object does not model (`pal`) stays
                    // absent.
                    let contents = VMaxContentsVmaxbFile {
                        snapshots: encode_vmax_snapshots(&voxels),
                        ..contents_editor_state(object_state, &object_placement)
                    };
                    contents_files.insert(data.clone(), contents);
                    contents_by_object.insert(object_id, data.clone());
                    data
                }
            };

            let pal = plan.pal.clone();

            objects.push(object_from_node(
                node,
                &object_ext,
                placement.parent_id.clone(),
                rotation,
                &object_placement,
                data,
                pal,
                &suffix,
            ));
        }
    }

    write_palette_files(
        &plans,
        &mut palette_settings_files,
        &mut palette_png_files,
        options.color_format,
    )?;

    let mut scene = main.ext().scene.clone();
    scene.groups = groups;
    scene.objects = objects;
    apply_scene_camera(&mut scene, options.scene_camera);

    Ok(VMaxFile {
        scene_json_file: scene,
        contents_files,
        palette_settings_files,
        palette_png_files,
        history_vmaxhb_files: BTreeMap::new(),
        history_vmaxhvsb_files: BTreeMap::new(),
        history_vmaxhvsc_files: BTreeMap::new(),
        selection_vmaxb_files: BTreeMap::new(),
        thumbnail_png: None,
        contents_vmax_pngs: BTreeMap::new(),
        group_pngs: BTreeMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        SceneCameraSource, VMaxExtNode, VMaxVoxMain, VMaxWriteOptions, from_vmax_file,
        to_vmax_file, to_vmax_vox_main,
    };
    use branded_id::U32Id;
    use std::collections::{BTreeMap, BTreeSet, HashMap};
    use ty_math::{TyQuaternionF64, TyVector3F64, TyVector3U32};
    use vmax::{
        VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion,
        VMaxObject, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneCamera,
        VMaxSceneJsonFile, VMaxViewBox,
        snapshots::{VMaxVoxel, decode_vmax_snapshots, encode_vmax_snapshots},
    };
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxValuePoolValue,
        VoxHierarchyNode, VoxObject, material::BASE_COLOR,
    };

    fn material(
        mi: &str,
        metalness: f64,
        roughness: f64,
        emission: f64,
        enable_shadows: bool,
    ) -> VMaxMaterial {
        VMaxMaterial {
            mi: mi.to_owned(),
            mc: metalness,
            rc: roughness,
            sic: emission,
            sh: enable_shadows,
            tc: None,
            md: None,
        }
    }

    /// A coefficient snapped to an f32-exact value, matching what the writer
    /// stores so a fixture round-trips to equality.
    fn f32r(value: f64) -> f64 {
        f64::from(value as f32)
    }

    /// The neutral default material the writer pads unused slots with.
    fn default_material(slot: usize) -> VMaxMaterial {
        VMaxMaterial {
            mi: (slot + 1).to_string(),
            mc: f32r(0.1),
            rc: f32r(0.9),
            sic: 0.0,
            sh: true,
            tc: None,
            md: None,
        }
    }

    /// A 256-byte `lc` usage mask with `1 << material_byte` set at each
    /// `(color_cell, material_byte)`.
    fn lc_mask(cells: &[(usize, u8)]) -> Vec<u8> {
        let mut lc = vec![0u8; 256];
        for &(cell, byte) in cells {
            lc[cell] |= 1 << byte;
        }
        lc
    }

    /// The fixed settings sidecar the reverse path writes on a rebuilt material
    /// palette: the real materials padded to the eight slots, the per-color
    /// material map (`lc`/`indices`/`current`), and the default editor state,
    /// so a fixture round-trips to equality.
    fn palette_settings(
        name: &str,
        materials: Vec<VMaxMaterial>,
        colors: Vec<[u8; 4]>,
        lc: Vec<u8>,
        indices: Vec<i64>,
        current: i64,
    ) -> VMaxPaletteSettingsVmaxpsbFile {
        let mut materials = materials;
        if !materials.is_empty() {
            while materials.len() < 8 {
                materials.push(default_material(materials.len()));
            }
        }
        VMaxPaletteSettingsVmaxpsbFile {
            name: name.to_owned(),
            materials,
            colors: colors.iter().flatten().copied().collect(),
            indices,
            lc,
            palette_type: 0,
            transparency: 1.0,
            r: 0,
            rt: "n".to_owned(),
            cmt: "ng".to_owned(),
            current,
            ali: "1".to_owned(),
            voxmats: Vec::new(),
            ls: Vec::new(),
        }
    }

    /// A 256-cell image: 255 colors then a transparent terminator.
    fn palette_png() -> VMaxPalettePngFile {
        let mut cells: Vec<[u8; 4]> = (0..255u32).map(|i| [i as u8, 0, 0, 255]).collect();
        cells.push([0, 0, 0, 0]);
        VMaxPalettePngFile(cells)
    }

    /// A document with a root group, a child object placed at the origin with
    /// authored bounds, a shared color and material palette, and preserved
    /// scene and object editor state. Built in the canonical form the reverse
    /// path emits so it round-trips to equality.
    fn sample() -> VMaxFile {
        let group = VMaxGroup {
            name: "grp".to_owned(),
            id: "g".to_owned(),
            parent_id: None,
            hidden: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ind: [0, 0, 0],
            s: Some(false),
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            // The group's content box is derived from its subtree: the child
            // object sits at node-local [0, 0, 0]..[2, 2, 2], so the box
            // centers on [1, 1, 1] with unit half-extents.
            center: [1.0, 1.0, 1.0],
            bounds_min: Some([-1.0, -1.0, -1.0]),
            bounds_max: Some([1.0, 1.0, 1.0]),
        };
        let object = VMaxObject {
            name: "obj".to_owned(),
            data: "contents.vmaxb".to_owned(),
            palette: "palette1.png".to_owned(),
            history: "history.vmaxhb".to_owned(),
            id: "o".to_owned(),
            parent_id: Some("g".to_owned()),
            hidden: None,
            // Canonical placement the reverse path emits: the grid is centered
            // in the workspace, the content box is symmetric about the center,
            // and the placement pivots about it.
            position: [-127.0, -127.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ind: [0, 0, 0],
            s: Some(false),
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            center: [128.0, 128.0, 1.0],
            bounds_min: Some([-1.0, -1.0, -1.0]),
            bounds_max: Some([1.0, 1.0, 1.0]),
        };

        let scene_json_file = VMaxSceneJsonFile {
            v: 4,
            cam: Some(VMaxSceneCamera::default()),
            background: Some("#101010".to_owned()),
            groups: vec![group],
            objects: vec![object],
            ..Default::default()
        };

        // Canonical snapshots: voxels re-encoded at the centered internal grid
        // just as the reverse path emits, the runtime grid [127, 127, 0]..[128,
        // 128, 1].
        let contents = VMaxContentsVmaxbFile {
            snapshots: encode_vmax_snapshots(&[
                VMaxVoxel {
                    position: [127, 127, 0],
                    material_idx: 0,
                    color_idx: 5,
                },
                VMaxVoxel {
                    position: [128, 128, 1],
                    material_idx: 1,
                    color_idx: 3,
                },
            ]),
            uuid: "u".to_owned(),
            v: 4,
            tools: None,
            brush: None,
            cam: None,
            pal: None,
        };

        let mut contents_files = BTreeMap::new();
        contents_files.insert("contents.vmaxb".to_owned(), contents);
        let mut palette_settings_files = BTreeMap::new();
        palette_settings_files.insert(
            "palette1.settings.vmaxpsb".to_owned(),
            palette_settings(
                "mat",
                vec![
                    material("1", 0.0, 1.0, 0.0, true),
                    material("2", 0.5, 0.25, 2.0, false),
                ],
                Vec::new(),
                // Voxel 0 (color cell 4) draws material 0; voxel 1 (cell 2)
                // draws material 1.
                lc_mask(&[(4, 0), (2, 1)]),
                vec![2, 4],
                2,
            ),
        );
        let mut palette_png_files = BTreeMap::new();
        palette_png_files.insert("palette1.png".to_owned(), palette_png());

        VMaxFile {
            scene_json_file,
            contents_files,
            palette_settings_files,
            palette_png_files,
            history_vmaxhb_files: BTreeMap::new(),
            history_vmaxhvsb_files: BTreeMap::new(),
            history_vmaxhvsc_files: BTreeMap::new(),
            selection_vmaxb_files: BTreeMap::new(),
            thumbnail_png: None,
            contents_vmax_pngs: BTreeMap::new(),
            group_pngs: BTreeMap::new(),
        }
    }

    /// `sample()` with two more objects under the group, each with its own
    /// contents file and editor state.
    fn three_object_sample() -> VMaxFile {
        let mut file = sample();
        let object = file.scene_json_file.objects[0].clone();
        let contents = file.contents_files["contents.vmaxb"].clone();
        for (id, suffix) in [("o2", "2"), ("o3", "3")] {
            let data = format!("contents{suffix}.vmaxb");
            file.scene_json_file.objects.push(VMaxObject {
                name: id.to_owned(),
                data: data.clone(),
                history: format!("history{suffix}.vmaxhb"),
                id: id.to_owned(),
                ..object.clone()
            });
            file.contents_files.insert(
                data,
                VMaxContentsVmaxbFile {
                    uuid: format!("u{suffix}"),
                    ..contents.clone()
                },
            );
        }
        file
    }

    /// Releasing a middle node, its object, and the object's palette, then
    /// compacting, drops their entries and rekeys the survivors' to their
    /// compacted ids. The rebuilt document carries the survivors' ids and
    /// editor state, and reloads to the same ext.
    #[test]
    fn released_entities_leave_the_survivors_provenance_rekeyed() {
        let file = three_object_sample();
        let mut main = from_vmax_file(&file).unwrap();
        let original = main.ext().clone();

        // The group, node 0, places nodes 1..=3. Node 2 places object 1, which
        // draws palette 1.
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        let doomed_node_id = U32Id::<BVoxHierarchyNode>::from_u32(2);
        let doomed_object_id = U32Id::<BVoxObject>::from_u32(1);
        let doomed_palette_id = U32Id::<BVoxPalette>::from_u32(1);
        let mut group = main.hierarchy_node(group_id).unwrap().clone();
        group.child_node_ids.retain(|&id| id != doomed_node_id);
        main.set_hierarchy_node(group_id, group).unwrap();
        main.release_hierarchy_node(doomed_node_id).unwrap();
        main.release_object(doomed_object_id).unwrap();
        main.release_palette(doomed_palette_id).unwrap();
        main.gc().unwrap();

        // The last node, object, and palette each slide down one id.
        let mut expected = original;
        expected.hierarchy_nodes.remove(&doomed_node_id);
        let last_node = expected
            .hierarchy_nodes
            .remove(&U32Id::from_u32(3))
            .unwrap();
        expected.hierarchy_nodes.insert(doomed_node_id, last_node);
        expected.object_states.remove(&doomed_object_id);
        let last_object = expected.object_states.remove(&U32Id::from_u32(2)).unwrap();
        expected.object_states.insert(doomed_object_id, last_object);
        expected.palettes.remove(&doomed_palette_id);
        let last_palette = expected.palettes.remove(&U32Id::from_u32(2)).unwrap();
        expected.palettes.insert(doomed_palette_id, last_palette);
        assert_eq!(main.ext(), &expected);

        let rebuilt = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let ids: Vec<&str> = rebuilt
            .scene_json_file
            .objects
            .iter()
            .map(|object| object.id.as_str())
            .collect();
        assert_eq!(ids, ["o", "o3"]);
        let uuids: BTreeSet<&str> = rebuilt
            .contents_files
            .values()
            .map(|contents| contents.uuid.as_str())
            .collect();
        assert_eq!(uuids, BTreeSet::from(["u", "u3"]));

        let reloaded = from_vmax_file(&rebuilt).unwrap();
        assert_eq!(reloaded.ext(), &expected);
    }

    /// A node and an object retained after the load take synthesized entries
    /// as they are retained. The write reads them back out and links no
    /// parent for a root.
    #[test]
    fn a_node_retained_after_the_load_takes_a_synthesized_entry() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        let mut object = VoxObject::new(String::new(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));
        let voxel_id = object.voxel_id(TyVector3U32::splat(0)).unwrap();
        object
            .retain_voxel(voxel_id, &[U32Id::<BVoxMaterial>::from_u32(0)])
            .unwrap();
        let object_id = main.retain_object(object).unwrap();
        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "added".to_owned(),
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(node_id).unwrap();

        let ext = main.ext();
        assert_eq!(ext.hierarchy_nodes.len(), 3);
        assert_eq!(
            ext.hierarchy_nodes[&node_id],
            VMaxExtNode {
                id: "00000000-0000-0000-0000-000000000001".to_owned(),
                index: [0, 0, 1],
                rotation: [0.0, 0.0, 0.0, 0.0],
                alignment: "f".to_owned(),
                pivot_face: "8".to_owned(),
                pivot_align: "4".to_owned(),
                selected: None,
            }
        );
        assert_eq!(ext.object_states.len(), 2);
        let object_state = &ext.object_states[&object_id];
        assert_eq!(object_state.uuid, "00000000-0000-0001-0000-000000000001");
        assert_eq!(object_state.v, 4);
        assert!(object_state.tools.is_some() && object_state.brush.is_some());
        assert_eq!(
            object_state.cam.as_ref().map(|cam| cam.o),
            Some([127.5, 127.5, 0.5])
        );

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let added = file
            .scene_json_file
            .objects
            .iter()
            .find(|object| object.name == "added")
            .expect("the retained node writes as an object");
        assert_eq!(added.id, "00000000-0000-0000-0000-000000000001");
        assert_eq!(added.ind, [0, 0, 1]);
        assert_eq!(added.parent_id, None);
        assert_eq!(added.t_al, "f");
        let contents = &file.contents_files[&added.data];
        assert_eq!(contents.uuid, "00000000-0000-0001-0000-000000000001");
        assert_eq!(
            contents.tools.as_ref().and_then(|tools| tools.vp.clone()),
            Some(VMaxViewBox {
                min: [127, 127, 0],
                max: [127, 127, 0],
            })
        );

        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(reloaded.ext().hierarchy_nodes.len(), 3);
    }

    /// A node rotated after the load writes its live rotation, while an
    /// unrotated one keeps the preserved spelling.
    #[test]
    fn a_node_rotated_after_the_load_writes_its_live_rotation() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        let mut group = main.hierarchy_node(group_id).unwrap().clone();
        group.transform.rotation = TyQuaternionF64::from_axis_angle(TyVector3F64::Z, 0.5);
        main.set_hierarchy_node(group_id, group).unwrap();

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let [x, y, z, angle] = file.scene_json_file.groups[0].rotation;
        assert!(x.abs() < 1e-12 && y.abs() < 1e-12);
        assert!((z - 1.0).abs() < 1e-12);
        assert!((angle - 0.5).abs() < 1e-12);
        assert_eq!(
            file.scene_json_file.objects[0].rotation,
            [0.0, 0.0, 0.0, 0.0]
        );
    }

    /// Voxel Max holds a tree, so a node placed by two parents, or a root
    /// that is also a child, refuses to write instead of picking a parent.
    #[test]
    fn a_node_with_two_parents_or_a_root_child_errors() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        let object_node_id = U32Id::<BVoxHierarchyNode>::from_u32(1);
        let other_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "other".to_owned(),
                child_node_ids: vec![object_node_id],
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(other_id).unwrap();
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());

        let mut main = from_vmax_file(&sample()).unwrap();
        main.push_root_hierarchy_node_id(object_node_id).unwrap();
        assert!(main.root_hierarchy_node_ids().contains(&group_id));
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());
    }

    /// A material retained to a palette with an exact material list takes the
    /// slot its material-axis value ids select, and the writer draws it. A
    /// retain after the value pools were pruned refuses, because the pools no
    /// longer index the list.
    #[test]
    fn a_retained_material_takes_the_slot_its_values_select() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        // Color cell 0 in slot 1: every material-axis property takes value 1.
        let palette = main.palette(palette_id).unwrap();
        let value_ids: Vec<U32Id<BVoxValuePoolValue>> = palette
            .iter_properties()
            .map(|(_, property)| U32Id::from_u32(u32::from(property.name != BASE_COLOR)))
            .collect();
        let material_id = main.retain_material(palette_id, value_ids).unwrap();
        assert_eq!(main.ext().palettes[&palette_id].slots[&material_id], 1);

        main.prune_value_pools();
        let value_ids: Vec<U32Id<BVoxValuePoolValue>> = main
            .palette(palette_id)
            .unwrap()
            .iter_properties()
            .map(|_| U32Id::from_u32(0))
            .collect();
        assert!(main.retain_material(palette_id, value_ids).is_err());
    }

    /// Voxel Max lists child groups before their parents in its own files:
    /// in `tyt-assets/src/vmax/mixed-shapes.vmax/scene.json` the groups at
    /// index 0 and 1 have a `pid` naming the group at index 2. The loader and
    /// the writer keep that listing order.
    #[test]
    fn child_groups_listed_before_their_parent_round_trip_in_order() {
        let mut file = sample();
        let root = file.scene_json_file.groups[0].clone();
        let group = |id: &str, parent_id: Option<&str>, ind: [i64; 3]| VMaxGroup {
            id: id.to_owned(),
            parent_id: parent_id.map(str::to_owned),
            ind,
            ..root.clone()
        };
        // The listing from the asset, with the object under the innermost
        // group so every group has geometry.
        file.scene_json_file.groups = vec![
            group(
                "233A8A71-7B3C-48D0-B0A8-D774275FA80E",
                Some("528B9E32-71C7-4887-9570-D920B7D9C988"),
                [0, 1, 0],
            ),
            group(
                "528B9E32-71C7-4887-9570-D920B7D9C988",
                Some("DEB88339-5C59-4CC6-B7F6-1DAC39397C1B"),
                [0, 1, 1],
            ),
            group("DEB88339-5C59-4CC6-B7F6-1DAC39397C1B", None, [0, 1, 2]),
        ];
        file.scene_json_file.objects[0].parent_id =
            Some("233A8A71-7B3C-48D0-B0A8-D774275FA80E".to_owned());

        let main = from_vmax_file(&file).unwrap();
        assert_eq!(main.root_hierarchy_node_ids(), [U32Id::from_u32(2)]);
        assert_eq!(
            main.hierarchy_node(U32Id::from_u32(2))
                .unwrap()
                .child_node_ids,
            [U32Id::from_u32(1)]
        );

        let rebuilt = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        assert_eq!(rebuilt.scene_json_file.groups, file.scene_json_file.groups);
        assert_eq!(
            rebuilt.scene_json_file.objects[0].parent_id,
            file.scene_json_file.objects[0].parent_id
        );
    }

    /// A reduction repaints onto a survivor, releases the rest, prunes the
    /// value pools, and compacts. The survivor's slot rides in the ext, so the
    /// document still draws its exact material even though its value ids were
    /// renumbered, and the reload records the one slot.
    #[test]
    fn a_reduction_keeps_the_survivors_material_slot_through_prune_and_gc() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        // The palette's materials sort by color cell then slot: material 0 draws
        // cell 2 in slot 1 and material 1 draws cell 4 in slot 0. Keep material
        // 0, whose slot the pools stop recording once slot 0's values are
        // pruned away and its own renumber to 0.
        let doomed_id = U32Id::<BVoxMaterial>::from_u32(1);
        let survivor_id = U32Id::<BVoxMaterial>::from_u32(0);
        main.repaint_materials(palette_id, &HashMap::from([(doomed_id, survivor_id)]))
            .unwrap();
        main.release_material(palette_id, doomed_id).unwrap();
        main.prune_value_pools();
        main.gc().unwrap();
        assert_eq!(
            main.ext().palettes[&palette_id].slots,
            BTreeMap::from([(survivor_id, 1)])
        );

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let voxels =
            decode_vmax_snapshots(&file.contents_files["contents.vmaxb"].snapshots).unwrap();
        assert!(voxels.iter().all(|voxel| voxel.material_idx == 1));
        let settings = &file.palette_settings_files["palette1.settings.vmaxpsb"];
        assert_eq!(settings.materials[1], material("2", 0.5, 0.25, 2.0, false));

        let reloaded = from_vmax_file(&file).unwrap();
        let palette = &reloaded.ext().palettes[&palette_id];
        assert_eq!(palette.slots, BTreeMap::from([(survivor_id, 1)]));
        assert_eq!(palette.materials.len(), 8);
    }

    /// An ext missing an entity's entry is malformed, so the writer errors
    /// instead of synthesizing one.
    #[test]
    fn an_ext_missing_an_entry_errors() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let ext = main.ext_mut();
        ext.hierarchy_nodes.remove(&U32Id::from_u32(1));
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());

        let mut main = from_vmax_file(&sample()).unwrap();
        let ext = main.ext_mut();
        ext.object_states.remove(&U32Id::from_u32(0));
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());

        let mut main = from_vmax_file(&sample()).unwrap();
        let ext = main.ext_mut();
        ext.palettes.remove(&U32Id::from_u32(0));
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());
    }

    #[test]
    fn round_trips_through_vox_state() {
        let original = sample();
        let main = from_vmax_file(&original).unwrap();
        let rebuilt = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        assert_eq!(rebuilt, original);
    }

    /// A document whose glowing material spans more colors than there are
    /// material slots writes back through a bare state, where the materials
    /// are derived rather than read from the ext. The emissive color rides the
    /// color axis, one per cell, so it must not split the materials: two
    /// derive, not one per color, and every voxel keeps its cell and slot.
    #[test]
    fn derives_materials_across_more_colors_than_slots() {
        let mut original = sample();
        let voxels: Vec<VMaxVoxel> = (0..10u8)
            .map(|i| VMaxVoxel {
                position: [127 + i32::from(i), 127, 0],
                material_idx: i % 2,
                color_idx: 10 + i,
            })
            .collect();
        original
            .contents_files
            .get_mut("contents.vmaxb")
            .unwrap()
            .snapshots = encode_vmax_snapshots(&voxels);
        let main = from_vmax_file(&original).unwrap();
        let bare = to_vmax_vox_main(main.take_ext().main).unwrap();
        let rebuilt = to_vmax_file(&bare, &VMaxWriteOptions::default()).unwrap();

        assert_eq!(rebuilt.palette_settings_files.len(), 1);
        let settings = rebuilt.palette_settings_files.values().next().unwrap();
        let sics: Vec<f64> = settings.materials.iter().map(|m| m.sic).collect();
        assert_eq!(sics, [0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // The bare path re-seats the grid, so compare the cells and slots in
        // row order rather than the positions.
        let mut written =
            decode_vmax_snapshots(&rebuilt.contents_files.values().next().unwrap().snapshots)
                .unwrap();
        written.sort_by_key(|v| v.position);
        let samples = |voxels: &[VMaxVoxel]| -> Vec<(u8, u8)> {
            voxels
                .iter()
                .map(|v| (v.material_idx, v.color_idx))
                .collect()
        };
        assert_eq!(samples(&written), samples(&voxels));
    }

    /// A material carrying dispersion, a transmission color (`tc`), and a
    /// non-finite coefficient round-trips exactly, alongside a plain material
    /// and the padded default slots. The value pools carry a finite-defaulted
    /// neutral copy, but the ext keeps the exact values, so the rebuilt
    /// document matches byte for byte. Coefficients are f32-exact, as Voxel Max
    /// stores them, and the non-finite `mc` is infinity, not NaN, so it
    /// compares equal.
    #[test]
    fn round_trips_rich_materials() {
        let mut original = sample();
        let mut materials = vec![
            VMaxMaterial {
                mi: "1".to_owned(),
                mc: f64::INFINITY,
                rc: f32r(0.25),
                sic: f32r(1.0),
                sh: true,
                tc: Some(f32r(0.6)),
                md: Some(VMaxMaterialDispersion {
                    absorption: f32r(0.1),
                    ior: f32r(1.4),
                    transmission: f32r(0.3),
                }),
            },
            material("2", f32r(0.5), f32r(0.25), f32r(2.0), false),
        ];
        for slot in materials.len()..8 {
            materials.push(default_material(slot));
        }
        original
            .palette_settings_files
            .get_mut("palette1.settings.vmaxpsb")
            .unwrap()
            .materials = materials;
        let main = from_vmax_file(&original).unwrap();
        let rebuilt = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        assert_eq!(rebuilt, original);
    }

    /// Puts a scene camera in the state's ext for the typed path to keep or
    /// replace.
    fn with_ext_scene_camera(main: &mut VMaxVoxMain, cam: VMaxSceneCamera) {
        let vmax_ext = main.ext_mut();
        vmax_ext.scene.cam = Some(cam);
    }

    #[test]
    fn scene_camera_ext_keeps_the_ext_camera() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let cam = VMaxSceneCamera {
            z: 321.0,
            ..Default::default()
        };
        with_ext_scene_camera(&mut main, cam);
        let options = VMaxWriteOptions {
            scene_camera: SceneCameraSource::Ext,
            ..Default::default()
        };
        let file = to_vmax_file(&main, &options).unwrap();
        assert_eq!(file.scene_json_file.cam, Some(cam));
    }

    /// `SceneCameraSource::Empty` replaces the ext's scene camera with the
    /// default, while leaving the camera unset keeps it.
    #[test]
    fn scene_camera_empty_replaces_the_ext_camera() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let cam = VMaxSceneCamera {
            z: 999.0,
            ..Default::default()
        };
        with_ext_scene_camera(&mut main, cam);

        let kept = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        assert_eq!(kept.scene_json_file.cam, Some(cam));

        let options = VMaxWriteOptions {
            scene_camera: SceneCameraSource::Empty,
            ..Default::default()
        };
        let empty = to_vmax_file(&main, &options).unwrap();
        assert_ne!(empty.scene_json_file.cam, Some(cam));
    }
}
