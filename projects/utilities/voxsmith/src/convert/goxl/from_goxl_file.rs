use crate::{
    BASE_COLOR_FACTOR, Error, GoxelCamera, GoxelExt, GoxelExtWrapper, GoxelImage, GoxelLayer,
    GoxelLight, GoxelMaterial, GoxelPreview, GoxelUnknownChunk, Result, to_vox_value,
};
use branded_id::U32Id;
use goxl::{GoxlBlock, GoxlCamera, GoxlFile, GoxlLayer, GoxlLight, GoxlMaterial, GoxlShape};
use std::collections::{HashMap, HashSet};
use ty_math::{TySrgbaU8, TyTransformF64, TyVector3U32};
use voxcore::{
    BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
    VoxValuePool,
};

/// Loads a decoded Goxel [`GoxlFile`] into a [`VoxMain`].
///
/// The shared `BL16` voxel blocks become objects sharing one `baseColorFactor`
/// palette, and the `LAYR` layers become the hierarchy nodes, each placing the
/// blocks it stamps. The state with no native voxcore home, such as the image
/// metadata, preview, materials, cameras, light, per-layer metadata, the clone
/// and shape definitions, the exact block placements, and unknown chunks, rides
/// in a `goxel` ext so the file can be written back exactly.
///
/// Errors on a layer placement that references a block outside the block list,
/// or if [`VoxMain::validate`](voxcore::VoxMain::validate) rejects the result.
pub fn from_goxl_file(file: &GoxlFile) -> Result<VoxMain> {
    let mut state = VoxMain::default();

    let (palette, materials) = build_palette(&mut state, file);
    let palette_id = state.add_palette(palette);

    for block in &file.blocks {
        // A Goxel block is a fixed 16-cube; it becomes the object's build
        // volume directly, with its live voxels wherever they sit inside it.
        state.add_object(build_object(block, palette_id, &materials));
    }

    let nodes = build_layer_nodes(file, state.object_count())?;
    let roots = (0..nodes.len())
        .map(|index| U32Id::from_u32(index as u32))
        .collect();
    for node in nodes {
        state.add_hierarchy_node(node);
    }
    state.set_root_hierarchy_nodes(roots);

    let ext = GoxelExtWrapper {
        goxel: goxel_ext(file),
    };
    state.set_ext(Some(to_vox_value(&ext)?));

    state.validate()?;
    Ok(state)
}

/// Builds the one shared palette: a color pool of one entry per distinct color
/// across every block's solid voxels, bound to `baseColorFactor`, with one
/// material per color and a map from a color to its material. The pool is added
/// to `state`. A file with no solid voxels gets a single placeholder color so
/// objects have a default material to sample.
fn build_palette(
    state: &mut VoxMain,
    file: &GoxlFile,
) -> (VoxPalette, HashMap<[u8; 4], U32Id<BVoxMaterial>>) {
    let mut order: Vec<[u8; 4]> = Vec::new();
    let mut seen: HashSet<[u8; 4]> = HashSet::new();
    for block in &file.blocks {
        for voxel in &block.voxels {
            if voxel.is_empty() {
                continue;
            }
            let color = [voxel.r, voxel.g, voxel.b, voxel.a];
            if seen.insert(color) {
                order.push(color);
            }
        }
    }
    if order.is_empty() {
        order.push([0, 0, 0, 0]);
    }

    // Colors ride in a shared sRGBA pool as float components in `[0, 1]`; each
    // material draws one value id into it.
    let pool = state.add_value_pool(VoxValuePool::Srgba {
        values: order
            .iter()
            .map(|&color| TySrgbaU8::from_array(color).to_f64().to_array())
            .collect(),
    });

    let mut palette = VoxPalette::default();
    palette.add_array_property(BASE_COLOR_FACTOR.to_owned(), pool);
    let mut materials = HashMap::with_capacity(order.len());
    for (index, color) in order.iter().enumerate() {
        let material = palette
            .add_material(vec![U32Id::from_u32(index as u32)])
            .expect("one value id for the one array property");
        materials.insert(*color, material);
    }

    (palette, materials)
}

