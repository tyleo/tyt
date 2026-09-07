use crate::{Result, VMaxColorFormat, ext::VMaxVoxMain, write_vmax};
use vmax::VMaxFile;

/// Writes a [`VMaxVoxMain`] back to a Voxel Max document, the inverse of
/// [`from_vmax_file_with_ext`](crate::ext::from_vmax_file_with_ext) and the
/// typed form of [`to_vmax_file`](crate::to_vmax_file). A loaded document
/// writes back exactly through its ext. A state carrying none writes a
/// synthesized document. For control over the scene camera, use
/// [`VmaxFileBuilder`](crate::VmaxFileBuilder) through
/// [`new_with_ext`](crate::VmaxFileBuilder::new_with_ext).
pub fn to_vmax_file_with_ext(
    state: &VMaxVoxMain,
    vmax_color_format: VMaxColorFormat,
) -> Result<VMaxFile> {
    write_vmax(state, state.ext().as_ref(), vmax_color_format, None)
}

#[cfg(test)]
mod tests {
    use crate::{
        SceneCameraSource, VMaxColorFormat, VmaxFileBuilder,
        ext::{VMaxExtNode, VMaxVoxMain, from_vmax_file_with_ext, to_vmax_file_with_ext},
    };
    use branded_id::U32Id;
    use std::collections::{BTreeMap, BTreeSet, HashMap};
    use ty_math::TyVector3U32;
    use vmax::{
        VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion,
        VMaxObject, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneCamera,
        VMaxSceneJsonFile,
        snapshots::{VMaxVoxel, decode_vmax_snapshots, encode_vmax_snapshots},
    };
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxObject,
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
    /// compacting, leaves the survivors' provenance aligned. The rebuilt
    /// document carries their ids and editor state, and reloads to the loaded
    /// ext minus the released entries.
    #[test]
    fn released_entities_leave_the_survivors_provenance_aligned() {
        let file = three_object_sample();
        let mut state = from_vmax_file_with_ext(&file).unwrap();
        let original = state
            .ext()
            .clone()
            .expect("a loaded document carries its ext");

        // The group, node 0, places nodes 1..=3. Node 2 places object 1, which
        // folds palette 1.
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        let doomed_node_id = U32Id::<BVoxHierarchyNode>::from_u32(2);
        let doomed_object_id = U32Id::<BVoxObject>::from_u32(1);
        let doomed_palette_id = U32Id::<BVoxPalette>::from_u32(1);
        let mut group = state.hierarchy_node(group_id).unwrap().clone();
        group.child_node_ids.retain(|&id| id != doomed_node_id);
        state.set_hierarchy_node(group_id, group).unwrap();
        state.release_hierarchy_node(doomed_node_id).unwrap();
        state.release_object(doomed_object_id).unwrap();
        state.release_palette(doomed_palette_id).unwrap();
        state.gc();

        let mut expected = original;
        expected.hierarchy_nodes.remove(2);
        expected.object_states.remove(1);
        expected.palettes.remove(1);
        assert_eq!(state.ext(), &Some(expected.clone()));

        let rebuilt = to_vmax_file_with_ext(&state, VMaxColorFormat::Png).unwrap();
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

        let reloaded = from_vmax_file_with_ext(&rebuilt).unwrap();
        assert_eq!(reloaded.ext(), &Some(expected));
    }

