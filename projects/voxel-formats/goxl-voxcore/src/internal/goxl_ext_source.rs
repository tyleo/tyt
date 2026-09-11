use crate::{
    Error, Result,
    ext::{GoxlExt, GoxlExtLayer},
};
use branded_id::U32Id;
use goxl::{
    GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
    GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxObject, VoxExt, VoxHierarchyNode, VoxMain, VoxObject,
    color::resolve_cell_color_or_transparent,
};

/// Writes a state to a Goxel file. [`GoxlExt`] rebuilds the loaded file.
/// `()` synthesizes one from the scene.
pub trait GoxlExtSource: VoxExt + Sized {
    /// Writes `state` to a [`GoxlFile`].
    fn write_goxl(state: &VoxMain<Self>) -> Result<GoxlFile>;
}

impl GoxlExtSource for GoxlExt {
    /// Writes a state to a Goxel [`GoxlFile`] through its ext, so a loaded file
    /// rebuilds exactly: each object emits one `16 x 16 x 16` block and the
    /// rest comes from the ext. An empty voxel is written back as the
    /// transparent zero voxel.
    ///
    /// Errors when an object's `baseColor` draws from a non-color value pool,
    /// or when the ext's layers are out of step with the hierarchy.
    fn write_goxl(state: &VoxMain<Self>) -> Result<GoxlFile> {
        let ext = state.ext();

        let layers = build_layers(state, &ext.layers)?;

        // Each object is the author's build volume, a fixed Goxel 16-cube, so a
        // block is written from it directly at the original positions, its
        // voxel colors read through the object's `baseColor` layer.
        let blocks = state
            .iter_objects()
            .map(|(_, object)| block_from_object(state, object))
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
}

/// Rebuilds a `16 x 16 x 16` block from an object: each grid cell takes its
/// color from the voxel's sampled material through the object's
/// `baseColor` layer, or the transparent zero voxel when empty.
fn block_from_object<T>(state: &VoxMain<T>, object: &VoxObject) -> Result<GoxlBlock> {
    let size = GoxlBlock::SIZE;
    let cell_color = resolve_cell_color_or_transparent(state, object)?;
    let mut voxels = Vec::with_capacity((size * size * size) as usize);

    // Storage order is x fastest, then y, then z, matching the loop nesting.
    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let voxel = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .filter(|&voxel_id| object.is_live(voxel_id))
                    .map(|voxel_id| {
                        let [r, g, b, a] = cell_color.color(voxel_id);
                        GoxlVoxel { r, g, b, a }
                    })
                    .unwrap_or_default();
                voxels.push(voxel);
            }
        }
    }

    Ok(GoxlBlock { voxels })
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

impl GoxlExtSource for () {
    /// Synthesizes a Goxel file from the bare scene of a state carrying no
    /// `goxl` ext, such as one cross-loaded from another format.
    ///
    /// Goxel has no scene hierarchy, only flat layers of placed `16 x 16 x 16`
    /// blocks, so the voxcore hierarchy is flattened: every object placement
    /// becomes one layer whose blocks are tiled from the object's grid and
    /// stamped at the placement's world translation, summed down the hierarchy
    /// from the roots. An object placed by no node is emitted once at the
    /// origin so no geometry is dropped. An object placed by several nodes is
    /// duplicated at each placement.
    ///
    /// Lossy only where Goxel cannot represent the source. Node grouping
    /// collapses because layers do not nest. Node rotation and scale drop
    /// because only translation survives the flattening. That translation
    /// rounds to whole voxels because block positions are integer coordinates.
    /// Colors stay per voxel with no palette merge. Goxel reads alpha 0 as an
    /// absent cell and a live voxel must be solid, so a fully transparent live
    /// color is written opaque. A voxel with no resolvable color is written
    /// opaque black. Any other alpha is kept.
    fn write_goxl(state: &VoxMain<Self>) -> Result<GoxlFile> {
        let mut builder = GoxlBuilder::default();
        for &root_id in state.root_hierarchy_node_ids() {
            builder.emit_node(state, root_id, TyVector3I32::new(0, 0, 0))?;
        }
        for (object_id, object) in state.iter_objects() {
            if !builder.placed.contains(&object_id.to_u32()) {
                builder.emit_object(
                    state,
                    object_id,
                    object,
                    TyVector3I32::new(0, 0, 0),
                    object.name(),
                )?;
            }
        }

        Ok(GoxlFile {
            blocks: builder.blocks,
            layers: builder.layers,
            ..GoxlFile::default()
        })
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
    fn emit_node<T>(
        &mut self,
        state: &VoxMain<T>,
        node_id: U32Id<BVoxHierarchyNode>,
        parent: TyVector3I32,
    ) -> Result<()> {
        let (name, child_object_ids, child_node_ids, world) = {
            let node = state
                .hierarchy_node(node_id)
                .expect("a hierarchy id from the state resolves");
            let position = node.transform.position;
            let world = parent + position.round().as_ivec3();
            (
                node.name.clone(),
                node.child_object_ids.clone(),
                node.child_node_ids.clone(),
                world,
            )
        };

        for object_id in child_object_ids {
            if let Some(object) = state.object(object_id) {
                self.emit_object(state, object_id, object, world, &name)?;
            }
        }
        for child_id in child_node_ids {
            self.emit_node(state, child_id, world)?;
        }

        Ok(())
    }

    /// Emits one layer placing `object` at its world position. The grid is
    /// tiled into `16 x 16 x 16` blocks, each stamped at the world position of
    /// its lower corner; a tile holding no solid voxel is never emitted.
    fn emit_object<T>(
        &mut self,
        state: &VoxMain<T>,
        object_id: U32Id<BVoxObject>,
        object: &VoxObject,
        world: TyVector3I32,
        name: &str,
    ) -> Result<()> {
        self.placed.insert(object_id.to_u32());

        // The object is the author's build volume, so each voxel sits at its
        // original local position directly.
        let cell_color = resolve_cell_color_or_transparent(state, object)?;
        let edge = GoxlBlock::SIZE as i32;
        let stride = GoxlBlock::SIZE as usize;
        let mut tiles: BTreeMap<[i32; 3], Vec<GoxlVoxel>> = BTreeMap::new();
        for voxel_id in object.iter_live() {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            let world_position = (world + position.as_ivec3()).to_array();
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
            let rgba = cell_color.color(voxel_id);
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

        Ok(())
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
