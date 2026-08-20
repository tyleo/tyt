use crate::{
    Error, Result, vox_hierarchy_node_from_voxj_hierarchy_node,
    vox_object_from_voxj_decoded_object, vox_palette_from_voxj_palette, vox_value_from_voxj_value,
    vox_value_pool_from_voxj_value_pool,
};
use branded_id::U32Id;
use voxcore::VoxMain;
use voxj::{VoxjFile, VoxjValue};
use voxj_codec::{decode_voxj_object, voxj_palette_material_counts};

/// Loads a [`VoxjFile`] into a [`VoxMain`]. Each object's position and sample
/// blocks are decoded, then entities take ids in listing order, so each id
/// equals its voxj array index and cross-references carry over. The nodes
/// land as one batch, since the wire permits a node to list a child that
/// appears later.
///
/// Errors if:
///
/// 1. a block is malformed
/// 2. object geometry is malformed
/// 3. a checked insertion rejects a cross-reference
pub fn from_voxj_file(file: &VoxjFile) -> Result<VoxMain> {
    let main = &file.main;
    let mut state = VoxMain::default();

    // Build each value before adding it so a failed conversion leaves the state
    // untouched. Value pools land first, so palette properties resolve against
    // them.
    for value_pool in &main.runtime_state.value_pools {
        state.retain_value_pool(vox_value_pool_from_voxj_value_pool(value_pool)?);
    }

    // An insertion names the entity it rejected by the ids it holds, which are
    // internal to the palette or object; the listing index is what points back
    // into the document.
    for (index, palette) in main.runtime_state.palettes.iter().enumerate() {
        state
            .retain_palette(vox_palette_from_voxj_palette(palette)?)
            .map_err(|error| Error::invalid(format!("palette {index}: {error}")))?;
    }

    for (index, object) in main.runtime_state.objects.iter().enumerate() {
        let material_counts =
            voxj_palette_material_counts(&object.layers, &main.runtime_state.palettes)?;
        let decoded = decode_voxj_object(object, &material_counts)?;
        // The build volume, present only when the document recorded margin
        // around the object's live voxels; otherwise the runtime grid is the
        // build volume.
        let edit = main
            .edit_state
            .as_ref()
            .and_then(|e| e.objects.get(index))
            .map(|edit| (edit.bounds, edit.origin));
        let vox_object = vox_object_from_voxj_decoded_object(&decoded, edit)?;
        state
            .retain_object(vox_object)
            .map_err(|error| Error::invalid(format!("object {index}: {error}")))?;
    }

    let nodes = main
        .runtime_state
        .nodes
        .iter()
        .map(vox_hierarchy_node_from_voxj_hierarchy_node)
        .collect::<Result<Vec<_>>>()?;
    state.retain_hierarchy_nodes(nodes)?;

    state.set_root_hierarchy_node_ids(
        main.runtime_state
            .root_nodes
            .iter()
            .map(|&index| U32Id::from_u32(index as u32))
            .collect(),
    )?;

    state.set_ext(
        main.ext
            .clone()
            .map(|map| vox_value_from_voxj_value(&VoxjValue::Object(map)))
            .transpose()?,
    );

    Ok(state)
}

#[cfg(test)]
mod tests {
    use crate::{
        EditStateMode, VoxjFileBuilder, from_voxj_bytes, from_voxj_file, to_voxj_bytes,
        to_voxj_file, to_voxjz_bytes,
    };
    use branded_id::U32Id;
    use std::{collections::BTreeSet, f64::consts::FRAC_1_SQRT_2};
    use voxcore::{BVoxLayer, BVoxObject, BVoxPalette};
    use voxj::{
        VoxjEditObject, VoxjEditState, VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjMap, VoxjObject,
        VoxjPalette, VoxjPositionBlock, VoxjProperty, VoxjRuntimeState, VoxjSampleBlock,
        VoxjTransform, VoxjValue, VoxjValuePool,
    };
    use voxj_codec::{decode_voxj_object, voxj_palette_material_counts};

