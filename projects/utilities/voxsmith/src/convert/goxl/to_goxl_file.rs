use crate::{
    GoxelExtWrapper, GoxelLayer, Result, cell_color, ext_for, from_vox_value, object_color_ref,
};
use branded_id::U32Id;
use goxl::{
    GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
    GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
};
use std::collections::{BTreeMap, HashSet};
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxMain, VoxObject};

/// Writes a [`VoxMain`] to a decoded Goxel [`GoxlFile`].
///
/// When the state carries the `goxel` ext the forward path writes, the file is
/// rebuilt losslessly from it, the inverse of
/// [`from_goxl_file`](crate::from_goxl_file): each object emits one `16 x 16 x
/// 16` block, and the layers, materials, cameras, light, preview, and image
/// come from the ext. When the ext is absent or names another format, the file
/// is synthesized from the bare scene instead by `synthesize_goxl`, so any
/// source can be written to Goxel. An empty voxel is written back as the
/// transparent zero voxel.
///
/// Errors only when a `goxel` ext is present but cannot be deserialized;
/// synthesis itself never errors.
pub fn to_goxl_file(state: &VoxMain) -> Result<GoxlFile> {
    let ext = match ext_for(state, "goxel") {
        Some(ext) => from_vox_value::<GoxelExtWrapper>(ext)?.goxel,
        None => return Ok(synthesize_goxl(state)),
    };

    // Each object is the author's build volume (a fixed Goxel 16-cube), so a
    // block is written from it directly at the original positions, its voxel
    // colors read through the object's `baseColorFactor` layer.
    let blocks = state
        .iter_objects()
        .map(|(_, object)| block_from_object(state, object))
        .collect();

    Ok(GoxlFile {
        version: ext.version,
        image: GoxlImage {
            bounding_box: ext.image.bounding_box,
            extra: GoxlDict(ext.image.extra),
        },
        preview: ext.preview.map(|preview| GoxlPreview {
            width: preview.width,
            height: preview.height,
            pixels: preview.pixels,
        }),
        blocks,
        materials: ext
            .materials
            .into_iter()
            .map(|material| GoxlMaterial {
                name: material.name,
                base_color: material.base_color,
                metallic: material.metallic,
                roughness: material.roughness,
                emission: material.emission,
                extra: GoxlDict(material.extra),
            })
            .collect(),
        layers: ext.layers.into_iter().map(layer_from_provenance).collect(),
        cameras: ext
            .cameras
            .into_iter()
            .map(|camera| GoxlCamera {
                name: camera.name,
                distance: camera.distance,
                orthographic: camera.orthographic,
                transform: camera.transform,
                active: camera.active,
                extra: GoxlDict(camera.extra),
            })
            .collect(),
        light: ext.light.map(|light| GoxlLight {
            pitch: light.pitch,
            yaw: light.yaw,
            intensity: light.intensity,
            fixed: light.fixed,
            ambient: light.ambient,
            shadow: light.shadow,
            extra: GoxlDict(light.extra),
        }),
        unknown_chunks: ext
            .unknown_chunks
            .into_iter()
            .map(|chunk| GoxlUnknownChunk {
                id: chunk.id,
                data: chunk.data,
            })
            .collect(),
    })
}

/// Synthesizes a Goxel file from a state that carries no `goxel` ext, such as
/// one cross-loaded from another format.
///
/// Goxel has no scene hierarchy, only flat layers of placed `16 x 16 x 16`
/// blocks, so the voxcore hierarchy is flattened: every object placement
/// becomes one layer whose blocks are tiled from the object's grid and stamped
/// at the placement's world translation, summed down the hierarchy from the
/// roots. An object placed by no node is emitted once at the origin so no
/// geometry is dropped, and an object placed by several nodes is duplicated at
/// each placement.
///
/// Lossy only where Goxel cannot represent the source: node grouping collapses,
/// since layers do not nest, and node rotation and scale are dropped, since
/// only translation survives the flattening. That translation is rounded to
/// whole voxels, since block positions are integer coordinates. Colors stay per
/// voxel with no palette merge. A live voxel must be solid, but Goxel reads
/// alpha 0 as an absent cell, so a fully transparent live color is written
/// opaque, and a voxel with no resolvable color is written opaque black; any
/// other alpha is kept.
fn synthesize_goxl(state: &VoxMain) -> GoxlFile {
    let mut builder = GoxlBuilder::default();
    for &root in state.root_hierarchy_nodes() {
        builder.emit_node(state, root, TyVector3I32::new(0, 0, 0));
    }
    for (id, object) in state.iter_objects() {
        if !builder.placed.contains(&id.to_u32()) {
            builder.emit_object(state, id, object, TyVector3I32::new(0, 0, 0), object.name());
        }
    }

    GoxlFile {
        blocks: builder.blocks,
        layers: builder.layers,
        ..GoxlFile::default()
    }
}

/// Accumulates a flattened Goxel scene: the shared blocks, the layers that
/// place them, and the ids of objects already placed by a hierarchy node.
#[derive(Default)]
struct GoxlBuilder {
    blocks: Vec<GoxlBlock>,
    layers: Vec<GoxlLayer>,
    placed: HashSet<u32>,
    next_id: i32,
}