    /// A node retained after the load takes a default entry, which the writer
    /// fills in like a synthesized node: a fresh id, no parent for a root, and
    /// the default anchor tokens.
    #[test]
    fn a_node_retained_after_the_load_writes_like_a_synthesized_node() {
        let mut state = from_vmax_file_with_ext(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        let mut object = VoxObject::new(String::new(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));
        let voxel_id = object.voxel_id(TyVector3U32::splat(0)).unwrap();
        object
            .retain_voxel(voxel_id, &[U32Id::<BVoxMaterial>::from_u32(0)])
            .unwrap();
        let object_id = state.retain_object(object).unwrap();
        let node_id = state
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "added".to_owned(),
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        state.push_root_hierarchy_node_id(node_id).unwrap();

        let ext = state.ext().as_ref().unwrap();
        assert_eq!(ext.hierarchy_nodes.len(), 3);
        assert_eq!(ext.hierarchy_nodes[2], VMaxExtNode::default());
        assert_eq!(ext.object_states.len(), 2);
        assert_eq!(ext.object_states[1], None);

        let file = to_vmax_file_with_ext(&state, VMaxColorFormat::Png).unwrap();
        let added = file
            .scene_json_file
            .objects
            .iter()
            .find(|object| object.name == "added")
            .expect("the retained node writes as an object");
        assert_eq!(added.id, "00000000-0000-0000-0000-000000000001");
        assert_eq!(added.parent_id, None);
        assert_eq!(added.t_al, "f");

        let reloaded = from_vmax_file_with_ext(&file).unwrap();
        assert_eq!(reloaded.ext().as_ref().unwrap().hierarchy_nodes.len(), 3);
    }

    /// A reduction repaints onto a survivor, releases the rest, prunes the
    /// value pools, and compacts. The survivor's slot rides in the ext, so the
    /// document still draws its exact material even though its value ids were
    /// renumbered, and the reload records the one slot.
    #[test]
    fn a_reduction_keeps_the_survivors_material_slot_through_prune_and_gc() {
        let mut state = from_vmax_file_with_ext(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        // The folded materials sort by color cell then slot: material 0 draws
        // cell 2 in slot 1 and material 1 draws cell 4 in slot 0. Keep material
        // 0, whose slot the pools stop recording once slot 0's values are
        // pruned away and its own renumber to 0.
        let doomed_id = U32Id::<BVoxMaterial>::from_u32(1);
        let survivor_id = U32Id::<BVoxMaterial>::from_u32(0);
        state
            .repaint_materials(palette_id, &HashMap::from([(doomed_id, survivor_id)]))
            .unwrap();
        state.release_material(palette_id, doomed_id).unwrap();
        state.prune_value_pools();
        state.gc();
        assert_eq!(
            state.ext().as_ref().unwrap().palettes[0]
                .as_ref()
                .unwrap()
                .slots,
            [1]
        );

        let file = to_vmax_file_with_ext(&state, VMaxColorFormat::Png).unwrap();
        let voxels =
            decode_vmax_snapshots(&file.contents_files["contents.vmaxb"].snapshots).unwrap();
        assert!(voxels.iter().all(|voxel| voxel.material_idx == 1));
        let settings = &file.palette_settings_files["palette1.settings.vmaxpsb"];
        assert_eq!(settings.materials[1], material("2", 0.5, 0.25, 2.0, false));

        let reloaded = from_vmax_file_with_ext(&file).unwrap();
        let palette = reloaded.ext().as_ref().unwrap().palettes[0]
            .as_ref()
            .unwrap();
        assert_eq!(palette.slots, [1]);
        assert_eq!(palette.materials.len(), 8);
    }

    /// An ext whose lists fall out of step with the listings is malformed, so
    /// the writer errors instead of pairing entries by a shifted index.
    #[test]
    fn an_ext_out_of_step_with_its_listings_errors() {
        let mut state = from_vmax_file_with_ext(&sample()).unwrap();
        let mut ext = state.ext().clone().unwrap();
        ext.hierarchy_nodes.pop();
        state.set_ext(Some(ext));
        assert!(to_vmax_file_with_ext(&state, VMaxColorFormat::Png).is_err());

        let mut state = from_vmax_file_with_ext(&sample()).unwrap();
        let mut ext = state.ext().clone().unwrap();
        ext.object_states.pop();
        state.set_ext(Some(ext));
        assert!(to_vmax_file_with_ext(&state, VMaxColorFormat::Png).is_err());
    }

    #[test]
    fn round_trips_through_vox_state() {
        let original = sample();
        let state = from_vmax_file_with_ext(&original).unwrap();
        let rebuilt = to_vmax_file_with_ext(&state, VMaxColorFormat::Png).unwrap();
        assert_eq!(rebuilt, original);
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
        let state = from_vmax_file_with_ext(&original).unwrap();
        let rebuilt = to_vmax_file_with_ext(&state, VMaxColorFormat::Png).unwrap();
        assert_eq!(rebuilt, original);
    }

    /// Puts a scene camera in the state's ext for the typed path to keep or
    /// replace.
    fn with_ext_scene_camera(state: &mut VMaxVoxMain, cam: VMaxSceneCamera) {
        let mut vmax_ext = state
            .ext()
            .clone()
            .expect("a loaded document carries its ext");
        vmax_ext.scene.cam = Some(cam);
        state.set_ext(Some(vmax_ext));
    }

    #[test]
    fn scene_camera_ext_keeps_the_ext_camera() {
        let mut state = from_vmax_file_with_ext(&sample()).unwrap();
        let cam = VMaxSceneCamera {
            z: 321.0,
            ..Default::default()
        };
        with_ext_scene_camera(&mut state, cam);
        let file = VmaxFileBuilder::new_with_ext(&state)
            .scene_camera(SceneCameraSource::Ext)
            .build()
            .unwrap();
        assert_eq!(file.scene_json_file.cam, Some(cam));
    }

    /// `SceneCameraSource::Empty` replaces the ext's scene camera with the
    /// default, while leaving the camera unset keeps it.
    #[test]
    fn scene_camera_empty_replaces_the_ext_camera() {
        let mut state = from_vmax_file_with_ext(&sample()).unwrap();
        let cam = VMaxSceneCamera {
            z: 999.0,
            ..Default::default()
        };
        with_ext_scene_camera(&mut state, cam);

        let kept = VmaxFileBuilder::new_with_ext(&state).build().unwrap();
        assert_eq!(kept.scene_json_file.cam, Some(cam));

        let empty = VmaxFileBuilder::new_with_ext(&state)
            .scene_camera(SceneCameraSource::Empty)
            .build()
            .unwrap();
        assert_ne!(empty.scene_json_file.cam, Some(cam));
    }
}
