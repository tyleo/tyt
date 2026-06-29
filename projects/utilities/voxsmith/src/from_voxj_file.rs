use crate::{
    Result, vox_hierarchy_node_from_voxj_hierarchy_node, vox_object_from_voxj_decoded_object,
    vox_palette_from_voxj_palette, vox_value_from_voxj_value,
};
use branded_id::U32Id;
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{VoxEditObject, VoxState};
use voxj::VoxjFile;
use voxj_codec::{decode_voxj_object, voxj_palette_cell_counts};

/// Loads a [`VoxjFile`] into a [`VoxState`]. Each object's position and sample
/// blocks are decoded, then entities take ids in listing order, so each id
/// equals its voxj array index and cross-references carry over.
///
/// Errors on a malformed block, malformed object geometry, or if
/// [`VoxState::validate`](voxcore::VoxState::validate) rejects the assembled
/// state.
pub fn from_voxj_file(file: &VoxjFile) -> Result<VoxState> {
    let main = &file.main;
    let mut state = VoxState::default();

    // Build each value before adding it so a failed conversion leaves the state
    // untouched.
    for palette in &main.runtime_state.palettes {
        state.add_palette(vox_palette_from_voxj_palette(palette)?);
    }

    for (index, object) in main.runtime_state.objects.iter().enumerate() {
        let cell_counts =
            voxj_palette_cell_counts(&object.palette_refs, &main.runtime_state.palettes)?;
        let decoded = decode_voxj_object(object, &cell_counts)?;
        let vox_object = vox_object_from_voxj_decoded_object(&decoded)?;
        let id = state.add_object(vox_object);
        if let Some(edit) = main.edit_state.as_ref().and_then(|e| e.objects.get(index)) {
            state.set_edit_object(
                id,
                VoxEditObject {
                    bounds: TyVector3U32::new(edit.bounds[0], edit.bounds[1], edit.bounds[2]),
                    origin: TyVector3I32::new(edit.origin[0], edit.origin[1], edit.origin[2]),
                },
            );
        }
    }

    for node in &main.runtime_state.hierarchy_nodes {
        state.add_hierarchy_node(vox_hierarchy_node_from_voxj_hierarchy_node(node)?);
    }

    state.set_root_hierarchy_nodes(
        main.runtime_state
            .root_hierarchy_nodes
            .iter()
            .map(|&index| U32Id::from_u32(index as u32))
            .collect(),
    );

    state.set_ext(
        main.ext
            .as_ref()
            .map(vox_value_from_voxj_value)
            .transpose()?,
    );

    // Check cross-references and acyclicity on the assembled state.
    state.validate()?;

    Ok(state)
}

#[cfg(test)]
mod tests {
    use crate::{from_voxj_bytes, from_voxj_file, to_voxj_bytes, to_voxj_file, to_voxjz_bytes};
    use branded_id::U32Id;
    use std::{collections::BTreeSet, f64::consts::FRAC_1_SQRT_2};
    use voxcore::{BVoxObject, BVoxPalette};
    use voxj::{
        VoxjEditObject, VoxjEditState, VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjMap, VoxjObject,
        VoxjPalette, VoxjPositionBlock, VoxjRuntimeState, VoxjSampleBlock, VoxjTransform,
        VoxjValue,
    };
    use voxj_codec::{decode_voxj_object, voxj_palette_cell_counts};

    /// `n` single-attribute cells whose value is the cell index.
    fn numbered_cells(n: usize) -> Vec<Vec<VoxjValue>> {
        (0..n).map(|i| vec![VoxjValue::Number(i as f64)]).collect()
    }

    /// One object holding raw-json position and sample blocks, the readable form
    /// the fixtures author geometry in.
    fn object(
        name: &str,
        palette_refs: Vec<usize>,
        bounds: [u32; 3],
        positions: Vec<[u32; 3]>,
        samples: Vec<Vec<u32>>,
    ) -> VoxjObject {
        VoxjObject {
            name: name.to_owned(),
            palette_refs,
            bounds,
            origin: [0, 0, 0],
            voxel_positions: VoxjPositionBlock::RawJson(positions),
            voxel_samples: VoxjSampleBlock::RawJson(samples),
        }
    }