/// Builds an object from a block: a dense `16 x 16 x 16` grid referencing the
/// shared palette on one layer, each solid voxel sampling its color material.
fn build_object(
    block: &GoxlBlock,
    palette: U32Id<BVoxPalette>,
    materials: &HashMap<[u8; 4], U32Id<BVoxMaterial>>,
) -> VoxObject {
    let size = GoxlBlock::SIZE;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size, size, size))
        .expect("a 16-cubed block fits the dense grid");

    // The single layer is the color; solid voxels overwrite material 0.
    object.add_layer(palette, U32Id::<BVoxMaterial>::from_u32(0));

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let Some(voxel) = block.voxel(x, y, z) else {
                    continue;
                };
                if voxel.is_empty() {
                    continue;
                }
                let color = [voxel.r, voxel.g, voxel.b, voxel.a];
                let material = materials
                    .get(&color)
                    .copied()
                    .expect("every solid color is in the palette");
                let id = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .expect("a coordinate inside the block is inside the grid");
                object
                    .retain_voxel(id, &[material])
                    .expect("one sample for the one layer");
            }
        }
    }

    object
}

/// Builds the hierarchy nodes, one per layer in stored order. A layer node
/// places the distinct blocks it stamps, deduplicated to satisfy voxcore's
/// per-node uniqueness rule; the exact placement list rides in the ext. Errors
/// on a placement that references a block outside the block list.
fn build_layer_nodes(file: &GoxlFile, object_count: usize) -> Result<Vec<VoxHierarchyNode>> {
    let mut nodes = Vec::with_capacity(file.layers.len());
    for layer in &file.layers {
        let mut child_objects = Vec::new();
        let mut seen = HashSet::new();
        for placement in &layer.blocks {
            let index = placement.block_index;
            if index < 0 || index as usize >= object_count {
                return Err(Error::invalid(format!(
                    "layer block placement references block {index}, which does not exist"
                )));
            }
            if seen.insert(index) {
                child_objects.push(U32Id::<BVoxObject>::from_u32(index as u32));
            }
        }
        nodes.push(VoxHierarchyNode {
            name: layer.name.clone(),
            child_nodes: Vec::new(),
            child_objects,
            transform: TyTransformF64::default(),
        });
    }
    Ok(nodes)
}

/// Builds the `goxel` ext payload from the state with no native home.
fn goxel_ext(file: &GoxlFile) -> GoxelExt {
    GoxelExt {
        version: file.version,
        image: GoxelImage {
            bounding_box: file.image.bounding_box,
            extra: file.image.extra.0.clone(),
        },
        preview: file.preview.as_ref().map(|preview| GoxelPreview {
            width: preview.width,
            height: preview.height,
            pixels: preview.pixels.clone(),
        }),
        materials: file.materials.iter().map(material_provenance).collect(),
        layers: file.layers.iter().map(layer_provenance).collect(),
        cameras: file.cameras.iter().map(camera_provenance).collect(),
        light: file.light.as_ref().map(light_provenance),
        unknown_chunks: file
            .unknown_chunks
            .iter()
            .map(|chunk| GoxelUnknownChunk {
                id: chunk.id,
                data: chunk.data.clone(),
            })
            .collect(),
    }
}

/// The ext provenance for one layer: its metadata, clone and shape definition,
/// and the full placement list.
fn layer_provenance(layer: &GoxlLayer) -> GoxelLayer {
    GoxelLayer {
        name: layer.name.clone(),
        id: layer.id,
        base_id: layer.base_id,
        material: layer.material,
        mode: layer.mode,
        visible: layer.visible,
        transform: layer.transform,
        bounding_box: layer.bounding_box,
        image_path: layer.image_path.clone(),
        shape: layer.shape.map(shape_token),
        color: layer.color,
        placements: layer
            .blocks
            .iter()
            .map(|block| (block.block_index, block.position))
            .collect(),
        extra: layer.extra.0.clone(),
    }
}

