use crate::{GoxlExtSource, Result, ext::GoxlVoxMain};
use goxl::GoxlFile;

/// Writes a [`GoxlVoxMain`] back to a Goxel [`GoxlFile`], the inverse of
/// [`from_goxl_file_with_ext`](crate::ext::from_goxl_file_with_ext) and the
/// typed form of [`to_goxl_file`](crate::to_goxl_file). A loaded file writes
/// back exactly through its ext. Errors if the ext is out of step with the
/// hierarchy.
pub fn to_goxl_file_with_ext(state: &GoxlVoxMain) -> Result<GoxlFile> {
    GoxlExtSource::write_goxl(state)
}

#[cfg(test)]
mod tests {
    use crate::ext::{GoxlVoxMain, from_goxl_file_with_ext, to_goxl_file_with_ext};
    use branded_id::U32Id;
    use goxl::{
        GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
        GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
    };
    use ty_math::{TyTransformF64, TyVector3F64};
    use voxcore::{BVoxHierarchyNode, BVoxObject, VoxHierarchyNode};

    /// A `4 x 4` matrix with distinct float cells, for transform and box
    /// fields.
    fn matrix(base: f32) -> [[f32; 4]; 4] {
        let mut matrix = [[0.0f32; 4]; 4];
        for (index, cell) in matrix.iter_mut().flatten().enumerate() {
            *cell = base + index as f32 * 0.5;
        }
        matrix
    }

    /// A full `16 x 16 x 16` block: mostly empty (all-zero cells), with a few
    /// tagged voxels including one with a partial alpha.
    fn block() -> GoxlBlock {
        let mut voxels = vec![GoxlVoxel::default(); GoxlBlock::SIZE.pow(3) as usize];
        voxels[0] = GoxlVoxel::new(10, 20, 30);
        voxels[1] = GoxlVoxel::new(255, 0, 0);
        voxels[100] = GoxlVoxel {
            r: 1,
            g: 2,
            b: 3,
            a: 128,
        };
        *voxels.last_mut().unwrap() = GoxlVoxel::new(7, 8, 9);
        GoxlBlock { voxels }
    }

    /// A pair whose value is raw bytes, for `extra` dictionaries.
    fn extra(key: &str, value: &[u8]) -> (String, Vec<u8>) {
        (key.to_owned(), value.to_vec())
    }