    /// A document exercising every field: a sparse object with margin bounds, a
    /// tight object sampling two palettes, multi-attribute palette cells, a
    /// hierarchy with a non-identity transform, roots, and a nested `ext`.
    fn sample_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    objects: vec![
                        object(
                            "sparse",
                            vec![0],
                            [4, 4, 4],
                            vec![[0, 0, 0], [3, 1, 2], [1, 3, 0], [2, 2, 3]],
                            vec![vec![1], vec![0], vec![5], vec![2]],
                        ),
                        object(
                            "tight",
                            vec![0, 1],
                            [2, 1, 1],
                            vec![[0, 0, 0], [1, 0, 0]],
                            vec![vec![3, 0], vec![1, 1]],
                        ),
                        object(
                            "no-palette",
                            Vec::new(),
                            [3, 1, 2],
                            vec![[0, 0, 0], [2, 0, 1]],
                            vec![Vec::new(), Vec::new()],
                        ),
                    ],
                    palettes: vec![
                        VoxjPalette {
                            attributes: vec!["rgba".to_owned()],
                            data: numbered_cells(6),
                        },
                        VoxjPalette {
                            attributes: vec!["metallic".to_owned(), "ior".to_owned()],
                            data: vec![
                                vec![VoxjValue::Bool(false), VoxjValue::Number(1.5)],
                                vec![VoxjValue::Bool(true), VoxjValue::Number(2.0)],
                            ],
                        },
                    ],
                    hierarchy_nodes: vec![
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
                    root_hierarchy_nodes: vec![0],
                },
                edit_state: None,
                ext: Some(VoxjValue::Object(VoxjMap(vec![(
                    "vendor".to_owned(),
                    VoxjValue::Array(vec![
                        VoxjValue::Number(1.0),
                        VoxjValue::Bool(true),
                        VoxjValue::Null,
                        VoxjValue::Text("x".to_owned()),
                    ]),
                )]))),
            },
        }
    }

    /// [`sample_file`] after removing the "tight" object (id 1) and the palette
    /// only it referenced (id 1), then compacting: the two survivors renumber to
    /// objects 0 and 1, the lone palette stays 0, and the "leaf" node loses its
    /// reference to the removed object.
    fn sample_file_without_tight() -> VoxjFile {
        let base = sample_file();
        VoxjFile {
            version: base.version,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    objects: vec![
                        base.main.runtime_state.objects[0].clone(),
                        base.main.runtime_state.objects[2].clone(),
                    ],
                    palettes: vec![base.main.runtime_state.palettes[0].clone()],
                    hierarchy_nodes: vec![
                        base.main.runtime_state.hierarchy_nodes[0].clone(),
                        VoxjHierarchyNode {
                            child_objects: Vec::new(),
                            ..base.main.runtime_state.hierarchy_nodes[1].clone()
                        },
                    ],
                    root_hierarchy_nodes: vec![0],
                },
                edit_state: None,
                ext: base.main.ext.clone(),
            },
        }
    }

    /// The `(position, samples)` pairs of an object, order-independent, since the
    /// dense grid re-emits voxels in raster order and the block encodings reorder
    /// them again. The object's blocks are decoded against the document palettes.
    fn voxel_set(object: &VoxjObject, palettes: &[VoxjPalette]) -> BTreeSet<([u32; 3], Vec<u32>)> {
        let cell_counts = voxj_palette_cell_counts(&object.palette_refs, palettes).unwrap();
        let decoded = decode_voxj_object(object, &cell_counts).unwrap();
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
        assert_eq!(got.palettes, want.palettes);
        assert_eq!(got.hierarchy_nodes, want.hierarchy_nodes);
        assert_eq!(got.root_hierarchy_nodes, want.root_hierarchy_nodes);
        assert_eq!(got.objects.len(), want.objects.len());
        for (got_object, want_object) in got.objects.iter().zip(&want.objects) {
            assert_eq!(got_object.name, want_object.name);
            assert_eq!(got_object.palette_refs, want_object.palette_refs);
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

    /// A document whose edit grid differs from the runtime grid (carries margin)
    /// round-trips the edit state through the VoxState and back.
    #[test]
    fn round_trips_edit_state() {
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    objects: vec![object(
                        "o",
                        vec![0],
                        [2, 1, 1],
                        vec![[0, 0, 0], [1, 0, 0]],
                        vec![vec![0], vec![1]],
                    )],
                    palettes: vec![VoxjPalette {
                        attributes: vec!["rgba".to_owned()],
                        data: numbered_cells(2),
                    }],
                    hierarchy_nodes: Vec::new(),
                    root_hierarchy_nodes: Vec::new(),
                },
                edit_state: Some(VoxjEditState {
                    objects: vec![VoxjEditObject {
                        bounds: [4, 2, 2],
                        origin: [-1, 0, 0],
                    }],
                }),
                ext: None,
            },
        };
        let state = from_voxj_file(&file).unwrap();
        assert_file_eq(&to_voxj_file(&state).unwrap(), &file);
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
            state.remove_object(U32Id::<BVoxObject>::from_u32(1)),
            Some(())
        );
        assert_eq!(
            state.remove_palette(U32Id::<BVoxPalette>::from_u32(1)),
            Some(())
        );
        state.gc();

        let bytes = to_voxj_bytes(&state).unwrap();
        let reloaded = from_voxj_bytes(&bytes).unwrap();
        assert_file_eq(
            &to_voxj_file(&reloaded).unwrap(),
            &sample_file_without_tight(),
        );
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
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![99], vec![0], vec![5], vec![2]]);
        assert!(from_voxj_file(&file).is_err());
    }

    /// A voxelless object (tight bounds `[0, 0, 0]`) may reference an empty
    /// palette: no voxel samples it, so it round-trips rather than being rejected.
    #[test]
    fn round_trips_empty_palette_referenced_by_voxelless_object() {
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    objects: vec![object(
                        "empty-ref",
                        vec![0],
                        [0, 0, 0],
                        Vec::new(),
                        Vec::new(),
                    )],
                    palettes: vec![VoxjPalette {
                        attributes: vec!["rgba".to_owned()],
                        data: Vec::new(),
                    }],
                    hierarchy_nodes: Vec::new(),
                    root_hierarchy_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        let state = from_voxj_file(&file).unwrap();
        assert_file_eq(&to_voxj_file(&state).unwrap(), &file);
    }

    /// A sparse object with huge bounds is rejected at the volume check, before
    /// it can force a dense multi-gigabyte allocation.
    #[test]
    fn rejects_oversized_dense_grid() {
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    objects: vec![object(
                        "huge",
                        Vec::new(),
                        [1024, 1024, 1024],
                        Vec::new(),
                        Vec::new(),
                    )],
                    palettes: Vec::new(),
                    hierarchy_nodes: Vec::new(),
                    root_hierarchy_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        assert!(from_voxj_file(&file).is_err());
    }
}
