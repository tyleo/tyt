use crate::{Error, GoxlExtLayer, GoxlVoxMain, Result};
use branded_id::U32Id;
use goxl::{
    GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
    GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
};
use std::collections::{HashMap, HashSet};
use ty_math::TyVector3U32;
use voxcore::{
    BVoxObject, VoxHierarchyNode, VoxMain, VoxObject, color::resolve_cell_color_or_transparent,
};

/// Writes a [`GoxlVoxMain`] to a Goxel [`GoxlFile`], the inverse of
/// [`from_goxl_file`](crate::from_goxl_file). A loaded file writes back
/// exactly through its ext. A state
/// [`to_goxl_vox_main`](crate::to_goxl_vox_main) gave its ext writes as a
/// file synthesized from the scene. Each object emits one `16 x 16 x 16`
/// block and the rest comes from the ext. A live voxel is written solid.
/// Goxel reads alpha 0 as an empty cell, so a fully transparent color is
/// forced opaque and any other alpha is kept. An empty cell is the
/// transparent zero voxel.
///
/// Errors if:
///
/// 1. an object is larger than a `16 x 16 x 16` block, which
///    `to_goxl_vox_main` retiles for a bare state
/// 2. an object's `baseColor` draws from a non-color value pool
/// 3. the ext's layers are out of step with the hierarchy
pub fn to_goxl_file(state: &GoxlVoxMain) -> Result<GoxlFile> {
    let ext = state.ext();

    let layers = build_layers(state, &ext.layers)?;

    // Each object is the author's build volume, a fixed Goxel 16-cube, so a
    // block is written from it directly at the original positions, its
    // voxel colors read through the object's `baseColor` layer.
    let blocks = state
        .iter_objects()
        .enumerate()
        .map(|(index, (_, object))| block_from_object(state, index, object))
        .collect::<Result<_>>()?;

    Ok(GoxlFile {
        version: ext.version,
        image: GoxlImage {
            bounding_box: ext.image.bounding_box,
            extra: GoxlDict(ext.image.extra.clone()),
        },
        preview: ext.preview.as_ref().map(|preview| GoxlPreview {
            width: preview.width,
            height: preview.height,
            pixels: preview.pixels.clone(),
        }),
        blocks,
        materials: ext
            .materials
            .iter()
            .map(|material| GoxlMaterial {
                name: material.name.clone(),
                base_color: material.base_color,
                metallic: material.metallic,
                roughness: material.roughness,
                emission: material.emission,
                extra: GoxlDict(material.extra.clone()),
            })
            .collect(),
        layers,
        cameras: ext
            .cameras
            .iter()
            .map(|camera| GoxlCamera {
                name: camera.name.clone(),
                distance: camera.distance,
                orthographic: camera.orthographic,
                transform: camera.transform,
                active: camera.active,
                extra: GoxlDict(camera.extra.clone()),
            })
            .collect(),
        light: ext.light.as_ref().map(|light| GoxlLight {
            pitch: light.pitch,
            yaw: light.yaw,
            intensity: light.intensity,
            fixed: light.fixed,
            ambient: light.ambient,
            shadow: light.shadow,
            extra: GoxlDict(light.extra.clone()),
        }),
        unknown_chunks: ext
            .unknown_chunks
            .iter()
            .map(|chunk| GoxlUnknownChunk {
                id: chunk.id,
                data: chunk.data.clone(),
            })
            .collect(),
    })
}

/// Rebuilds a `16 x 16 x 16` block from the object at listing `index`: each
/// grid cell takes its color from the voxel's sampled material through the
/// object's `baseColor` layer, or the transparent zero voxel when empty.
/// Errors when the object is larger than a block.
fn block_from_object<T>(state: &VoxMain<T>, index: usize, object: &VoxObject) -> Result<GoxlBlock> {
    let size = GoxlBlock::SIZE;
    let bounds = object.bounds();
    if bounds.x > size || bounds.y > size || bounds.z > size {
        return Err(Error::invalid(format!(
            "object {index} is {} x {} x {} but a Goxel block is {size} x {size} x {size}",
            bounds.x, bounds.y, bounds.z
        )));
    }

    let cell_color = resolve_cell_color_or_transparent(state, object)?;
    let mut voxels = Vec::with_capacity((size * size * size) as usize);

    // Storage order is x fastest, then y, then z, matching the loop nesting.
    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let voxel = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .filter(|&voxel_id| object.is_live(voxel_id))
                    .map(|voxel_id| solid_voxel(cell_color.color(voxel_id)))
                    .unwrap_or_default();
                voxels.push(voxel);
            }
        }
    }

    Ok(GoxlBlock { voxels })
}