    /// A file exercising every modeled chunk: two shared blocks, a normal layer
    /// stamping both, a clone layer, a shape layer, materials, cameras, a
    /// light, a preview, an image box, and an unknown chunk.
    fn sample_file() -> GoxlFile {
        GoxlFile {
            version: 2,
            image: GoxlImage {
                bounding_box: Some(matrix(1.0)),
                extra: GoxlDict(vec![extra("vendor", &[1, 2, 3, 4])]),
            },
            preview: Some(GoxlPreview {
                width: 2,
                height: 3,
                pixels: vec![
                    [0, 0, 0, 0],
                    [255, 255, 255, 255],
                    [1, 2, 3, 4],
                    [5, 6, 7, 8],
                    [9, 10, 11, 12],
                    [13, 14, 15, 16],
                ],
            }),
            blocks: vec![
                block(),
                GoxlBlock {
                    voxels: vec![GoxlVoxel::new(50, 60, 70); GoxlBlock::SIZE.pow(3) as usize],
                },
            ],
            materials: vec![
                GoxlMaterial {
                    name: "Default".to_owned(),
                    base_color: [0.25, 0.5, 0.75, 1.0],
                    metallic: 0.1,
                    roughness: 0.9,
                    emission: [0.0, 0.5, 1.0],
                    extra: GoxlDict(vec![extra("_custom", &[9])]),
                },
                GoxlMaterial::default(),
            ],
            layers: vec![
                GoxlLayer {
                    name: "Layer 0".to_owned(),
                    id: 1,
                    base_id: 0,
                    material: 0,
                    mode: 0,
                    visible: true,
                    transform: matrix(2.0),
                    blocks: vec![
                        GoxlLayerBlock {
                            block_index: 0,
                            position: [0, 0, 0],
                        },
                        GoxlLayerBlock {
                            block_index: 1,
                            position: [-16, 16, -32],
                        },
                    ],
                    bounding_box: Some(matrix(3.0)),
                    image_path: Some("/tmp/ref.png".to_owned()),
                    shape: None,
                    color: None,
                    extra: GoxlDict(vec![extra("_layer", &[7, 7])]),
                },
                GoxlLayer {
                    name: "Clone".to_owned(),
                    id: 2,
                    base_id: 1,
                    material: 1,
                    mode: 1,
                    visible: false,
                    transform: matrix(4.0),
                    blocks: Vec::new(),
                    bounding_box: None,
                    image_path: None,
                    shape: None,
                    color: None,
                    extra: GoxlDict::default(),
                },
                GoxlLayer {
                    name: "Shape".to_owned(),
                    id: 3,
                    base_id: 0,
                    material: 0,
                    mode: 0,
                    visible: true,
                    transform: matrix(5.0),
                    blocks: Vec::new(),
                    bounding_box: None,
                    image_path: None,
                    shape: Some(GoxlShape::Cylinder),
                    color: Some([200, 150, 100, 255]),
                    extra: GoxlDict::default(),
                },
            ],
            cameras: vec![
                GoxlCamera {
                    name: "Camera".to_owned(),
                    distance: 12.5,
                    orthographic: true,
                    transform: matrix(6.0),
                    active: true,
                    extra: GoxlDict(vec![extra("_cam", &[3, 3, 3])]),
                },
                GoxlCamera::default(),
            ],
            light: Some(GoxlLight {
                pitch: 0.5,
                yaw: -0.25,
                intensity: 1.5,
                fixed: true,
                ambient: 0.2,
                shadow: 0.8,
                extra: GoxlDict(vec![extra("_light", &[1])]),
            }),
            unknown_chunks: vec![GoxlUnknownChunk {
                id: *b"XTRA",
                data: vec![9, 8, 7, 6, 5],
            }],
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_goxl_file_with_ext(&file).unwrap();
        assert_eq!(to_goxl_file_with_ext(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = GoxlFile::default();
        let state = from_goxl_file_with_ext(&file).unwrap();
        assert_eq!(to_goxl_file_with_ext(&state).unwrap(), file);
    }

    /// A layer may stamp the same block at several positions. The voxcore node
    /// lists the placed object once, but the ext keeps the full placement list,
    /// so the layer round-trips.
    #[test]
    fn round_trips_a_layer_stamping_one_block_twice() {
        let file = GoxlFile {
            blocks: vec![block()],
            layers: vec![GoxlLayer {
                name: "L".to_owned(),
                blocks: vec![
                    GoxlLayerBlock {
                        block_index: 0,
                        position: [0, 0, 0],
                    },
                    GoxlLayerBlock {
                        block_index: 0,
                        position: [16, 0, 0],
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        };
        let state = from_goxl_file_with_ext(&file).unwrap();
        assert_eq!(to_goxl_file_with_ext(&state).unwrap(), file);
    }

    /// A block of one color.
    fn solid_block(r: u8, g: u8, b: u8) -> GoxlBlock {
        GoxlBlock {
            voxels: vec![GoxlVoxel::new(r, g, b); GoxlBlock::SIZE.pow(3) as usize],
        }
    }

    /// A layer stamping `blocks` at their positions.
    fn stamping_layer(name: &str, id: i32, blocks: &[(i32, [i32; 3])]) -> GoxlLayer {
        GoxlLayer {
            name: name.to_owned(),
            id,
            blocks: blocks
                .iter()
                .map(|&(block_index, position)| GoxlLayerBlock {
                    block_index,
                    position,
                })
                .collect(),
            ..Default::default()
        }
    }

    /// Three blocks under three layers. Blocks 0 and 2 are each stamped twice.
    /// A surviving layer then keeps a repeated stamp.
    fn placed_blocks_file() -> GoxlFile {
        GoxlFile {
            blocks: vec![
                solid_block(10, 0, 0),
                solid_block(0, 20, 0),
                solid_block(0, 0, 30),
            ],
            layers: vec![
                stamping_layer(
                    "keep",
                    1,
                    &[(0, [0, 0, 0]), (1, [16, 0, 0]), (0, [32, 0, 0])],
                ),
                stamping_layer("drop", 2, &[(1, [0, 16, 0])]),
                stamping_layer("tail", 3, &[(2, [0, 0, 16]), (2, [0, 0, 32])]),
            ],
            ..Default::default()
        }
    }

    fn set_child_objects(state: &mut GoxlVoxMain, index: u32, child_object_ids: Vec<u32>) {
        let node_id = U32Id::<BVoxHierarchyNode>::from_u32(index);
        let node = VoxHierarchyNode {
            child_object_ids: child_object_ids
                .into_iter()
                .map(U32Id::<BVoxObject>::from_u32)
                .collect(),
            ..state.hierarchy_node(node_id).unwrap().clone()
        };
        state.set_hierarchy_node(node_id, node).unwrap();
    }

    /// Dropping the middle layer and its block, then compacting, leaves the
    /// survivors' provenance aligned. The first layer keeps its two stamps of
    /// block 0. The last layer stamps its block at the new index. The rebuilt
    /// file reloads to the loaded ext minus the released entries.
    #[test]
    fn released_entities_leave_the_survivors_provenance_aligned() {
        let file = placed_blocks_file();
        let mut state = from_goxl_file_with_ext(&file).unwrap();
        let original = state.ext().clone();

        let node = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        state
            .set_root_hierarchy_node_ids(vec![node(0), node(2)])
            .unwrap();
        set_child_objects(&mut state, 0, vec![0]);
        set_child_objects(&mut state, 1, Vec::new());
        state.release_hierarchy_node(node(1)).unwrap();
        state
            .release_object(U32Id::<BVoxObject>::from_u32(1))
            .unwrap();
        state.gc();

        let mut expected = original;
        expected.layers.remove(1);
        let [Some(keep), Some(tail)] = expected.layers.as_mut_slice() else {
            panic!("two loaded layers survive");
        };
        keep.placements = vec![(0, [0, 0, 0]), (0, [32, 0, 0])];
        tail.placements = vec![(1, [0, 0, 16]), (1, [0, 0, 32])];
        assert_eq!(state.ext(), &expected.clone());

        let rebuilt = to_goxl_file_with_ext(&state).unwrap();
        let mut want = file;
        want.blocks.remove(1);
        want.layers.remove(1);
        want.layers[0] = stamping_layer("keep", 1, &[(0, [0, 0, 0]), (0, [32, 0, 0])]);
        want.layers[1] = stamping_layer("tail", 3, &[(1, [0, 0, 16]), (1, [0, 0, 32])]);
        assert_eq!(rebuilt, want);

        let reloaded = from_goxl_file_with_ext(&rebuilt).unwrap();
        assert_eq!(reloaded.ext(), &expected);
    }

    /// A node retained after the load takes no entry. The writer fills it in
    /// like a synthesized layer.
    #[test]
    fn a_node_retained_after_the_load_writes_a_synthesized_layer() {
        let file = placed_blocks_file();
        let mut state = from_goxl_file_with_ext(&file).unwrap();
        let node_id = state
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "added".to_owned(),
                child_object_ids: vec![U32Id::<BVoxObject>::from_u32(2)],
                transform: TyTransformF64 {
                    position: TyVector3F64::new(3.4, -2.6, 16.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
        state.push_root_hierarchy_node_id(node_id).unwrap();
        assert_eq!(state.ext().layers[3], None);

        let rebuilt = to_goxl_file_with_ext(&state).unwrap();
        let mut want = file;
        want.layers
            .push(stamping_layer("added", 4, &[(2, [3, -3, 16])]));
        assert_eq!(rebuilt, want);
    }

    /// Moving the last block to the front renumbers every layer's stamps, so
    /// each layer still stamps the block it did.
    #[test]
    fn a_moved_object_keeps_each_layers_block() {
        let file = placed_blocks_file();
        let mut state = from_goxl_file_with_ext(&file).unwrap();
        state
            .move_object(U32Id::<BVoxObject>::from_u32(2), 0)
            .unwrap();

        let rebuilt = to_goxl_file_with_ext(&state).unwrap();
        let mut want = file;
        want.blocks.rotate_right(1);
        want.layers[0] = stamping_layer(
            "keep",
            1,
            &[(1, [0, 0, 0]), (2, [16, 0, 0]), (1, [32, 0, 0])],
        );
        want.layers[1] = stamping_layer("drop", 2, &[(2, [0, 16, 0])]);
        want.layers[2] = stamping_layer("tail", 3, &[(0, [0, 0, 16]), (0, [0, 0, 32])]);
        assert_eq!(rebuilt, want);
    }

    /// An ext out of step with the hierarchy is malformed. The writer errors
    /// instead of pairing entries by a shifted index, stamping what the state
    /// does not place, or cloning a layer the file lacks.
    #[test]
    fn an_ext_out_of_step_with_its_listings_errors() {
        let file = placed_blocks_file();
        let mut state = from_goxl_file_with_ext(&file).unwrap();
        let ext = state.ext_mut();
        ext.layers.pop();
        assert!(to_goxl_file_with_ext(&state).is_err());

        let mut state = from_goxl_file_with_ext(&file).unwrap();
        let ext = state.ext_mut();
        let Some(drop) = &mut ext.layers[1] else {
            panic!("layer 1 is loaded");
        };
        drop.placements.push((2, [0, 0, 0]));
        assert!(to_goxl_file_with_ext(&state).is_err());

        let mut state = from_goxl_file_with_ext(&file).unwrap();
        let ext = state.ext_mut();
        let Some(drop) = &mut ext.layers[1] else {
            panic!("layer 1 is loaded");
        };
        drop.base_id = 9;
        assert!(to_goxl_file_with_ext(&state).is_err());
    }

    #[cfg(feature = "codec")]
    mod codec {
        use super::*;
        use crate::codec::{from_goxl_bytes_with_ext, to_goxl_bytes_with_ext};
        use goxl_codec::DependenciesImpl;

        #[test]
        fn round_trips_through_goxl_bytes() {
            let file = sample_file();
            let state = from_goxl_file_with_ext(&file).unwrap();
            let bytes = to_goxl_bytes_with_ext(&DependenciesImpl, &state).unwrap();
            let reloaded = from_goxl_bytes_with_ext(&DependenciesImpl, &bytes).unwrap();
            assert_eq!(to_goxl_file_with_ext(&reloaded).unwrap(), file);
        }
    }
}