    /// A `float` value pool of ascending values `0.0 ..= (n - 1)`, so a
    /// material reading value-index `m` resolves to `m` as a float.
    fn numbered_value_pool(n: usize) -> VoxjValuePool {
        VoxjValuePool::Float((0..n).map(|i| i as f64).collect())
    }

    /// A one-property palette over `value_pool` with `n` materials,
    /// material `m` reading value-index `m`, one row `[m]` per material.
    fn numbered_palette(name: &str, value_pool: usize, n: usize) -> VoxjPalette {
        VoxjPalette {
            properties: vec![property(name, value_pool)],
            materials: (0..n).map(|m| vec![m]).collect(),
        }
    }

    fn property(name: &str, value_pool: usize) -> VoxjProperty {
        VoxjProperty {
            name: name.to_owned(),
            value_pool,
        }
    }

    /// One object holding raw-json position and sample blocks, the readable
    /// form the fixtures author geometry in. `samples` is one row of material
    /// indices per voxel, one entry per layer; transposed into the per-layer
    /// channels the sample block stores, their count read off the row arity.
    fn object(
        name: &str,
        layers: Vec<usize>,
        bounds: [u32; 3],
        positions: Vec<[u32; 3]>,
        samples: Vec<Vec<u32>>,
    ) -> VoxjObject {
        let channel_count = samples.first().map_or(0, Vec::len);
        let channels = (0..channel_count)
            .map(|p| samples.iter().map(|row| row[p]).collect())
            .collect();
        VoxjObject {
            name: name.to_owned(),
            layers,
            bounds,
            origin: [0, 0, 0],
            voxel_positions: VoxjPositionBlock::RawJson(positions),
            voxel_samples: VoxjSampleBlock::RawJson(channels),
        }
    }