/// The ext provenance for one material.
fn material_provenance(material: &GoxlMaterial) -> GoxelMaterial {
    GoxelMaterial {
        name: material.name.clone(),
        base_color: material.base_color,
        metallic: material.metallic,
        roughness: material.roughness,
        emission: material.emission,
        extra: material.extra.0.clone(),
    }
}

/// The ext provenance for one camera.
fn camera_provenance(camera: &GoxlCamera) -> GoxelCamera {
    GoxelCamera {
        name: camera.name.clone(),
        distance: camera.distance,
        orthographic: camera.orthographic,
        transform: camera.transform,
        active: camera.active,
        extra: camera.extra.0.clone(),
    }
}

/// The ext provenance for the light settings.
fn light_provenance(light: &GoxlLight) -> GoxelLight {
    GoxelLight {
        pitch: light.pitch,
        yaw: light.yaw,
        intensity: light.intensity,
        fixed: light.fixed,
        ambient: light.ambient,
        shadow: light.shadow,
        extra: light.extra.0.clone(),
    }
}

/// The on-disk shape name for a procedural shape.
fn shape_token(shape: GoxlShape) -> String {
    match shape {
        GoxlShape::Sphere => "sphere".to_owned(),
        GoxlShape::Cube => "cube".to_owned(),
        GoxlShape::Cylinder => "cylinder".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR_FACTOR, from_goxl_bytes, from_goxl_file, to_goxl_bytes, to_goxl_file};
    use branded_id::U32Id;
    use goxl::{
        GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
        GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
    };
    use std::collections::BTreeSet;
    use ty_math::{TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, VoxHierarchyNode, VoxMain, VoxMap, VoxObject,
        VoxPalette, VoxValue, VoxValuePool,
    };

    /// The float sRGB components in `[0, 1]` of a `#RRGGBBAA` hex string.
    fn srgba(hex: &str) -> [f64; 4] {
        TySrgbaU8::from_hex(hex)
            .expect("a valid hex color")
            .to_f64()
            .to_array()
    }

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

    /// A state with no format ext, built straight from voxcore: a red-green
    /// object and a blue object sharing one `rgba` palette, placed by a
    /// hierarchy of a nested group and two roots. This is the cross-format
    /// synthesis input.
    fn source_state() -> VoxMain {
        let mut state = VoxMain::default();

        // One baseColorFactor palette: a transparent placeholder, then red,
        // green, blue.
        let pool = state.add_value_pool(VoxValuePool::Srgba {
            values: ["#00000000", "#FF0000FF", "#00FF00FF", "#0000FFFF"]
                .iter()
                .map(|hex| srgba(hex))
                .collect(),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property(BASE_COLOR_FACTOR.to_owned(), pool);
        for index in 0..4 {
            palette
                .add_material(vec![U32Id::from_u32(index)])
                .expect("one value id for the one array property");
        }
        let palette_id = state.add_palette(palette);
        let material = |index: u32| U32Id::<BVoxMaterial>::from_u32(index);

        // Object 0: a red then a green voxel along x.
        let mut wide = VoxObject::new(String::new(), TyVector3U32::new(2, 1, 1))
            .expect("a 2x1x1 grid is within the dense limit");
        wide.add_layer(palette_id, material(0));
        for (x, color) in [(0u32, 1u32), (1, 2)] {
            let voxel = wide
                .voxel_id(TyVector3U32::new(x, 0, 0))
                .expect("a position within the grid");
            wide.retain_voxel(voxel, &[material(color)])
                .expect("one sample for the one layer");
        }
        state.add_object(wide);

        // Object 1: a single blue voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.add_layer(palette_id, material(0));
        let voxel = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel, &[material(3)])
            .expect("one sample for the one layer");
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

    /// A solid voxel in world space: `x`, `y`, `z`, and an `rgba` color.
    type WorldVoxel = (i32, i32, i32, (u8, u8, u8, u8));

    /// The world voxels a file places: each layer block's solid cells decoded
    /// to world coordinates and color, order-independent.
    fn world_voxels(file: &GoxlFile) -> BTreeSet<WorldVoxel> {
        let stride = GoxlBlock::SIZE as usize;
        let mut set = BTreeSet::new();
        for layer in &file.layers {
            for placement in &layer.blocks {
                let block = &file.blocks[placement.block_index as usize];
                for (index, voxel) in block.voxels.iter().enumerate() {
                    if voxel.is_empty() {
                        continue;
                    }
                    let x = index % stride;
                    let y = index / stride % stride;
                    let z = index / (stride * stride);
                    set.insert((
                        placement.position[0] + x as i32,
                        placement.position[1] + y as i32,
                        placement.position[2] + z as i32,
                        (voxel.r, voxel.g, voxel.b, voxel.a),
                    ));
                }
            }
        }
        set
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_goxl_file(&file).unwrap();
        assert_eq!(to_goxl_file(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_through_goxl_bytes() {
        let file = sample_file();
        let state = from_goxl_file(&file).unwrap();
        let bytes = to_goxl_bytes(&state).unwrap();
        let reloaded = from_goxl_bytes(&bytes).unwrap();
        assert_eq!(to_goxl_file(&reloaded).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = GoxlFile::default();
        let state = from_goxl_file(&file).unwrap();
        assert_eq!(to_goxl_file(&state).unwrap(), file);
    }

    /// A default state has no objects and no ext, so the writer synthesizes an
    /// empty file rather than erroring on the missing ext.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let state = VoxMain::default();
        let file = to_goxl_file(&state).unwrap();
        assert!(file.blocks.is_empty());
        assert!(file.layers.is_empty());
    }

    /// A state with no `goxel` ext, such as one cross-loaded from another
    /// format, synthesizes a file: the hierarchy flattens to layers of placed
    /// blocks whose world voxels and colors match the source, one layer per
    /// placement named for its node, and the file reads back into a valid
    /// state.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let state = source_state();
        let file = to_goxl_file(&state).unwrap();

        let red = (0xFF, 0, 0, 0xFF);
        let green = (0, 0xFF, 0, 0xFF);
        let blue = (0, 0, 0xFF, 0xFF);

        // Object 0 is placed at +5x under a group, object 1 at +3y.
        assert_eq!(
            world_voxels(&file),
            BTreeSet::from([(5, 0, 0, red), (6, 0, 0, green), (0, 3, 0, blue)])
        );

        assert_eq!(file.layers.len(), 2);
        assert_eq!(file.layers[0].name, "wide");
        assert_eq!(file.layers[1].name, "unit");

        // Each object tiles to one block, and each block reads back as its own
        // object in a valid state.
        assert_eq!(file.blocks.len(), 2);
        let reloaded = from_goxl_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 2);
    }

    /// A foreign ext, here `voxel-max`, is ignored by the Goxel writer: it
    /// synthesizes from the scene rather than failing to parse the wrong ext.
    #[test]
    fn synthesizes_past_a_foreign_ext() {
        let mut state = source_state();
        state.set_ext(Some(VoxValue::Object(VoxMap(vec![(
            "voxel-max".to_owned(),
            VoxValue::Null,
        )]))));
        let file = to_goxl_file(&state).unwrap();
        assert_eq!(file.layers.len(), 2);
    }

    #[test]
    fn rejects_a_dangling_layer_block_placement() {
        let file = GoxlFile {
            layers: vec![GoxlLayer {
                blocks: vec![GoxlLayerBlock {
                    block_index: 5,
                    position: [0, 0, 0],
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(from_goxl_file(&file).is_err());
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
}
