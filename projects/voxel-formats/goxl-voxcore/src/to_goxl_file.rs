use crate::{Error, GoxlExtLayer, GoxlVoxMain, Result};
use branded_id::U32Id;
use goxl::{
    GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
    GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
};
use std::collections::{HashMap, HashSet};
use ty_math::TyVector3U32;
use voxcore::{BVoxObject, VoxHierarchyNode, VoxObject, color::resolve_cell_color_or_transparent};

/// Writes a [`GoxlVoxMain`] to a Goxel [`GoxlFile`], the inverse of
/// [`from_goxl_file`](crate::from_goxl_file). A loaded file writes back
/// exactly through its ext. A state
/// [`to_goxl_vox_main`](crate::to_goxl_vox_main) gave its ext writes as a
/// file synthesized from the scene. Each object emits one `16 x 16 x 16`
/// block and the rest comes from the ext. A layer takes its node's name and
/// stamps its entry's placements. A live voxel is written solid. Goxel reads
/// alpha 0 as an empty cell, so a fully transparent color is forced opaque
/// and any other alpha is kept. An empty cell is the transparent zero voxel.
///
/// Errors if:
///
/// 1. an object is larger than a `16 x 16 x 16` block, which
///    `to_goxl_vox_main` retiles for a bare state
/// 2. an object's `baseColor` draws from a non-color value pool
/// 3. the ext's layers are out of step with the hierarchy: a node has no
///    entry, an entry stamps objects its node does not place, or an entry
///    clones a layer id no entry has
pub fn to_goxl_file(main: &GoxlVoxMain) -> Result<GoxlFile> {
    let ext = main.ext();

    let layers = build_layers(main)?;

    // Each object is the author's build volume, a fixed Goxel 16-cube, so a
    // block is written from it directly at the original positions, its
    // voxel colors read through the object's `baseColor` layer.
    let blocks = main
        .iter_objects()
        .enumerate()
        .map(|(index, (_, object))| block_from_object(main, index, object))
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
fn block_from_object(main: &GoxlVoxMain, index: usize, object: &VoxObject) -> Result<GoxlBlock> {
    let size = GoxlBlock::SIZE;
    let bounds = object.bounds();
    if bounds.x > size || bounds.y > size || bounds.z > size {
        return Err(Error::invalid(format!(
            "object {index} is {} x {} x {} but a Goxel block is {size} x {size} x {size}",
            bounds.x, bounds.y, bounds.z
        )));
    }

    let cell_color = resolve_cell_color_or_transparent(main, object)?;
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
/// Errors when the ext is out of step with the hierarchy.
fn build_layers(main: &GoxlVoxMain) -> Result<Vec<GoxlLayer>> {
    let layers = &main.ext().layers;
    let node_count = main.hierarchy_node_count();
    if layers.len() != node_count {
        return Err(Error::invalid(format!(
            "goxl ext has {} layer entries but the state has {node_count} hierarchy nodes",
            layers.len()
        )));
    }

    let index_by_object: HashMap<U32Id<BVoxObject>, i32> = main
        .iter_objects()
        .enumerate()
        .map(|(index, (object_id, _))| (object_id, index as i32))
        .collect();
    let ids: HashSet<i32> = layers.values().map(|layer| layer.id).collect();

    let mut built = Vec::with_capacity(node_count);
    for (node_id, node) in main.iter_hierarchy_nodes() {
        let Some(layer) = layers.get(&node_id) else {
            return Err(Error::invalid(format!(
                "goxl ext has no layer entry for node {}",
                node_id.to_u32()
            )));
        };

        let stamped = distinct(layer.placements.iter().map(|placement| placement.object_id));
        if stamped != node.child_object_ids {
            return Err(Error::invalid(format!(
                "goxl ext layer for node {} stamps objects {:?} but the node places {:?}",
                node_id.to_u32(),
                bare_ids(&stamped),
                bare_ids(&node.child_object_ids)
            )));
        }
        if layer.base_id != 0 && !ids.contains(&layer.base_id) {
            return Err(Error::invalid(format!(
                "goxl ext layer for node {} clones layer id {} but no entry has it",
                node_id.to_u32(),
                layer.base_id
            )));
        }
        built.push(layer_from_provenance(node, layer, &index_by_object));
    }

    Ok(built)
}

/// `values` without repeats, in first-seen order.
fn distinct(values: impl Iterator<Item = U32Id<BVoxObject>>) -> Vec<U32Id<BVoxObject>> {
    let mut seen = HashSet::new();
    values.filter(|value| seen.insert(*value)).collect()
}

/// `ids` as their bare `u32`s, for an error message.
fn bare_ids(ids: &[U32Id<BVoxObject>]) -> Vec<u32> {
    ids.iter().map(|id| id.to_u32()).collect()
}

/// Rebuilds one layer from its node and ext provenance: the node's name, the
/// placements at their objects' listing indices, and the clone or shape
/// definition.
fn layer_from_provenance(
    node: &VoxHierarchyNode,
    layer: &GoxlExtLayer,
    index_by_object: &HashMap<U32Id<BVoxObject>, i32>,
) -> GoxlLayer {
    GoxlLayer {
        name: node.name.clone(),
        id: layer.id,
        base_id: layer.base_id,
        material: layer.material,
        mode: layer.mode,
        visible: layer.visible,
        transform: layer.transform,
        blocks: layer
            .placements
            .iter()
            .map(|placement| GoxlLayerBlock {
                block_index: *index_by_object
                    .get(&placement.object_id)
                    .expect("a stamped object is one the node places"),
                position: placement.position,
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
    use crate::{GoxlExtPlacement, GoxlVoxMain, from_goxl_file, synthesized_layer, to_goxl_file};
    use branded_id::U32Id;
    use goxl::{
        GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
        GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
    };
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{BVoxHierarchyNode, BVoxObject, Error as VoxError, VoxHierarchyNode, VoxObject};

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

    fn node(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn object(index: u32) -> U32Id<BVoxObject> {
        U32Id::from_u32(index)
    }

    fn placement(object_index: u32, position: [i32; 3]) -> GoxlExtPlacement {
        GoxlExtPlacement {
            object_id: object(object_index),
            position,
        }
    }

    #[test]
    fn round_trips_through_vox_main() {
        let file = sample_file();

        let main = from_goxl_file(&file).unwrap();

        assert_eq!(to_goxl_file(&main).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = GoxlFile::default();

        let main = from_goxl_file(&file).unwrap();

        assert_eq!(to_goxl_file(&main).unwrap(), file);
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

        let main = from_goxl_file(&file).unwrap();

        assert_eq!(to_goxl_file(&main).unwrap(), file);
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

    fn set_child_objects(main: &mut GoxlVoxMain, index: u32, child_object_ids: Vec<u32>) {
        let node_id = node(index);

        let node = VoxHierarchyNode {
            child_object_ids: child_object_ids.into_iter().map(object).collect(),
            ..main.hierarchy_node(node_id).unwrap().clone()
        };

        main.set_hierarchy_node(node_id, node).unwrap();
    }

    /// Dropping the middle layer and its block, then compacting, leaves the
    /// survivors' provenance keyed by their new ids. The first layer keeps its
    /// two stamps of block 0. The last layer stamps its block by its new id.
    /// The rebuilt file reloads to the loaded ext minus the released entries.
    #[test]
    fn released_entities_leave_the_survivors_provenance_aligned() {
        let file = placed_blocks_file();

        let mut main = from_goxl_file(&file).unwrap();

        let original = main.ext().clone();

        main.set_root_hierarchy_node_ids(vec![node(0), node(2)])
            .unwrap();

        set_child_objects(&mut main, 0, vec![0]);

        set_child_objects(&mut main, 1, Vec::new());

        main.release_hierarchy_node(node(1)).unwrap();

        main.release_object(object(1)).unwrap();

        main.gc().unwrap();

        // After the gc, node 2 is node 1 and object 2 is object 1.
        let mut expected = original;

        expected.layers.remove(&node(1));

        let mut tail = expected.layers.remove(&node(2)).unwrap();

        tail.placements = vec![placement(1, [0, 0, 16]), placement(1, [0, 0, 32])];

        expected.layers.insert(node(1), tail);

        expected.layers.get_mut(&node(0)).unwrap().placements =
            vec![placement(0, [0, 0, 0]), placement(0, [32, 0, 0])];

        assert_eq!(main.ext(), &expected);

        let rebuilt = to_goxl_file(&main).unwrap();

        let mut want = file;

        want.blocks.remove(1);

        want.layers.remove(1);

        want.layers[0] = stamping_layer("keep", 1, &[(0, [0, 0, 0]), (0, [32, 0, 0])]);

        want.layers[1] = stamping_layer("tail", 3, &[(1, [0, 0, 16]), (1, [0, 0, 32])]);

        assert_eq!(rebuilt, want);

        let reloaded = from_goxl_file(&rebuilt).unwrap();

        assert_eq!(reloaded.ext(), &expected);
    }

    /// A node retained after the load gets a synthesized entry on the spot:
    /// a fresh layer id, stamping the node's objects at its translation
    /// rounded to whole voxels.
    #[test]
    fn a_node_retained_after_the_load_gets_a_synthesized_entry() {
        let file = placed_blocks_file();

        let mut main = from_goxl_file(&file).unwrap();

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "added".to_owned(),
                child_object_ids: vec![object(2)],
                transform: TyTransformF64 {
                    position: TyVector3F64::new(3.4, -2.6, 16.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();

        main.push_root_hierarchy_node_id(node_id).unwrap();

        assert_eq!(
            main.ext().layers[&node_id],
            synthesized_layer(4, vec![placement(2, [3, -3, 16])])
        );

        let rebuilt = to_goxl_file(&main).unwrap();

        let mut want = file;

        want.layers
            .push(stamping_layer("added", 4, &[(2, [3, -3, 16])]));

        assert_eq!(rebuilt, want);
    }

    /// Moving the last block to the front changes no id, so each layer still
    /// stamps the object it did, at its new listing index.
    #[test]
    fn a_moved_object_keeps_each_layers_block() {
        let file = placed_blocks_file();

        let mut main = from_goxl_file(&file).unwrap();

        main.move_object(object(2), 0).unwrap();

        let rebuilt = to_goxl_file(&main).unwrap();

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

    /// Releasing a layer that another layer clones would leave the clone's
    /// `base_id` dangling. The release is refused and nothing changes. The
    /// clone releases first, then the base.
    #[test]
    fn releasing_a_cloned_base_layer_is_refused() {
        let file = sample_file();

        let mut main = from_goxl_file(&file).unwrap();

        main.set_root_hierarchy_node_ids(vec![node(2)]).unwrap();

        assert!(matches!(
            main.release_hierarchy_node(node(0)),
            Err(VoxError::Ext { .. })
        ));

        assert!(main.hierarchy_node(node(0)).is_some());

        assert_eq!(main.ext().layers.len(), 3);

        main.release_hierarchy_node(node(1)).unwrap();

        main.release_hierarchy_node(node(0)).unwrap();

        assert_eq!(main.ext().layers.len(), 1);

        assert_eq!(
            to_goxl_file(&main).unwrap().layers,
            vec![file.layers[2].clone()]
        );
    }

    /// An ext out of step with the hierarchy is malformed. The writer errors
    /// instead of stamping what the state does not place or cloning a layer
    /// the file lacks.
    #[test]
    fn an_ext_out_of_step_with_its_listings_errors() {
        let file = placed_blocks_file();

        let mut main = from_goxl_file(&file).unwrap();

        main.ext_mut().layers.remove(&node(2));

        assert!(to_goxl_file(&main).is_err());

        let mut main = from_goxl_file(&file).unwrap();

        main.ext_mut()
            .layers
            .get_mut(&node(1))
            .unwrap()
            .placements
            .push(placement(2, [0, 0, 0]));

        assert!(to_goxl_file(&main).is_err());

        let mut main = from_goxl_file(&file).unwrap();

        main.ext_mut().layers.get_mut(&node(1)).unwrap().base_id = 9;

        assert!(to_goxl_file(&main).is_err());
    }

    /// An object larger than a block cannot be one block. The writer errors
    /// instead of truncating it. `to_goxl_vox_main` retiles such a scene.
    #[test]
    fn an_object_larger_than_a_block_errors() {
        let mut main = from_goxl_file(&GoxlFile::default()).unwrap();

        let object_id = main
            .retain_object(VoxObject::new("wide".to_owned(), TyVector3U32::new(17, 1, 1)).unwrap())
            .unwrap();

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();

        main.push_root_hierarchy_node_id(node_id).unwrap();

        assert!(to_goxl_file(&main).is_err());
    }

    #[cfg(feature = "codec")]
    mod codec {
        use super::*;
        use crate::codec::{from_goxl_bytes, to_goxl_bytes};
        use goxl_codec::DependenciesImpl;

        #[test]
        fn round_trips_through_goxl_bytes() {
            let file = sample_file();

            let main = from_goxl_file(&file).unwrap();

            let bytes = to_goxl_bytes(&DependenciesImpl, &main).unwrap();

            let reloaded = from_goxl_bytes(&DependenciesImpl, &bytes).unwrap();

            assert_eq!(to_goxl_file(&reloaded).unwrap(), file);
        }
    }
}