/// One solid Goxel voxel for a sampled `[r, g, b, a]` color. A live voxel must
/// be present, but Goxel reads alpha 0 as an empty cell, so a fully transparent
/// color is forced opaque; any other alpha is kept.
fn solid_voxel(rgba: [u8; 4]) -> GoxlVoxel {
    let alpha = if rgba[3] == 0 { 255 } else { rgba[3] };
    GoxlVoxel {
        r: rgba[0],
        g: rgba[1],
        b: rgba[2],
        a: alpha,
    }
}

/// Rebuilds the layers from the ext, one per hierarchy node in listing order.
/// A `None` entry stands for a node retained after the load. It becomes a
/// layer like a synthesized one. Errors when the ext is out of step with the
/// hierarchy.
fn build_layers<T>(state: &VoxMain<T>, layers: &[Option<GoxlExtLayer>]) -> Result<Vec<GoxlLayer>> {
    let node_count = state.hierarchy_node_count();
    if layers.len() != node_count {
        return Err(Error::invalid(format!(
            "goxl ext has {} layers but the state has {node_count} hierarchy nodes",
            layers.len()
        )));
    }

    let index_by_object: HashMap<U32Id<BVoxObject>, i32> = state
        .iter_objects()
        .enumerate()
        .map(|(index, (object_id, _))| (object_id, index as i32))
        .collect();
    let ids: HashSet<i32> = layers.iter().flatten().map(|layer| layer.id).collect();
    // A fresh id lands above every stored id and above the no-clone id 0.
    let mut next_id = ids.iter().copied().max().unwrap_or(0).max(0);

    let mut built = Vec::with_capacity(layers.len());
    for (index, ((_, node), layer)) in state.iter_hierarchy_nodes().zip(layers).enumerate() {
        let placed: Vec<i32> = node
            .child_object_ids
            .iter()
            .map(|object_id| {
                *index_by_object
                    .get(object_id)
                    .expect("a placed object is one of the state's")
            })
            .collect();
        let Some(layer) = layer else {
            next_id += 1;
            built.push(synthesized_layer(next_id, node, &placed));
            continue;
        };

        let referenced = distinct(layer.placements.iter().map(|(block, _)| *block));
        if referenced != placed {
            return Err(Error::invalid(format!(
                "goxl ext layer {index} places blocks {referenced:?} but hierarchy node {index} \
                 places objects {placed:?}"
            )));
        }
        if layer.base_id != 0 && !ids.contains(&layer.base_id) {
            return Err(Error::invalid(format!(
                "goxl ext layer {index} clones layer id {} but no layer has it",
                layer.base_id
            )));
        }
        built.push(layer_from_provenance(layer));
    }

    Ok(built)
}

/// A layer for a node retained after the load, shaped like a synthesized one.
/// The node's objects are stamped at its translation rounded to whole voxels.
fn synthesized_layer(id: i32, node: &VoxHierarchyNode, placed: &[i32]) -> GoxlLayer {
    let position = node.transform.position.round().as_ivec3().to_array();
    GoxlLayer {
        name: node.name.clone(),
        id,
        blocks: placed
            .iter()
            .map(|&block_index| GoxlLayerBlock {
                block_index,
                position,
            })
            .collect(),
        ..GoxlLayer::default()
    }
}

/// `values` without repeats, in first-seen order.
fn distinct(values: impl Iterator<Item = i32>) -> Vec<i32> {
    let mut seen = HashSet::new();
    values.filter(|value| seen.insert(*value)).collect()
}

/// Rebuilds one layer from its ext provenance, restoring its placements and the
/// clone or shape definition.
fn layer_from_provenance(layer: &GoxlExtLayer) -> GoxlLayer {
    GoxlLayer {
        name: layer.name.clone(),
        id: layer.id,
        base_id: layer.base_id,
        material: layer.material,
        mode: layer.mode,
        visible: layer.visible,
        transform: layer.transform,
        blocks: layer
            .placements
            .iter()
            .map(|&(block_index, position)| GoxlLayerBlock {
                block_index,
                position,
            })
            .collect(),
        bounding_box: layer.bounding_box,
        image_path: layer.image_path.clone(),
        shape: layer.shape.as_deref().and_then(shape_from_token),
        color: layer.color,
        extra: GoxlDict(layer.extra.clone()),
    }
}

