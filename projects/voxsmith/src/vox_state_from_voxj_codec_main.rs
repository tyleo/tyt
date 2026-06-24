use crate::{
    vox_hierarchy_node_from_voxj_hierarchy_node, vox_object_from_voxj_codec_object,
    vox_palette_from_voxj_palette, vox_value_from_voxj_value,
};
use branded_id::U32Id;
use std::io;
use voxcore::VoxState;
use voxj::VoxjCodecMain;

/// Loads a [`VoxjCodecMain`] into a [`VoxState`]. Entities take ids in listing
/// order, so each id equals its voxj array index and cross-references carry over.
///
/// Errors on malformed object geometry (oversized grid, position out of bounds,
/// ragged rows) or if [`VoxState::validate`](voxcore::VoxState::validate) rejects
/// the assembled state.
pub fn vox_state_from_voxj_codec_main(main: &VoxjCodecMain) -> io::Result<VoxState> {
    let mut state = VoxState::default();

    // Build each value before adding it so a failed conversion leaves the state
    // untouched.
    for palette in &main.palettes {
        state.add_palette(vox_palette_from_voxj_palette(palette)?);
    }

    for object in &main.objects {
        state.add_object(vox_object_from_voxj_codec_object(object)?);
    }

    for node in &main.hierarchy_nodes {
        state.add_hierarchy_node(vox_hierarchy_node_from_voxj_hierarchy_node(node)?);
    }

    state.set_root_hierarchy_nodes(
        main.root_hierarchy_nodes
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
    state.validate().map_err(io::Error::other)?;

    Ok(state)
}

#[cfg(test)]
mod tests {
    use crate::{
        vox_state_from_voxj_bytes, vox_state_from_voxj_codec_main, vox_state_to_voxj_bytes,
        vox_state_to_voxjz_bytes, voxj_codec_main_from_vox_state,
    };
    use std::{collections::BTreeSet, f64::consts::FRAC_1_SQRT_2};
    use voxj::{
        VoxjCodecMain, VoxjCodecObject, VoxjHierarchyNode, VoxjMap, VoxjPalette, VoxjTransform,
        VoxjValue,
    };

    /// `n` single-attribute cells whose value is the cell index.
    fn numbered_cells(n: usize) -> Vec<Vec<VoxjValue>> {
        (0..n).map(|i| vec![VoxjValue::Number(i as f64)]).collect()
    }

    /// A document exercising every field: a sparse object with margin bounds, a
    /// tight object sampling two palettes, multi-attribute palette cells, a
    /// hierarchy with a non-identity transform, roots, and a nested `ext`.
    fn sample_main() -> VoxjCodecMain {
        VoxjCodecMain {
            objects: vec![
                VoxjCodecObject {
                    name: "sparse".to_owned(),
                    palette_refs: vec![0],
                    bounds: [4, 4, 4],
                    positions: vec![[0, 0, 0], [3, 1, 2], [1, 3, 0], [2, 2, 3]],
                    samples: vec![vec![1], vec![0], vec![5], vec![2]],
                },
                VoxjCodecObject {
                    name: "tight".to_owned(),
                    palette_refs: vec![0, 1],
                    bounds: [2, 1, 1],
                    positions: vec![[0, 0, 0], [1, 0, 0]],
                    samples: vec![vec![3, 0], vec![1, 1]],
                },
                VoxjCodecObject {
                    name: "no-palette".to_owned(),
                    palette_refs: Vec::new(),
                    bounds: [3, 1, 2],
                    positions: vec![[0, 0, 0], [2, 0, 1]],
                    samples: vec![Vec::new(), Vec::new()],
                },
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
            ext: Some(VoxjValue::Object(VoxjMap(vec![(
                "vendor".to_owned(),
                VoxjValue::Array(vec![
                    VoxjValue::Number(1.0),
                    VoxjValue::Bool(true),
                    VoxjValue::Null,
                    VoxjValue::Text("x".to_owned()),
                ]),
            )]))),
        }
    }

    /// The `(position, samples)` pairs of an object, order-independent, since the
    /// dense grid re-emits voxels in raster order rather than listing order.
    fn voxel_set(object: &VoxjCodecObject) -> BTreeSet<([u32; 3], Vec<u32>)> {
        object
            .positions
            .iter()
            .copied()
            .zip(object.samples.iter().cloned())
            .collect()
    }

    fn assert_main_eq(got: &VoxjCodecMain, want: &VoxjCodecMain) {
        assert_eq!(got.palettes, want.palettes);
        assert_eq!(got.hierarchy_nodes, want.hierarchy_nodes);
        assert_eq!(got.root_hierarchy_nodes, want.root_hierarchy_nodes);
        assert_eq!(got.ext, want.ext);
        assert_eq!(got.objects.len(), want.objects.len());
        for (got, want) in got.objects.iter().zip(&want.objects) {
            assert_eq!(got.name, want.name);
            assert_eq!(got.palette_refs, want.palette_refs);
            assert_eq!(got.bounds, want.bounds);
            assert_eq!(voxel_set(got), voxel_set(want));
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let main = sample_main();
        let state = vox_state_from_voxj_codec_main(&main).unwrap();
        assert_main_eq(&voxj_codec_main_from_vox_state(&state), &main);
    }

    #[test]
    fn round_trips_through_voxj_bytes() {
        let main = sample_main();
        let state = vox_state_from_voxj_codec_main(&main).unwrap();
        let bytes = vox_state_to_voxj_bytes(&state).unwrap();
        let reloaded = vox_state_from_voxj_bytes(&bytes).unwrap();
        assert_main_eq(&voxj_codec_main_from_vox_state(&reloaded), &main);
    }

    #[test]
    fn round_trips_through_voxjz_bytes() {
        let main = sample_main();
        let state = vox_state_from_voxj_codec_main(&main).unwrap();
        let bytes = vox_state_to_voxjz_bytes(&state).unwrap();
        let reloaded = vox_state_from_voxj_bytes(&bytes).unwrap();
        assert_main_eq(&voxj_codec_main_from_vox_state(&reloaded), &main);
    }

    #[test]
    fn rejects_position_outside_bounds() {
        let mut main = sample_main();
        main.objects[1].positions[0] = [9, 0, 0];
        assert!(vox_state_from_voxj_codec_main(&main).is_err());
    }

    #[test]
    fn rejects_out_of_range_sample() {
        let mut main = sample_main();
        main.objects[0].samples[0] = vec![99];
        assert!(vox_state_from_voxj_codec_main(&main).is_err());
    }

    /// An object with bounds but no voxels may reference an empty palette: no
    /// voxel samples it, so it round-trips rather than being rejected.
    #[test]
    fn round_trips_empty_palette_referenced_by_voxelless_object() {
        let main = VoxjCodecMain {
            objects: vec![VoxjCodecObject {
                name: "empty-ref".to_owned(),
                palette_refs: vec![0],
                bounds: [2, 2, 2],
                positions: Vec::new(),
                samples: Vec::new(),
            }],
            palettes: vec![VoxjPalette {
                attributes: vec!["rgba".to_owned()],
                data: Vec::new(),
            }],
            hierarchy_nodes: Vec::new(),
            root_hierarchy_nodes: Vec::new(),
            ext: None,
        };
        let state = vox_state_from_voxj_codec_main(&main).unwrap();
        assert_main_eq(&voxj_codec_main_from_vox_state(&state), &main);
    }

    /// A sparse object with huge bounds is rejected at the volume check, before
    /// it can force a dense multi-gigabyte allocation.
    #[test]
    fn rejects_oversized_dense_grid() {
        let main = VoxjCodecMain {
            objects: vec![VoxjCodecObject {
                name: "huge".to_owned(),
                palette_refs: Vec::new(),
                bounds: [1024, 1024, 1024],
                positions: Vec::new(),
                samples: Vec::new(),
            }],
            palettes: Vec::new(),
            hierarchy_nodes: Vec::new(),
            root_hierarchy_nodes: Vec::new(),
            ext: None,
        };
        assert!(vox_state_from_voxj_codec_main(&main).is_err());
    }
}