    /// A document exercising every field.
    fn sample_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![
                        // value pool 0: six base values, bound by palette 0.
                        numbered_value_pool(6),
                        // value pool 1: metallic values.
                        VoxjValuePool::Float(vec![0.0, 0.5, 1.0]),
                        // value pool 2: ior values.
                        VoxjValuePool::Float(vec![1.5, 2.0]),
                    ],
                    palettes: vec![
                        numbered_palette("baseColor", 0, 6),
                        // Two properties over the scalar value pools, one row
                        // per material: material 0 = { metallic: 0.0,
                        // ior: 1.5 }, material 1 = { metallic: 0.5,
                        // ior: 2.0 }.
                        VoxjPalette {
                            properties: vec![property("metallic", 1), property("ior", 2)],
                            materials: vec![vec![0, 0], vec![1, 1]],
                        },
                    ],
                    objects: vec![
                        object(
                            "sparse",
                            vec![0],
                            [4, 4, 4],
                            vec![[0, 0, 0], [3, 1, 2], [1, 3, 0], [2, 2, 3]],
                            vec![vec![1], vec![0], vec![5], vec![2]],
                        ),
                        // Two layers sharing palette 1: layers do not merge, so
                        // each voxel samples a material index per layer.
                        object(
                            "tight",
                            vec![1, 1],
                            [2, 1, 1],
                            vec![[0, 0, 0], [1, 0, 0]],
                            vec![vec![1, 0], vec![0, 1]],
                        ),
                        object(
                            "no-palette",
                            Vec::new(),
                            [3, 1, 2],
                            vec![[0, 0, 0], [2, 0, 1]],
                            vec![Vec::new(), Vec::new()],
                        ),
                    ],
                    nodes: vec![
                        VoxjHierarchyNode {
                            name: "group".to_owned(),
                            child_nodes: vec![1],
                            child_objects: vec![0],
                            transform: VoxjTransform {
                                position: [1.0, 2.0, 3.0],
                                rotation: [0.0, 0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2],
                                scale: [2.0, 2.0, 2.0],
                            },
                        },
                        VoxjHierarchyNode {
                            name: "leaf".to_owned(),
                            child_nodes: Vec::new(),
                            child_objects: vec![1],
                            transform: VoxjTransform {
                                position: [0.0, 0.0, 0.0],
                                rotation: [0.0, 0.0, 0.0, 1.0],
                                scale: [1.0, 1.0, 1.0],
                            },
                        },
                    ],
                    root_nodes: vec![0],
                },
                edit_state: None,
                ext: Some(VoxjMap(vec![(
                    "vendor".to_owned(),
                    VoxjValue::Array(vec![
                        VoxjValue::Number(1.0),
                        VoxjValue::Bool(true),
                        VoxjValue::Null,
                        VoxjValue::Text("x".to_owned()),
                    ]),
                )])),
            },
        }
    }

    /// [`sample_file`] after removing the "tight" object (id 1) and the palette
    /// only it referenced (id 1), then compacting: the two survivors renumber
    /// to objects 0 and 1, the lone palette stays 0, and the "leaf" node loses
    /// its reference to the removed object. The value pools are untouched:
    /// there is no value-pool removal, so the now-unreferenced value pools stay
    /// in place.
    fn sample_file_without_tight() -> VoxjFile {
        let base = sample_file();
        VoxjFile {
            version: base.version,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: base.main.runtime_state.value_pools.clone(),
                    objects: vec![
                        base.main.runtime_state.objects[0].clone(),
                        base.main.runtime_state.objects[2].clone(),
                    ],
                    palettes: vec![base.main.runtime_state.palettes[0].clone()],
                    nodes: vec![
                        base.main.runtime_state.nodes[0].clone(),
                        VoxjHierarchyNode {
                            child_objects: Vec::new(),
                            ..base.main.runtime_state.nodes[1].clone()
                        },
                    ],
                    root_nodes: vec![0],
                },
                edit_state: None,
                ext: base.main.ext.clone(),
            },
        }
    }

    /// The `(position, samples)` pairs of an object, order-independent: the
    /// dense grid re-emits voxels in raster order and the block encodings
    /// reorder them again. The blocks decode against the document palettes.
    fn voxel_set(object: &VoxjObject, palettes: &[VoxjPalette]) -> BTreeSet<([u32; 3], Vec<u32>)> {
        let material_counts = voxj_palette_material_counts(&object.layers, palettes).unwrap();
        let decoded = decode_voxj_object(object, &material_counts).unwrap();
        decoded
            .positions
            .iter()
            .copied()
            .zip(decoded.samples.iter().cloned())
            .collect()
    }

    fn assert_file_eq(got: &VoxjFile, want: &VoxjFile) {
        assert_eq!(got.main.edit_state, want.main.edit_state);
        assert_eq!(got.main.ext, want.main.ext);
        let (got, want) = (&got.main.runtime_state, &want.main.runtime_state);
        assert_eq!(got.value_pools, want.value_pools);
        assert_eq!(got.palettes, want.palettes);
        assert_eq!(got.nodes, want.nodes);
        assert_eq!(got.root_nodes, want.root_nodes);
        assert_eq!(got.objects.len(), want.objects.len());
        for (got_object, want_object) in got.objects.iter().zip(&want.objects) {
            assert_eq!(got_object.name, want_object.name);
            assert_eq!(got_object.layers, want_object.layers);
            assert_eq!(got_object.bounds, want_object.bounds);
            assert_eq!(
                voxel_set(got_object, &got.palettes),
                voxel_set(want_object, &want.palettes)
            );
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_voxj_file(&file).unwrap();
        assert_file_eq(&to_voxj_file(&state).unwrap(), &file);
    }

    /// A one-object document whose edit grid is larger than its runtime grid, so
    /// the object carries margin.
    fn margin_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![numbered_value_pool(2)],
                    objects: vec![object(
                        "o",
                        vec![0],
                        [2, 1, 1],
                        vec![[0, 0, 0], [1, 0, 0]],
                        vec![vec![0], vec![1]],
                    )],
                    palettes: vec![numbered_palette("baseColor", 0, 2)],
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: Some(VoxjEditState {
                    objects: vec![VoxjEditObject {
                        bounds: [4, 2, 2],
                        origin: [-1, 0, 0],
                    }],
                }),
                ext: None,
            },
        }
    }

    /// A document whose edit grid differs from the runtime grid (has margin)
    /// round-trips the edit state through the VoxMain and back.
    #[test]
    fn round_trips_edit_state() {
        let file = margin_file();
        let state = from_voxj_file(&file).unwrap();
        assert_file_eq(&to_voxj_file(&state).unwrap(), &file);
    }

    /// The builder's defaults reproduce the document [`to_voxj_file`] writes.
    #[test]
    fn builder_default_matches_to_voxj_file() {
        let state = from_voxj_file(&sample_file()).unwrap();
        assert_file_eq(
            &VoxjFileBuilder::new(&state).build().unwrap(),
            &to_voxj_file(&state).unwrap(),
        );
    }

    /// `EditStateMode::Always` records the edit state even when every object is
    /// already tight, one entry per object holding its build volume.
    #[test]
    fn always_records_edit_state_when_tight() {
        let state = from_voxj_file(&sample_file()).unwrap();
        // Auto omits it: every object in the fixture is already tight.
        assert_eq!(to_voxj_file(&state).unwrap().main.edit_state, None);

        let file = VoxjFileBuilder::new(&state)
            .edit_state(EditStateMode::Always)
            .build()
            .unwrap();
        assert_eq!(
            file.main.edit_state,
            Some(VoxjEditState {
                objects: vec![
                    VoxjEditObject {
                        bounds: [4, 4, 4],
                        origin: [0, 0, 0],
                    },
                    VoxjEditObject {
                        bounds: [2, 1, 1],
                        origin: [0, 0, 0],
                    },
                    VoxjEditObject {
                        bounds: [3, 1, 2],
                        origin: [0, 0, 0],
                    },
                ],
            })
        );
    }

    /// `EditStateMode::Never` drops the edit state even when an object carries
    /// margin, so its build volume is not recorded.
    #[test]
    fn never_discards_edit_state_with_margin() {
        let file = margin_file();
        let state = from_voxj_file(&file).unwrap();
        // Auto would emit it, since the object carries margin.
        assert!(to_voxj_file(&state).unwrap().main.edit_state.is_some());

        let got = VoxjFileBuilder::new(&state)
            .edit_state(EditStateMode::Never)
            .build()
            .unwrap();
        assert_eq!(got.main.edit_state, None);
    }

    /// `ext(false)` drops the user-defined ext block; the default keeps it.
    #[test]
    fn ext_false_drops_the_ext_block() {
        let state = from_voxj_file(&sample_file()).unwrap();
        assert!(
            VoxjFileBuilder::new(&state)
                .build()
                .unwrap()
                .main
                .ext
                .is_some()
        );
        let dropped = VoxjFileBuilder::new(&state).ext(false).build().unwrap();
        assert_eq!(dropped.main.ext, None);
    }

    #[test]
    fn round_trips_through_voxj_bytes() {
        let file = sample_file();
        let state = from_voxj_file(&file).unwrap();
        let bytes = to_voxj_bytes(&state).unwrap();
        let reloaded = from_voxj_bytes(&bytes).unwrap();
        assert_file_eq(&to_voxj_file(&reloaded).unwrap(), &file);
    }

    #[test]
    fn round_trips_through_voxjz_bytes() {
        let file = sample_file();
        let state = from_voxj_file(&file).unwrap();
        let bytes = to_voxjz_bytes(&state).unwrap();
        let reloaded = from_voxj_bytes(&bytes).unwrap();
        assert_file_eq(&to_voxj_file(&reloaded).unwrap(), &file);
    }

    #[test]
    fn gc_on_a_loaded_state_preserves_the_round_trip() {
        let file = sample_file();
        let mut state = from_voxj_file(&file).unwrap();
        // A freshly loaded state is already contiguous, so gc leaves the saved
        // document unchanged.
        state.gc();
        assert_file_eq(&to_voxj_file(&state).unwrap(), &file);
    }

    #[test]
    fn remove_then_gc_round_trips_through_bytes() {
        let mut state = from_voxj_file(&sample_file()).unwrap();

        // Remove the "tight" object and the palette only it referenced, then
        // compact so the save numbers entities by listing index again.
        assert_eq!(
            state.release_object(U32Id::<BVoxObject>::from_u32(1)),
            Ok(())
        );
        assert_eq!(
            state.release_palette(U32Id::<BVoxPalette>::from_u32(1)),
            Ok(())
        );
        state.gc();

        let bytes = to_voxj_bytes(&state).unwrap();
        let reloaded = from_voxj_bytes(&bytes).unwrap();
        assert_file_eq(
            &to_voxj_file(&reloaded).unwrap(),
            &sample_file_without_tight(),
        );
    }

    /// Removing the first layer keeps the surviving layers' relative order
    /// through a gc and a save: the document still lists the second and third
    /// layers' palettes and samples in their original order. Removing the first
    /// of three is the smallest case a swap-remove would get wrong, listing the
    /// third layer before the second.
    #[test]
    fn remove_first_layer_then_gc_keeps_layer_order() {
        let palettes = vec![
            numbered_palette("baseColor", 0, 2),
            numbered_palette("baseColor", 0, 2),
            numbered_palette("baseColor", 0, 2),
        ];
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![numbered_value_pool(2)],
                    objects: vec![object(
                        "o",
                        vec![0, 1, 2],
                        [2, 1, 1],
                        vec![[0, 0, 0], [1, 0, 0]],
                        vec![vec![0, 1, 0], vec![1, 0, 1]],
                    )],
                    palettes: palettes.clone(),
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        let mut state = from_voxj_file(&file).unwrap();

        // Layer ids follow the listing on load, so the first layer is id 0.
        let object_id = U32Id::<BVoxObject>::from_u32(0);
        assert_eq!(
            state.release_layer(object_id, U32Id::<BVoxLayer>::from_u32(0)),
            Ok(())
        );
        state.gc();
        state.validate().unwrap();

        let bytes = to_voxj_bytes(&state).unwrap();
        let reloaded = from_voxj_bytes(&bytes).unwrap();
        let expected = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![numbered_value_pool(2)],
                    objects: vec![object(
                        "o",
                        vec![1, 2],
                        [2, 1, 1],
                        vec![[0, 0, 0], [1, 0, 0]],
                        vec![vec![1, 0], vec![0, 1]],
                    )],
                    palettes,
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        assert_file_eq(&to_voxj_file(&reloaded).unwrap(), &expected);
    }

    #[test]
    fn rejects_position_outside_bounds() {
        let mut file = sample_file();
        file.main.runtime_state.objects[1].voxel_positions =
            VoxjPositionBlock::RawJson(vec![[9, 0, 0], [1, 0, 0]]);
        assert!(from_voxj_file(&file).is_err());
    }

    #[test]
    fn rejects_out_of_range_sample() {
        let mut file = sample_file();
        // One channel (object 0 has one layer); the first voxel samples
        // material 99, out of range for palette 0's six materials.
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![99, 0, 5, 2]]);
        assert!(from_voxj_file(&file).is_err());
    }

    /// A voxelless object (tight bounds `[0, 0, 0]`) still carries one empty
    /// channel per layer and round-trips rather than being rejected.
    #[test]
    fn round_trips_a_voxelless_object_with_a_layer() {
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![numbered_value_pool(1)],
                    objects: vec![VoxjObject {
                        name: "empty-ref".to_owned(),
                        layers: vec![0],
                        bounds: [0, 0, 0],
                        origin: [0, 0, 0],
                        voxel_positions: VoxjPositionBlock::RawJson(Vec::new()),
                        voxel_samples: VoxjSampleBlock::RawJson(vec![Vec::new()]),
                    }],
                    palettes: vec![numbered_palette("baseColor", 0, 1)],
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        let state = from_voxj_file(&file).unwrap();
        assert_file_eq(&to_voxj_file(&state).unwrap(), &file);
    }

    /// Sibling variant palettes round-trip: both share the same value pools and
    /// differ in one column, and the object layered over them carries one
    /// channel per layer.
    #[test]
    fn round_trips_sibling_variant_palettes() {
        let variant = |strength: usize| VoxjPalette {
            properties: vec![property("baseColor", 0), property("emissiveStrength", 1)],
            materials: vec![vec![0, strength], vec![1, strength]],
        };
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![numbered_value_pool(2), numbered_value_pool(3)],
                    objects: vec![object(
                        "lamp",
                        vec![0, 1],
                        [2, 1, 1],
                        vec![[0, 0, 0], [1, 0, 0]],
                        vec![vec![0, 0], vec![1, 1]],
                    )],
                    palettes: vec![variant(0), variant(2)],
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        let state = from_voxj_file(&file).unwrap();
        assert_file_eq(&to_voxj_file(&state).unwrap(), &file);
    }

    /// Every vector kind maps one to one through the vox state: the values
    /// come back bit-identical, colors and plain vectors alike.
    #[test]
    fn round_trips_vector_value_pools() {
        let file = vector_kind_file();
        let state = from_voxj_file(&file).unwrap();
        let written = to_voxj_file(&state).unwrap();
        assert_eq!(
            written.main.runtime_state.value_pools,
            file.main.runtime_state.value_pools
        );
    }

    /// A one-voxel object over a palette binding one property per vector kind.
    fn vector_kind_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![
                        VoxjValuePool::Vec2Float(vec![[0.25, 0.75]]),
                        VoxjValuePool::Vec2Int(vec![[3, 7]]),
                        VoxjValuePool::Vec3Float(vec![[2.5, 0.0, 1.0]]),
                        VoxjValuePool::Vec3Int(vec![[-1, 0, 1]]),
                        VoxjValuePool::Vec4Float(vec![
                            [1.0, 0.0, 0.0, 1.0],
                            [0.0, 1.0, 0.0, 128.0 / 255.0],
                        ]),
                        VoxjValuePool::Vec4Int(vec![[0, 1, 2, 3]]),
                    ],
                    objects: vec![object(
                        "o",
                        vec![0],
                        [1, 1, 1],
                        vec![[0, 0, 0]],
                        vec![vec![0]],
                    )],
                    palettes: vec![VoxjPalette {
                        properties: vec![
                            property("customUv", 0),
                            property("customCell", 1),
                            property("emissiveColor", 2),
                            property("customAxis", 3),
                            property("baseColor", 4),
                            property("customTint", 5),
                        ],
                        materials: vec![vec![0, 0, 0, 0, 0, 0], vec![0, 0, 0, 0, 1, 0]],
                    }],
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        }
    }

    /// A sparse object with huge bounds is rejected at the volume check, before
    /// it can force a dense multi-gigabyte allocation.
    #[test]
    fn rejects_oversized_dense_grid() {
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: Vec::new(),
                    objects: vec![object(
                        "huge",
                        Vec::new(),
                        [1024, 1024, 1024],
                        Vec::new(),
                        Vec::new(),
                    )],
                    palettes: Vec::new(),
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        assert!(from_voxj_file(&file).is_err());
    }
}
