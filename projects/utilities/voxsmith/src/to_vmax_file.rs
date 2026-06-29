use crate::{Result, VoxelMaxColorFormat, write_vmax};
use vmax::VMaxFile;
use voxcore::VoxMain;

/// Writes a [`VoxMain`] back to a Voxel Max document with default settings, the
/// inverse of [`from_vmax_file`](crate::from_vmax_file). `voxel_max_color_format`
/// selects where each palette's colors are stored, as described on
/// [`VoxelMaxColorFormat`]. For control over the scene camera, use
/// [`VmaxFileBuilder`](crate::VmaxFileBuilder).
pub fn to_vmax_file(
    state: &VoxMain,
    voxel_max_color_format: VoxelMaxColorFormat,
) -> Result<VMaxFile> {
    write_vmax(state, voxel_max_color_format, None)
}

#[cfg(test)]
mod tests {
    use crate::{
        SceneCameraSource, VmaxFileBuilder, VoxelMaxColorFormat, VoxelMaxExt, VoxelMaxExtWrapper,
        cell_color, from_vmax_file, object_color_ref, to_vmax_file, to_vox_value,
    };
    use branded_id::U32Id;
    use std::collections::{BTreeMap, BTreeSet};
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3U32};
    use vmax::{
        VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxObject, VMaxPalettePngFile,
        VMaxSceneCamera, VMaxSceneJsonFile,
    };
    use vmax_codec::{VMaxVoxel, decode_vmax_snapshots, encode_vmax_snapshots};
    use voxcore::{
        BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteCell, VoxHierarchyNode, VoxMain,
        VoxMap, VoxObject, VoxPalette, VoxValue,
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

    /// The fixed editor-state defaults the reverse path writes on a rebuilt
    /// material palette, so a fixture round-trips to equality.
    fn palette_settings(
        name: &str,
        materials: Vec<VMaxMaterial>,
        colors: Vec<[u8; 4]>,
    ) -> vmax::VMaxPaletteSettingsVmaxpsbFile {
        vmax::VMaxPaletteSettingsVmaxpsbFile {
            name: name.to_owned(),
            materials,
            colors: colors.iter().flatten().copied().collect(),
            indices: Vec::new(),
            lc: vec![0u8; 256],
            palette_type: 0,
            transparency: 1.0,
            r: 0,
            rt: "n".to_owned(),
            cmt: "ng".to_owned(),
            current: 0,
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
    /// authored bounds, a shared color and material palette, and preserved scene
    /// and object editor state. Built in the canonical form the reverse path
    /// emits so it round-trips to equality.
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
            // The group's content box is derived from its subtree: the child object
            // sits at node-local [0, 0, 0]..[2, 2, 2], so the box centers on [1, 1, 1]
            // with unit half-extents.
            center: [1.0, 1.0, 1.0],
            bounds_min: Some([-1.0, -1.0, -1.0]),
            bounds_max: Some([1.0, 1.0, 1.0]),
        };
        let object = VMaxObject {
            name: "obj".to_owned(),
            data: "contents.vmaxb".to_owned(),
            palette: "palette.png".to_owned(),
            history: "history.vmaxhb".to_owned(),
            id: "o".to_owned(),
            parent_id: Some("g".to_owned()),
            hidden: None,
            // Canonical placement the reverse path emits: the grid is centered in
            // the workspace, the content box is symmetric about the center, and the
            // placement pivots about it.
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

        // Canonical snapshots: voxels re-encoded at the centered internal grid just
        // as the reverse path emits, the runtime grid [127, 127, 0]..[128, 128, 1].
        let contents = VMaxContentsVmaxbFile {
            snapshots: encode_vmax_snapshots(&[
                VMaxVoxel {
                    position: [127, 127, 0],
                    material_idx: 1,
                    color_idx: 5,
                },
                VMaxVoxel {
                    position: [128, 128, 1],
                    material_idx: 0,
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
            "palette.settings.vmaxpsb".to_owned(),
            palette_settings(
                "mat",
                vec![
                    material("1", 0.0, 1.0, 0.0, true),
                    material("2", 0.5, 0.25, 2.0, false),
                ],
                Vec::new(),
            ),
        );
        let mut palette_png_files = BTreeMap::new();
        palette_png_files.insert("palette.png".to_owned(), palette_png());

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

    #[test]
    fn round_trips_through_vox_state() {
        let original = sample();
        let state = from_vmax_file(&original).unwrap();
        let rebuilt = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(rebuilt, original);
    }

    /// A state carrying no format ext, built straight from voxcore: a red-green
    /// object and a blue object sharing one `rgba` palette, placed by a small
    /// hierarchy of a nested group and two roots. The writer must synthesize the
    /// `voxel-max` ext from this rather than read one. Red is cell 0, so a live
    /// voxel references the palette index Voxel Max reads as empty; synthesis must
    /// shift it past index 0 rather than drop the voxel.
    fn source_state() -> VoxMain {
        let mut state = VoxMain::default();

        // One rgba palette whose first real color is cell 0: red, green, blue.
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        for hex in ["#FF0000FF", "#00FF00FF", "#0000FFFF"] {
            palette
                .add_cell(vec![VoxValue::Text(hex.to_owned())])
                .expect("one value per attribute");
        }
        let palette_id = state.add_palette(palette);
        let cell = |index: u32| U32Id::<BVoxPaletteCell>::from_u32(index);

        // Object 0: a red (cell 0) then a green (cell 1) voxel along x.
        let mut wide = VoxObject::new(String::new(), TyVector3U32::new(2, 1, 1))
            .expect("a 2x1x1 grid is within the dense limit");
        wide.add_palette_ref(palette_id, cell(0));
        for (x, color) in [(0u32, 0u32), (1, 1)] {
            let voxel = wide
                .voxel_id(TyVector3U32::new(x, 0, 0))
                .expect("a position within the grid");
            wide.retain_voxel(voxel, &[cell(color)])
                .expect("one sample for the one reference");
        }
        state.add_object(wide);

        // Object 1: a single blue (cell 2) voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.add_palette_ref(palette_id, cell(0));
        let voxel = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel, &[cell(2)])
            .expect("one sample for the one reference");
        state.add_object(unit);

        let object = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at = |x: f64, y: f64, z: f64| {
            TyTransformF64::new(
                TyVector3F64::new(x, y, z),
                TyQuaternionF64::identity(),
                TyVector3F64::new(1.0, 1.0, 1.0),
            )
        };

        // node 0 groups node 1, which places object 0 at +5x; node 2 places
        // object 1 at +3y. Nodes 0 and 2 are the roots.
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "group".to_owned(),
            child_nodes: vec![node(1)],
            child_objects: Vec::new(),
            transform: TyTransformF64::default(),
        });
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "wide".to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![object(0)],
            transform: placed_at(5.0, 0.0, 0.0),
        });
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "unit".to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![object(1)],
            transform: placed_at(0.0, 3.0, 0.0),
        });
        state.set_root_hierarchy_nodes(vec![node(0), node(2)]);

        state.validate().expect("a well-formed source state");
        state
    }

    /// Every live voxel as `(world position, color)`, walking the hierarchy from
    /// the roots and accumulating each node's translation. Order-independent and
    /// resolved per voxel, so a state compares to one round-tripped through a
    /// synthesized document without depending on object, palette, or voxel order.
    fn world_voxels(state: &VoxMain) -> BTreeSet<([i32; 3], [u8; 4])> {
        fn walk(
            state: &VoxMain,
            node: U32Id<BVoxHierarchyNode>,
            origin: [i32; 3],
            voxels: &mut BTreeSet<([i32; 3], [u8; 4])>,
        ) {
            let node = state.hierarchy_node(node).expect("a valid node");
            let position = node.transform.position;
            let translation = [
                origin[0] + position.x.round() as i32,
                origin[1] + position.y.round() as i32,
                origin[2] + position.z.round() as i32,
            ];
            for &object in &node.child_objects {
                let object = state.object(object).expect("a valid object");
                let color = object_color_ref(state, object);
                // A voxel sits at node-local `origin + grid`; these states use
                // identity rotation and unit scale, so the world position is the
                // accumulated translation plus that offset.
                let object_origin = object.origin();
                for voxel in object.iter_live() {
                    let grid = object.voxel_position(voxel).expect("within the grid");
                    let world = [
                        translation[0] + object_origin.x + grid.x as i32,
                        translation[1] + object_origin.y + grid.y as i32,
                        translation[2] + object_origin.z + grid.z as i32,
                    ];
                    let rgba = color.map_or([0, 0, 0, 0], |(reference, palette, attribute)| {
                        cell_color(object, voxel, reference, palette, attribute)
                    });
                    voxels.insert((world, rgba));
                }
            }
            for &child in &node.child_nodes {
                walk(state, child, translation, voxels);
            }
        }

        let mut voxels = BTreeSet::new();
        for &root in state.root_hierarchy_nodes() {
            walk(state, root, [0, 0, 0], &mut voxels);
        }
        voxels
    }

    /// A default state has no scene, so the writer synthesizes an empty document
    /// rather than erroring on the missing ext.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let state = VoxMain::default();
        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert!(file.scene_json_file.objects.is_empty());
        assert!(file.scene_json_file.groups.is_empty());
    }

    /// A state with no `voxel-max` ext, such as one cross-loaded from another
    /// format, synthesizes a document that `from_vmax_file` reads back with the
    /// same world geometry, colors, and placement.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let source = source_state();
        let file = to_vmax_file(&source, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(world_voxels(&reloaded), world_voxels(&source));
    }

    /// A foreign ext, here `magica-voxel`, is ignored by the Voxel Max writer: it
    /// synthesizes from the scene rather than failing to parse the wrong ext.
    #[test]
    fn synthesizes_past_a_foreign_ext() {
        let mut source = source_state();
        source.set_ext(Some(VoxValue::Object(VoxMap(vec![(
            "magica-voxel".to_owned(),
            VoxValue::Null,
        )]))));
        let file = to_vmax_file(&source, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(world_voxels(&reloaded), world_voxels(&source));
    }

    /// Synthesis gives every node a distinct `ind`, since Voxel Max collapses nodes
    /// that share the `[0, 0, 0]` path triple onto one. Groups take the `1` lane and
    /// objects the `0` lane.
    #[test]
    fn synthesizes_distinct_node_ind() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // A root group parenting two object nodes.
        state.add_hierarchy_node(group_node("g", &[1, 2], at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("a", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("b", 1, at(5.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let scene = &file.scene_json_file;
        let inds: Vec<[i64; 3]> = scene
            .groups
            .iter()
            .map(|g| g.ind)
            .chain(scene.objects.iter().map(|o| o.ind))
            .collect();
        let distinct: BTreeSet<[i64; 3]> = inds.iter().copied().collect();
        assert_eq!(distinct.len(), inds.len(), "every node ind is distinct");
        assert!(scene.groups.iter().all(|g| g.ind[1] == 1));
        assert!(scene.objects.iter().all(|o| o.ind[1] == 0));
    }

    /// A node placing several objects, as a Goxel layer's blocks do, keeps every
    /// object: Voxel Max models one object per node, so synthesis flattens the
    /// extra objects to siblings that share the node's placement rather than
    /// dropping all but the first.
    #[test]
    fn synthesizes_a_node_placing_several_objects() {
        let mut state = VoxMain::default();

        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        for hex in ["#FF0000FF", "#00FF00FF"] {
            palette
                .add_cell(vec![VoxValue::Text(hex.to_owned())])
                .expect("one value per attribute");
        }
        let palette_id = state.add_palette(palette);
        let cell = |index: u32| U32Id::<BVoxPaletteCell>::from_u32(index);

        // Two unit objects, a red one (cell 0) and a green one (cell 1).
        for color in [0u32, 1] {
            let mut object = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
                .expect("a 1x1x1 grid is within the dense limit");
            object.add_palette_ref(palette_id, cell(0));
            let voxel = object
                .voxel_id(TyVector3U32::new(0, 0, 0))
                .expect("a position within the grid");
            object
                .retain_voxel(voxel, &[cell(color)])
                .expect("one sample for the one reference");
            state.add_object(object);
        }

        // One node placing both objects at the same offset, so they coincide in
        // the world at distinct colors.
        let object = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "layer".to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![object(0), object(1)],
            transform: TyTransformF64::new(
                TyVector3F64::new(10.0, 0.0, 0.0),
                TyQuaternionF64::identity(),
                TyVector3F64::new(1.0, 1.0, 1.0),
            ),
        });
        state.set_root_hierarchy_nodes(vec![node(0)]);
        state.validate().expect("a well-formed source state");

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file.scene_json_file.objects.len(), 2);
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([10, 0, 0], red), ([10, 0, 0], green)])
        );
    }

    /// An `rgba` palette holding the given colors.
    fn rgba_palette(hexes: &[&str]) -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        for hex in hexes {
            palette
                .add_cell(vec![VoxValue::Text((*hex).to_owned())])
                .expect("one value per attribute");
        }
        palette
    }

    /// An object of `bounds` whose live voxels each sample one color cell, given as
    /// `(position, cell)` pairs.
    fn color_object(
        palette: U32Id<BVoxPalette>,
        bounds: TyVector3U32,
        voxels: &[([u32; 3], u32)],
    ) -> VoxObject {
        let mut object = VoxObject::new(String::new(), bounds).expect("within the dense limit");
        object.add_palette_ref(palette, U32Id::<BVoxPaletteCell>::from_u32(0));
        for &([x, y, z], cell) in voxels {
            let id = object
                .voxel_id(TyVector3U32::new(x, y, z))
                .expect("a position within the grid");
            object
                .retain_voxel(id, &[U32Id::<BVoxPaletteCell>::from_u32(cell)])
                .expect("one sample for the one reference");
        }
        object
    }

    /// A translation-only transform.
    fn at(x: f64, y: f64, z: f64) -> TyTransformF64 {
        TyTransformF64::new(
            TyVector3F64::new(x, y, z),
            TyQuaternionF64::identity(),
            TyVector3F64::new(1.0, 1.0, 1.0),
        )
    }

    /// A node placing object `object` at `transform`.
    fn object_node(name: &str, object: u32, transform: TyTransformF64) -> VoxHierarchyNode {
        VoxHierarchyNode {
            name: name.to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![U32Id::<BVoxObject>::from_u32(object)],
            transform,
        }
    }

    /// A group node parenting the given child nodes at `transform`.
    fn group_node(name: &str, children: &[u32], transform: TyTransformF64) -> VoxHierarchyNode {
        VoxHierarchyNode {
            name: name.to_owned(),
            child_nodes: children
                .iter()
                .map(|&c| U32Id::<BVoxHierarchyNode>::from_u32(c))
                .collect(),
            child_objects: Vec::new(),
            transform,
        }
    }

    /// The voxels decoded from a contents file's snapshots.
    fn contents_voxels(file: &VMaxFile, name: &str) -> Vec<VMaxVoxel> {
        decode_vmax_snapshots(&file.contents_files[name].snapshots).expect("decodable snapshots")
    }

    /// The image stores colors 0-based then a transparent terminator, and voxels
    /// take 1-based indices: the first color sits at image index 0, the terminator
    /// at 255, and each voxel's color_idx is its cell + 1.
    #[test]
    fn synthesizes_colors_zero_based_with_a_terminator() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let png = &file.palette_png_files["palette.png"].0;
        assert_eq!(png.len(), 256);
        assert_eq!(png[0], [0xFF, 0, 0, 0xFF]);
        assert_eq!(png[1], [0, 0xFF, 0, 0xFF]);
        assert_eq!(png[255], [0, 0, 0, 0]);
        let mut indices: Vec<u8> = contents_voxels(&file, "contents.vmaxb")
            .iter()
            .map(|voxel| voxel.color_idx)
            .collect();
        indices.sort_unstable();
        assert_eq!(indices, [1, 2]);
    }

    /// A voxel on the first color (index 1) and one on the last (index 255)
    /// round-trip through a plist palette, whose `colors` table is 0-based. Guards
    /// the bug a real Voxel Max plist file hit: the last color read back as out of
    /// range.
    #[test]
    fn round_trips_first_and_last_color_through_plist() {
        let mut state = VoxMain::default();
        // 255 colors: red first, blue last, green between.
        let mut hexes = vec!["#FF0000FF".to_owned()];
        hexes.extend((0..253).map(|_| "#00FF00FF".to_owned()));
        hexes.push("#0000FFFF".to_owned());
        let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
        let palette = state.add_palette(rgba_palette(&refs));
        // A voxel on the first color (cell 0) and one on the last (cell 254).
        state.add_object(color_object(
            palette,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 254)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Plist).unwrap();
        // The plist colors are 0-based: red first, blue last.
        let colors: Vec<[u8; 4]> = file.palette_settings_files["palette.settings.vmaxpsb"]
            .colors
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        assert_eq!(colors.len(), 255);
        assert_eq!(colors[0], [0xFF, 0, 0, 0xFF]);
        assert_eq!(colors[254], [0, 0, 0xFF, 0xFF]);
        // Voxels carry 1-based indices: the first color is 1, the last is 255.
        let mut indices: Vec<u8> = contents_voxels(&file, "contents.vmaxb")
            .iter()
            .map(|voxel| voxel.color_idx)
            .collect();
        indices.sort_unstable();
        assert_eq!(indices, [1, 255]);
        // It reads back without the out-of-range error, resolving to red and blue.
        let reloaded = from_vmax_file(&file).unwrap();
        let resolved: BTreeSet<[u8; 4]> = world_voxels(&reloaded)
            .into_iter()
            .map(|(_, rgba)| rgba)
            .collect();
        assert_eq!(
            resolved,
            BTreeSet::from([[0xFF, 0, 0, 0xFF], [0, 0, 0xFF, 0xFF]])
        );
    }

    /// A palette with more colors than fit is rejected rather than silently
    /// truncated or wrapped: 255 colors is the budget and round-trips, 256 overflows.
    #[test]
    fn errors_when_colors_exceed_palette_budget() {
        let synthesize = |count: u32| {
            let mut state = VoxMain::default();
            let hexes: Vec<String> = (0..count).map(|i| format!("#{:06X}FF", i)).collect();
            let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
            let palette = state.add_palette(rgba_palette(&refs));
            let voxels: Vec<([u32; 3], u32)> = (0..count).map(|i| ([i, 0, 0], i)).collect();
            state.add_object(color_object(
                palette,
                TyVector3U32::new(count, 1, 1),
                &voxels,
            ));
            state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
            state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
            state.validate().unwrap();
            to_vmax_file(&state, VoxelMaxColorFormat::Png)
        };

        let file = synthesize(255).expect("255 colors fit the palette");
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            reloaded.object(U32Id::from_u32(0)).unwrap().live_count(),
            255
        );
        assert!(synthesize(256).is_err());
    }

    /// An object with no color palette borrows the default palette name rather than
    /// an empty `pal` Voxel Max cannot resolve, and writes no file of its own. The
    /// colored and empty objects share that name.
    #[test]
    fn synthesizes_a_colorless_object_sharing_the_default_palette() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        // Object 0: a single red voxel. Object 1: an empty, colorless object,
        // whose tight grid is [0, 0, 0].
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_object(VoxObject::new(String::new(), TyVector3U32::new(0, 0, 0)).unwrap());
        state.add_hierarchy_node(object_node("colored", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("colorless", 1, at(10.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        // Both objects name the one real palette; no extra or placeholder file.
        let names: BTreeSet<&str> = file
            .scene_json_file
            .objects
            .iter()
            .map(|object| object.palette.as_str())
            .collect();
        assert_eq!(names, BTreeSet::from(["palette.png"]));
        assert_eq!(
            file.palette_png_files.keys().collect::<Vec<_>>(),
            ["palette.png"]
        );

        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(world_voxels(&reloaded), BTreeSet::from([([0, 0, 0], red)]));
    }

    /// A colorless object's live voxels take a non-empty index (the borrowed
    /// palette's first color), not the empty index 0, so they do not vanish.
    #[test]
    fn synthesizes_colorless_voxels_on_a_non_empty_index() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        // Object 0: a colored voxel, so a `palette.png` exists to borrow. Object 1:
        // a colorless object that nonetheless has a live voxel.
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        let mut colorless = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1)).unwrap();
        let voxel = colorless.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        colorless.retain_voxel(voxel, &[]).unwrap();
        state.add_object(colorless);
        state.add_hierarchy_node(object_node("colored", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("colorless", 1, at(10.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        // No live voxel, colored or colorless, lands on the empty index 0.
        let indices: Vec<u8> = file
            .contents_files
            .values()
            .flat_map(|contents| decode_vmax_snapshots(&contents.snapshots).unwrap())
            .map(|voxel| voxel.color_idx)
            .collect();
        assert!(!indices.is_empty());
        assert!(indices.iter().all(|&index| index >= 1));
    }

    /// A node placing several objects and also parenting child nodes flattens to
    /// sibling object-nodes sharing the node's placement, with the child nodes
    /// hanging off the first object, and every object gets its own files.
    #[test]
    fn synthesizes_a_node_placing_objects_and_child_nodes() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF", "#0000FFFF"]));
        for cell in 0..3 {
            state.add_object(color_object(
                palette,
                TyVector3U32::new(1, 1, 1),
                &[([0, 0, 0], cell)],
            ));
        }
        // A fourth object placed by a child node, to confirm it hangs off the
        // first object of the multi-object parent.
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // node 0 places objects 0, 1, 2 at +10x and parents node 1; node 1 places
        // object 3 at +1y of the first object.
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "layer".to_owned(),
            child_nodes: vec![U32Id::<BVoxHierarchyNode>::from_u32(1)],
            child_objects: vec![
                U32Id::<BVoxObject>::from_u32(0),
                U32Id::<BVoxObject>::from_u32(1),
                U32Id::<BVoxObject>::from_u32(2),
            ],
            transform: at(10.0, 0.0, 0.0),
        });
        state.add_hierarchy_node(object_node("child", 3, at(0.0, 1.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        // Four scene objects: three from the multi-object node, one from the child.
        assert_eq!(file.scene_json_file.objects.len(), 4);
        // The three siblings share the parent's id, and ids are all distinct.
        let ids: BTreeSet<&str> = file
            .scene_json_file
            .objects
            .iter()
            .map(|object| object.id.as_str())
            .collect();
        assert_eq!(ids.len(), 4);

        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        let blue = [0, 0, 0xFF, 0xFF];
        // The three siblings keep their place. The child node hangs off the first
        // object, and a node placed under an object anchors at that object's
        // content-center pivot, not its grid corner. The object re-centers on reload
        // (its origin becomes round(box_min - center)), so its descendant shifts by
        // that origin, here to [11, 2, 1]. This only arises for object-under-object
        // nesting, which other formats such as Goxel produce; Voxel Max objects are
        // leaves, so a Voxel Max round-trip is unaffected.
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([
                ([10, 0, 0], red),
                ([10, 0, 0], green),
                ([10, 0, 0], blue),
                ([11, 2, 1], red),
            ])
        );
    }

    /// Two nodes placing one object instance it: the object's voxels are rebuilt
    /// once into a shared contents file, the scene carries two objects with
    /// distinct ids, and reloading collapses them back to one voxcore object placed
    /// at both positions.
    #[test]
    fn synthesizes_instances_sharing_one_contents_file() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_hierarchy_node(object_node("a", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("b", 0, at(20.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file.contents_files.len(), 1);
        assert_eq!(file.scene_json_file.objects.len(), 2);
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 1);
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], red), ([20, 0, 0], red)])
        );
    }

    /// A subtree shared by several parents is placed once per parent: synthesis
    /// duplicates it per path so the rebuilt world matches voxcore, where a node's
    /// placement composes along every path to it.
    #[test]
    fn synthesizes_a_shared_subtree_at_every_parent() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // node 2 places object 0 at +1x and is a child of both group node 0 (at
        // the origin) and group node 1 (at +100x).
        state.add_hierarchy_node(group_node("a", &[2], at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(group_node("b", &[2], at(100.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("c", 0, at(1.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([1, 0, 0], red), ([101, 0, 0], red)])
        );
    }

    /// A node that is both a root and a child is placed at both, since voxcore
    /// renders it along each path; synthesis emits it once per path rather than
    /// dropping its root placement.
    #[test]
    fn synthesizes_a_node_that_is_root_and_child() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // node 1 places object 0 at +1x; it is both a root and a child of group
        // node 0 at +100x.
        state.add_hierarchy_node(group_node("a", &[1], at(100.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("c", 0, at(1.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([1, 0, 0], red), ([101, 0, 0], red)])
        );
    }

    /// A node reachable from no root is dropped, since voxcore never places it, so
    /// synthesis does not promote it to a spurious root.
    #[test]
    fn drops_a_node_unreachable_from_the_roots() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // node 0 is a root placing object 0; node 1 places object 1 but is neither
        // a root nor anyone's child.
        state.add_hierarchy_node(object_node("rooted", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("orphan", 1, at(50.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file.scene_json_file.objects.len(), 1);
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(world_voxels(&reloaded), BTreeSet::from([([0, 0, 0], red)]));
    }

    /// Synthesis encodes the node's quaternion rotation back to `t_r`, so rotation
    /// survives alongside translation and scale; the lone voxel still renders at its
    /// original world point.
    #[test]
    fn keeps_rotation_scale_and_translation() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        let rotation =
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), 90f64.to_radians());
        let transform = TyTransformF64::new(
            TyVector3F64::new(7.0, 0.0, 0.0),
            rotation,
            TyVector3F64::new(2.0, 1.0, 1.0),
        );
        state.add_hierarchy_node(object_node("o", 0, transform));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let node = reloaded.hierarchy_node(U32Id::from_u32(0)).unwrap();

        // The axis-angle round-trip reproduces the rotation to floating-point noise.
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        let r = node.transform.rotation;
        assert!(close(r.x, rotation.x) && close(r.y, rotation.y));
        assert!(close(r.z, rotation.z) && close(r.w, rotation.w));
        assert_eq!(node.transform.scale, TyVector3F64::new(2.0, 1.0, 1.0));

        // node.position is the content-center pivot and the rotation now applies to
        // the grid, so the lone voxel still renders at its original world point:
        // position + R * (scale * (origin + local)), with local at the grid corner.
        let object = reloaded.object(U32Id::from_u32(0)).unwrap();
        let origin = object.origin();
        let s = node.transform.scale;
        let local = TyVector3F64::new(
            s.x * origin.x as f64,
            s.y * origin.y as f64,
            s.z * origin.z as f64,
        );
        let world = node.transform.position + r.rotate(local);
        assert!(close(world.x, 7.0) && close(world.y, 0.0) && close(world.z, 0.0));
    }

    /// A material palette survives synthesis across every color format, and the
    /// material index is not shifted by the color offset.
    #[test]
    fn synthesizes_material_palettes_across_color_formats() {
        let formats = [
            VoxelMaxColorFormat::Png,
            VoxelMaxColorFormat::Plist,
            VoxelMaxColorFormat::All,
        ];
        for format in formats {
            let mut state = VoxMain::default();
            let color = state.add_palette(rgba_palette(&["#FF0000FF"]));
            let mut materials = VoxPalette::default();
            for attribute in ["metallic", "roughness", "emissive", "shadows"] {
                materials.add_attribute(attribute.to_owned());
            }
            materials
                .add_cell(vec![
                    VoxValue::Number(0.5),
                    VoxValue::Number(0.25),
                    VoxValue::Number(2.0),
                    VoxValue::Bool(true),
                ])
                .unwrap();
            let material = state.add_palette(materials);
            let mut object = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1)).unwrap();
            object.add_palette_ref(color, U32Id::<BVoxPaletteCell>::from_u32(0));
            object.add_palette_ref(material, U32Id::<BVoxPaletteCell>::from_u32(0));
            let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
            object
                .retain_voxel(
                    voxel,
                    &[
                        U32Id::<BVoxPaletteCell>::from_u32(0),
                        U32Id::<BVoxPaletteCell>::from_u32(0),
                    ],
                )
                .unwrap();
            state.add_object(object);
            state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
            state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
            state.validate().unwrap();

            let file = to_vmax_file(&state, format).unwrap();
            // Materials are not offset: the one voxel keeps material index 0.
            assert!(
                contents_voxels(&file, "contents.vmaxb")
                    .iter()
                    .all(|voxel| voxel.material_idx == 0)
            );

            let reloaded = from_vmax_file(&file).unwrap();
            let material = reloaded
                .iter_palettes()
                .map(|(_, palette)| palette)
                .find(|palette| {
                    palette
                        .iter_attributes()
                        .any(|(_, name)| name == "metallic")
                })
                .expect("a material palette survives");
            let cell = material.iter_cells().next().expect("one material cell");
            let value = |name: &str| {
                let id = material
                    .iter_attributes()
                    .find(|(_, attribute)| *attribute == name)
                    .map(|(id, _)| id)
                    .unwrap();
                material.cell_value(cell, id).cloned()
            };
            assert_eq!(value("metallic"), Some(VoxValue::Number(0.5)));
            assert_eq!(value("roughness"), Some(VoxValue::Number(0.25)));
            assert_eq!(value("emissive"), Some(VoxValue::Number(2.0)));
            assert_eq!(value("shadows"), Some(VoxValue::Bool(true)));
        }
    }

    /// An `rgb` source palette carrying 6-hex colors widens to opaque RGBA: the
    /// missing alpha defaults to fully opaque rather than transparent.
    #[test]
    fn widens_rgb_source_to_opaque() {
        let mut state = VoxMain::default();
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgb".to_owned());
        palette
            .add_cell(vec![VoxValue::Text("#3366CC".to_owned())])
            .unwrap();
        let palette = state.add_palette(palette);
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(
            file.palette_png_files["palette.png"].0[0],
            [0x33, 0x66, 0xCC, 0xFF]
        );
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], [0x33, 0x66, 0xCC, 0xFF])])
        );
    }

    /// Writing a colored single-object scene with no ext, reloading it, then
    /// writing it again reaches a fixed point: the second write takes the lossless
    /// path and reproduces the first document exactly.
    #[test]
    fn is_idempotent_for_a_colored_object() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(3.0, 4.0, 5.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file1 = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file1).unwrap();
        let file2 = to_vmax_file(&reloaded, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file2, file1);
    }

    /// A deep hierarchy of nested groups and object nodes round-trips: every leaf's
    /// world voxel survives the reconstructed parent chain.
    #[test]
    fn synthesizes_a_deep_hierarchy() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 1)],
        ));
        // root group 0 -> group 1 -> group 2 -> object node 3, plus object node 4
        // hanging off group 1, so leaves sit at different depths.
        state.add_hierarchy_node(group_node("r", &[1], at(1.0, 0.0, 0.0)));
        state.add_hierarchy_node(group_node("g1", &[2, 4], at(0.0, 2.0, 0.0)));
        state.add_hierarchy_node(group_node("g2", &[3], at(0.0, 0.0, 3.0)));
        state.add_hierarchy_node(object_node("leaf", 0, at(10.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("mid", 1, at(0.0, 20.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([11, 2, 3], red), ([1, 22, 0], green)])
        );
    }

    /// A group's content box is derived from its subtree, not stored: the union, in
    /// the group's own frame, of each child object's content box mapped through the
    /// placing node's transform. Two equally-sized children at different offsets
    /// union to a box spanning both.
    #[test]
    fn derives_a_group_content_box_from_its_subtree() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        // Two [2, 2, 2] objects, each tight (voxels reach both ends of every axis).
        for _ in 0..2 {
            state.add_object(color_object(
                palette,
                TyVector3U32::new(2, 2, 2),
                &[([0, 0, 0], 0), ([1, 1, 1], 0)],
            ));
        }
        // A root group parenting two object nodes, the second offset +10x.
        state.add_hierarchy_node(group_node("g", &[1, 2], at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("a", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("b", 1, at(10.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let group = file
            .scene_json_file
            .groups
            .iter()
            .find(|g| g.name == "g")
            .expect("the root group");
        // Each object box is [0, 0, 0]..[2, 2, 2] in its node-local frame, centered
        // on [1, 1, 1]; object b is shifted +10x, so the union spans x [0, 12] and
        // y/z [0, 2], centered on [6, 1, 1] with half-extents [6, 1, 1].
        assert_eq!(group.center, [6.0, 1.0, 1.0]);
        assert_eq!(group.bounds_min, Some([-6.0, -1.0, -1.0]));
        assert_eq!(group.bounds_max, Some([6.0, 1.0, 1.0]));
    }

    /// An empty object synthesizes to a scene object with empty contents and a
    /// content box framed on its grid. Reloading keeps the `[3, 4, 5]` build volume
    /// as the object's grid; with no live voxels its derived runtime extent is empty.
    #[test]
    fn synthesizes_an_empty_object_with_bounds() {
        let mut state = VoxMain::default();
        // An empty object holds no voxels; its grid is the [3, 4, 5] build volume.
        let object = VoxObject::new(String::new(), TyVector3U32::new(3, 4, 5)).unwrap();
        state.add_object(object);
        state.add_hierarchy_node(object_node("empty", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file.scene_json_file.objects.len(), 1);
        assert!(contents_voxels(&file, "contents.vmaxb").is_empty());
        // The content box frames the [3, 4, 5] build volume, centered in the 256
        // workspace: e_c at its center, e_ma the half-extents.
        assert_eq!(file.scene_json_file.objects[0].center, [127.5, 128.0, 2.5]);
        assert_eq!(
            file.scene_json_file.objects[0].bounds_max,
            Some([1.5, 2.0, 2.5])
        );
        // Reloading keeps the [3, 4, 5] build volume as the object's grid; with no
        // live voxels its derived runtime extent is empty.
        let reloaded = from_vmax_file(&file).unwrap();
        let id = U32Id::<BVoxObject>::from_u32(0);
        assert_eq!(
            reloaded.object(id).unwrap().bounds(),
            TyVector3U32::new(3, 4, 5)
        );
        assert_eq!(reloaded.object(id).unwrap().live_extent(), None);
    }

    /// A fully transparent color round-trips as a real color at image index 0
    /// rather than being mistaken for the trailing terminator.
    #[test]
    fn round_trips_a_transparent_color() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#11223300"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        // The color sits at index 0 (cell 0); the terminator is at the end.
        assert_eq!(
            file.palette_png_files["palette.png"].0[0],
            [0x11, 0x22, 0x33, 0]
        );
        assert_eq!(file.palette_png_files["palette.png"].0[255], [0, 0, 0, 0]);
        assert!(
            contents_voxels(&file, "contents.vmaxb")
                .iter()
                .all(|v| v.color_idx == 1)
        );
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], [0x11, 0x22, 0x33, 0])])
        );
    }

    /// A padded source palette larger than the budget is fine as long as its
    /// referenced colors fit, matching a MagicaVoxel source whose fixed 256-entry
    /// palette uses only a few low indices. The size alone must not be rejected.
    #[test]
    fn synthesizes_a_padded_palette_using_low_indices() {
        let mut state = VoxMain::default();
        let hexes: Vec<String> = (0..256).map(|i| format!("#{:06X}FF", i)).collect();
        let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
        let palette = state.add_palette(rgba_palette(&refs));
        // One voxel references a low cell; the other 255 cells are padding.
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 5)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], [0x00, 0x00, 0x05, 0xFF])])
        );
    }

    /// A color-only object keeps its colors in plist mode, where the colors ride in
    /// the settings sidecar rather than an image, instead of reloading as white.
    #[test]
    fn synthesizes_a_color_only_object_in_plist() {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Plist).unwrap();
        // No image in plist mode; the colors must still survive via the sidecar.
        assert!(file.palette_png_files.is_empty());
        assert!(!file.palette_settings_files.is_empty());
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], red), ([1, 0, 0], green)])
        );
    }

    /// A minimal no-ext state: one red voxel placed by one object node, the input
    /// the synthesis path takes.
    fn one_object_state() -> VoxMain {
        let mut state = VoxMain::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();
        state
    }

    /// Gives the state a `voxel-max` ext carrying `cam` as its scene camera, so the
    /// lossless path has a camera to keep or replace.
    fn with_ext_scene_camera(state: &mut VoxMain, cam: VMaxSceneCamera) {
        let voxel_max = VoxelMaxExt {
            scene: VMaxSceneJsonFile {
                cam: Some(cam),
                ..Default::default()
            },
            palettes: vec![None; state.palette_count()],
            ..Default::default()
        };
        let ext = to_vox_value(&VoxelMaxExtWrapper { voxel_max }).unwrap();
        state.set_ext(Some(ext));
        state.validate().unwrap();
    }

    /// `SceneCameraSource::Camera` writes the given scene camera, replacing whatever
    /// the path would otherwise produce.
    #[test]
    fn scene_camera_camera_overrides_the_scene_camera() {
        let state = one_object_state();
        let camera = VMaxSceneCamera {
            z: 123.0,
            ..Default::default()
        };
        let file = VmaxFileBuilder::new(&state)
            .scene_camera(SceneCameraSource::Camera(camera))
            .build()
            .unwrap();
        assert_eq!(file.scene_json_file.cam, Some(camera));
    }

    /// `SceneCameraSource::Ext` errors on a state with no `voxel-max` ext, since
    /// there is no camera to keep, but succeeds and keeps the camera once one is
    /// present.
    #[test]
    fn scene_camera_ext_requires_an_ext() {
        let mut state = one_object_state();
        assert!(
            VmaxFileBuilder::new(&state)
                .scene_camera(SceneCameraSource::Ext)
                .build()
                .is_err()
        );

        let cam = VMaxSceneCamera {
            z: 321.0,
            ..Default::default()
        };
        with_ext_scene_camera(&mut state, cam);
        let file = VmaxFileBuilder::new(&state)
            .scene_camera(SceneCameraSource::Ext)
            .build()
            .unwrap();
        assert_eq!(file.scene_json_file.cam, Some(cam));
    }

    /// `SceneCameraSource::Empty` replaces the ext's scene camera with the default,
    /// while leaving the camera unset keeps it.
    #[test]
    fn scene_camera_empty_replaces_the_ext_camera() {
        let mut state = one_object_state();
        let cam = VMaxSceneCamera {
            z: 999.0,
            ..Default::default()
        };
        with_ext_scene_camera(&mut state, cam);

        let kept = VmaxFileBuilder::new(&state).build().unwrap();
        assert_eq!(kept.scene_json_file.cam, Some(cam));

        let empty = VmaxFileBuilder::new(&state)
            .scene_camera(SceneCameraSource::Empty)
            .build()
            .unwrap();
        assert_ne!(empty.scene_json_file.cam, Some(cam));
    }
}