/// The procedural shape for an on-disk shape name, or `None` for an
/// unrecognized one.
fn shape_from_token(token: &str) -> Option<GoxlShape> {
    match token {
        "sphere" => Some(GoxlShape::Sphere),
        "cube" => Some(GoxlShape::Cube),
        "cylinder" => Some(GoxlShape::Cylinder),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{GoxlVoxMain, from_goxl_file, to_goxl_file};
    use branded_id::U32Id;
    use goxl::{
        GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
        GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
    };
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxObject};

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
        let state = from_goxl_file(&file).unwrap();
        assert_eq!(to_goxl_file(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = GoxlFile::default();
        let state = from_goxl_file(&file).unwrap();
        assert_eq!(to_goxl_file(&state).unwrap(), file);
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
        let state = from_goxl_file(&file).unwrap();
        assert_eq!(to_goxl_file(&state).unwrap(), file);
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
        let mut state = from_goxl_file(&file).unwrap();
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

        let rebuilt = to_goxl_file(&state).unwrap();
        let mut want = file;
        want.blocks.remove(1);
        want.layers.remove(1);
        want.layers[0] = stamping_layer("keep", 1, &[(0, [0, 0, 0]), (0, [32, 0, 0])]);
        want.layers[1] = stamping_layer("tail", 3, &[(1, [0, 0, 16]), (1, [0, 0, 32])]);
        assert_eq!(rebuilt, want);

        let reloaded = from_goxl_file(&rebuilt).unwrap();
        assert_eq!(reloaded.ext(), &expected);
    }

    /// A node retained after the load takes no entry. The writer fills it in
    /// like a synthesized layer.
    #[test]
    fn a_node_retained_after_the_load_writes_a_synthesized_layer() {
        let file = placed_blocks_file();
        let mut state = from_goxl_file(&file).unwrap();
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

        let rebuilt = to_goxl_file(&state).unwrap();
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
        let mut state = from_goxl_file(&file).unwrap();
        state
            .move_object(U32Id::<BVoxObject>::from_u32(2), 0)
            .unwrap();

        let rebuilt = to_goxl_file(&state).unwrap();
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
        let mut state = from_goxl_file(&file).unwrap();
        let ext = state.ext_mut();
        ext.layers.pop();
        assert!(to_goxl_file(&state).is_err());

        let mut state = from_goxl_file(&file).unwrap();
        let ext = state.ext_mut();
        let Some(drop) = &mut ext.layers[1] else {
            panic!("layer 1 is loaded");
        };
        drop.placements.push((2, [0, 0, 0]));
        assert!(to_goxl_file(&state).is_err());

        let mut state = from_goxl_file(&file).unwrap();
        let ext = state.ext_mut();
        let Some(drop) = &mut ext.layers[1] else {
            panic!("layer 1 is loaded");
        };
        drop.base_id = 9;
        assert!(to_goxl_file(&state).is_err());
    }

    /// An object larger than a block cannot be one block. The writer errors
    /// instead of truncating it. `to_goxl_vox_main` retiles such a scene.
    #[test]
    fn an_object_larger_than_a_block_errors() {
        let mut state = from_goxl_file(&GoxlFile::default()).unwrap();

        let object_id = state
            .retain_object(VoxObject::new("wide".to_owned(), TyVector3U32::new(17, 1, 1)).unwrap())
            .unwrap();

        let node_id = state
            .retain_hierarchy_node(VoxHierarchyNode {
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();

        state.push_root_hierarchy_node_id(node_id).unwrap();

        assert!(to_goxl_file(&state).is_err());
    }

    #[cfg(feature = "codec")]
    mod codec {
        use super::*;
        use crate::codec::{from_goxl_bytes, to_goxl_bytes};
        use goxl_codec::DependenciesImpl;

        #[test]
        fn round_trips_through_goxl_bytes() {
            let file = sample_file();
            let state = from_goxl_file(&file).unwrap();
            let bytes = to_goxl_bytes(&DependenciesImpl, &state).unwrap();
            let reloaded = from_goxl_bytes(&DependenciesImpl, &bytes).unwrap();
            assert_eq!(to_goxl_file(&reloaded).unwrap(), file);
        }
    }
}