impl GoxlBuilder {
    /// Walks one hierarchy node, summing its translation into the world
    /// position, emitting a layer for each object it places, then recursing
    /// into its child nodes.
    fn emit_node(
        &mut self,
        state: &VoxMain,
        node_id: U32Id<BVoxHierarchyNode>,
        parent: TyVector3I32,
    ) {
        let (name, child_objects, child_nodes, world) = {
            let node = state
                .hierarchy_node(node_id)
                .expect("a hierarchy id from the state resolves");
            let position = node.transform.position;
            let world = parent + position.round().to_i32();
            (
                node.name.clone(),
                node.child_objects.clone(),
                node.child_nodes.clone(),
                world,
            )
        };

        for object_id in child_objects {
            if let Some(object) = state.object(object_id) {
                self.emit_object(state, object_id, object, world, &name);
            }
        }
        for child in child_nodes {
            self.emit_node(state, child, world);
        }
    }

    /// Emits one layer placing `object` at its world position. The grid is
    /// tiled into `16 x 16 x 16` blocks, each stamped at the world position of
    /// its lower corner; a tile holding no solid voxel is never emitted.
    fn emit_object(
        &mut self,
        state: &VoxMain,
        object_id: U32Id<BVoxObject>,
        object: &VoxObject,
        world: TyVector3I32,
        name: &str,
    ) {
        self.placed.insert(object_id.to_u32());

        // The object is the author's build volume, so each voxel sits at its
        // original local position directly.
        let color = object_color_ref(state, object);
        let edge = GoxlBlock::SIZE as i32;
        let stride = GoxlBlock::SIZE as usize;
        let mut tiles: BTreeMap<[i32; 3], Vec<GoxlVoxel>> = BTreeMap::new();
        for voxel in object.iter_live() {
            let position = object
                .voxel_position(voxel)
                .expect("a live voxel is within the grid");
            let world_position = (world + position.to_i32()).to_array();
            let origin = [
                world_position[0].div_euclid(edge) * edge,
                world_position[1].div_euclid(edge) * edge,
                world_position[2].div_euclid(edge) * edge,
            ];
            let local = [
                (world_position[0] - origin[0]) as usize,
                (world_position[1] - origin[1]) as usize,
                (world_position[2] - origin[2]) as usize,
            ];
            let index = local[0] + stride * (local[1] + stride * local[2]);
            let rgba = color
                .map(|(layer, palette, binding)| {
                    cell_color(state, object, voxel, layer, palette, binding)
                })
                .unwrap_or([0, 0, 0, 0]);
            let block = tiles
                .entry(origin)
                .or_insert_with(|| vec![GoxlVoxel::default(); GoxlBlock::SIZE.pow(3) as usize]);
            block[index] = solid_voxel(rgba);
        }

        let mut blocks = Vec::with_capacity(tiles.len());
        for (origin, voxels) in tiles {
            let block_index = self.blocks.len() as i32;
            self.blocks.push(GoxlBlock { voxels });
            blocks.push(GoxlLayerBlock {
                block_index,
                position: origin,
            });
        }

        self.next_id += 1;
        self.layers.push(GoxlLayer {
            name: name.to_owned(),
            id: self.next_id,
            blocks,
            ..GoxlLayer::default()
        });
    }
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

/// Rebuilds a `16 x 16 x 16` block from an object: each grid cell takes its
/// color from the voxel's sampled material through the object's
/// `baseColorFactor` layer, or the transparent zero voxel when empty.
fn block_from_object(state: &VoxMain, object: &VoxObject) -> GoxlBlock {
    let size = GoxlBlock::SIZE;
    let color = object_color_ref(state, object);
    let mut voxels = Vec::with_capacity((size * size * size) as usize);

    // Storage order is x fastest, then y, then z, matching the loop nesting.
    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let voxel = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .filter(|&id| object.is_live(id))
                    .map(|id| {
                        let [r, g, b, a] = color
                            .map(|(layer, palette, binding)| {
                                cell_color(state, object, id, layer, palette, binding)
                            })
                            .unwrap_or([0, 0, 0, 0]);
                        GoxlVoxel { r, g, b, a }
                    })
                    .unwrap_or_default();
                voxels.push(voxel);
            }
        }
    }

    GoxlBlock { voxels }
}

/// Rebuilds one layer from its ext provenance, restoring its placements and the
/// clone or shape definition.
fn layer_from_provenance(layer: GoxelLayer) -> GoxlLayer {
    GoxlLayer {
        name: layer.name,
        id: layer.id,
        base_id: layer.base_id,
        material: layer.material,
        mode: layer.mode,
        visible: layer.visible,
        transform: layer.transform,
        blocks: layer
            .placements
            .into_iter()
            .map(|(block_index, position)| GoxlLayerBlock {
                block_index,
                position,
            })
            .collect(),
        bounding_box: layer.bounding_box,
        image_path: layer.image_path,
        shape: layer.shape.as_deref().and_then(shape_from_token),
        color: layer.color,
        extra: GoxlDict(layer.extra),
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
