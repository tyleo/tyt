use crate::{
    BASE_COLOR_FACTOR, Error, MagicaVoxelExt, MagicaVoxelExtWrapper, MagicaVoxelFrame,
    MagicaVoxelNodeBody, Result, cell_color, ext_for, from_vox_value, object_color_ref, pool_color,
};
use branded_id::U32Id;
use mvox::{
    MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer, MVoxMaterial,
    MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette, MVoxRenderObject, MVoxRotation,
    MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel, MVoxShapeNode, MVoxTransformNode,
    MVoxUnknownChunk, MVoxVoxel,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use ty_math::TyVector3F64;
use voxcore::{BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxMain, VoxObject};

/// Writes a [`VoxMain`] back to a decoded MagicaVoxel [`MVoxFile`], the inverse
/// of [`from_mvox_file`](crate::from_mvox_file).
///
/// A state carrying the `magica-voxel` ext the forward path writes is rebuilt
/// from it exactly. A state without that ext, such as one loaded from another
/// format, is synthesized from the bare voxcore scene by `synthesize_mvox`.
/// Each object emits one model whose voxels are listed in ascending raster
/// order, which need not match their original stored order.
///
/// Errors if the ext is present but its per-node entries do not line up with
/// the hierarchy, or if synthesis exceeds a MagicaVoxel limit such as the
/// per-axis voxel cap.
pub fn to_mvox_file(state: &VoxMain) -> Result<MVoxFile> {
    let ext = match ext_for(state, "magica-voxel") {
        Some(ext) => from_vox_value::<MagicaVoxelExtWrapper>(ext)?.magica_voxel,
        None => return synthesize_mvox(state),
    };

    // The forward path adds exactly one palette and references it from every
    // object on one layer; each material's color resolves through its pool.
    let palette = state.iter_palettes().next().map(|(id, _)| id);
    let file_palette = ext.palette_present.then(|| MVoxPalette {
        colors: colors_from_palette(state, palette),
    });

    let materials = build_materials(&ext);
    // Each object is the author's build volume, so the written model keeps its
    // dimensions and voxel positions directly.
    let models = state
        .iter_objects()
        .map(|(_, object)| model_from_object(object))
        .collect();
    let scene_nodes = build_scene_nodes(state, &ext)?;

    let layers = ext
        .layers
        .iter()
        .map(|layer| MVoxLayer {
            id: layer.id,
            name: layer.name.clone(),
            hidden: layer.hidden,
            extra: MVoxDict(layer.extra.clone()),
        })
        .collect();
    let render_objects = ext
        .render_objects
        .iter()
        .map(|attributes| MVoxRenderObject {
            attributes: MVoxDict(attributes.clone()),
        })
        .collect();
    let cameras = ext
        .cameras
        .iter()
        .map(|camera| MVoxCamera {
            id: camera.id,
            mode: camera.mode.clone(),
            focus: camera.focus,
            angle: camera.angle,
            radius: camera.radius,
            frustum: camera.frustum,
            fov: camera.fov,
            extra: MVoxDict(camera.extra.clone()),
        })
        .collect();
    let unknown_chunks = ext
        .unknown_chunks
        .iter()
        .map(|chunk| MVoxUnknownChunk {
            id: chunk.id,
            content: chunk.content.clone(),
            children: chunk.children.clone(),
        })
        .collect();

    Ok(MVoxFile {
        version: ext.version,
        models,
        palette: file_palette,
        scene_nodes,
        materials,
        layers,
        render_objects,
        cameras,
        palette_notes: ext.palette_notes,
        index_map: ext
            .index_map
            .as_deref()
            .and_then(|map| <[u8; 256]>::try_from(map).ok()),
        unknown_chunks,
    })
}

/// The 256 palette colors read back through `baseColorFactor`: material `index`
/// gives color index `index`. Transparent where the palette or its color
/// binding is absent.
fn colors_from_palette(state: &VoxMain, palette: Option<U32Id<BVoxPalette>>) -> [MVoxColor; 256] {
    let mut colors = [MVoxColor::default(); 256];
    let Some(palette) = palette else {
        return colors;
    };
    let Some(binding) = state
        .palette(palette)
        .and_then(|palette| palette.binding_by_attribute(BASE_COLOR_FACTOR))
    else {
        return colors;
    };
    for (index, color) in colors.iter_mut().enumerate() {
        let material = U32Id::<BVoxMaterial>::from_u32(index as u32);
        if let Some([r, g, b, a]) = state
            .material_value(palette, material, binding)
            .and_then(|(pool, value)| pool_color(pool, value))
        {
            *color = MVoxColor::new(r, g, b, a);
        }
    }
    colors
}

/// Rebuilds the materials from the ext, which holds each one's exact optional
/// fields, so an absent field round-trips as absent. The palette pools carry
/// only a default-substituted neutral copy.
fn build_materials(ext: &MagicaVoxelExt) -> Vec<MVoxMaterial> {
    ext.materials
        .iter()
        .map(|material| MVoxMaterial {
            id: material.id,
            material_type: material
                .material_type
                .as_deref()
                .map(material_type_from_token),
            weight: material.weight,
            rough: material.rough,
            spec: material.spec,
            ior: material.ior,
            att: material.att,
            flux: material.flux,
            extra: MVoxDict(material.extra.clone()),
        })
        .collect()
}

/// The material shading model for a `_type` token. Known tokens map to their
/// variants; anything else is preserved as-is.
fn material_type_from_token(token: &str) -> MVoxMaterialType {
    match token {
        "_diffuse" => MVoxMaterialType::Diffuse,
        "_metal" => MVoxMaterialType::Metal,
        "_glass" => MVoxMaterialType::Glass,
        "_emit" => MVoxMaterialType::Emit,
        other => MVoxMaterialType::Other(other.to_owned()),
    }
}

/// Builds a model from an object: its size from the grid bounds and one voxel
/// per live cell, in ascending raster order, each taking its color index from
/// the material it samples on the object's first layer. A material's index is
/// its color index.
fn model_from_object(object: &VoxObject) -> MVoxModel {
    let bounds = object.bounds();
    let layer = object.iter_layers().next().map(|(layer, _)| layer);

    let voxels = object
        .iter_live()
        .map(|voxel_id| {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            let color_index = layer
                .and_then(|layer| object.voxel_material(voxel_id, layer))
                .map_or(0, |material| material.to_u32() as u8);
            MVoxVoxel {
                x: position.x as u8,
                y: position.y as u8,
                z: position.z as u8,
                color_index,
            }
        })
        .collect();

    MVoxModel {
        size: [bounds.x, bounds.y, bounds.z],
        voxels,
    }
}

/// Rebuilds the scene nodes from the ext, one per entry in stored order. The
/// references come from the ext, which holds the exact lists, so a shape that
/// draws one model on several frames or any other repeated reference
/// round-trips. Errors if the ext node count does not match the hierarchy.
fn build_scene_nodes(state: &VoxMain, ext: &MagicaVoxelExt) -> Result<Vec<MVoxSceneNode>> {
    let node_count = state.hierarchy_node_count();
    if node_count != ext.scene_nodes.len() {
        return Err(Error::Invalid(format!(
            "magica-voxel ext has {} scene nodes but the state has {node_count} hierarchy nodes",
            ext.scene_nodes.len()
        )));
    }

    Ok(ext
        .scene_nodes
        .iter()
        .map(|provenance| MVoxSceneNode {
            id: provenance.id,
            attributes: MVoxNodeAttributes {
                name: provenance.name.clone(),
                hidden: provenance.hidden,
                extra: MVoxDict(provenance.attr_extra.clone()),
            },
            body: match &provenance.body {
                MagicaVoxelNodeBody::Transform {
                    child,
                    layer,
                    frames,
                } => MVoxSceneNodeBody::Transform(MVoxTransformNode {
                    child: *child,
                    layer: *layer,
                    frames: frames.iter().map(frame_from_provenance).collect(),
                }),
                MagicaVoxelNodeBody::Group { children } => {
                    MVoxSceneNodeBody::Group(MVoxGroupNode {
                        children: children.clone(),
                    })
                }
                MagicaVoxelNodeBody::Shape { models } => MVoxSceneNodeBody::Shape(MVoxShapeNode {
                    models: models
                        .iter()
                        .map(|model| MVoxShapeModel {
                            model: model.model,
                            frame_index: model.frame_index,
                            extra: MVoxDict(model.extra.clone()),
                        })
                        .collect(),
                }),
            },
        })
        .collect())
}

/// Rebuilds one transform-node frame from its ext provenance.
fn frame_from_provenance(frame: &MagicaVoxelFrame) -> MVoxFrame {
    MVoxFrame {
        rotation: MVoxRotation(frame.rotation),
        translation: frame.translation,
        frame_index: frame.frame_index,
        extra: MVoxDict(frame.extra.clone()),
    }
}

/// Synthesizes a MagicaVoxel file from a state that carries no `magica-voxel`
/// ext, such as one loaded from another format. Builds one model per object, a
/// single global 256-color palette gathering every distinct color used, and a
/// scene graph mirroring the voxcore hierarchy.
///
/// Lossy only at MagicaVoxel's own limits: an object grid may be at most 256
/// voxels per axis, since voxel coordinates are bytes, and a scene transform
/// carries only translation, so node rotation and scale are dropped. The
/// palette is lossless up to 255 distinct colors; beyond that the table is full
/// and the excess collapses onto the last slot.
fn synthesize_mvox(state: &VoxMain) -> Result<MVoxFile> {
    let (palette, color_index) = synthesize_palette(state);
    let models = state
        .iter_objects()
        .map(|(_, object)| synthesize_model(state, object, &color_index))
        .collect::<Result<Vec<_>>>()?;

    Ok(MVoxFile {
        version: 150,
        models,
        palette: Some(palette),
        scene_nodes: synthesize_scene(state),
        ..MVoxFile::default()
    })
}

/// Gathers every distinct color used across all objects into one 256-color
/// table and the matching color-to-index map. Slot 0 is MagicaVoxel's reserved
/// empty color, so real colors fill `1..=255`; a 256th distinct color and
/// beyond reuse the last slot.
fn synthesize_palette(state: &VoxMain) -> (MVoxPalette, HashMap<[u8; 4], u8>) {
    let mut colors = [MVoxColor::default(); 256];
    let mut color_index: HashMap<[u8; 4], u8> = HashMap::new();
    let mut next = 1usize;

    for (_, object) in state.iter_objects() {
        let Some((layer, palette, binding)) = object_color_ref(state, object) else {
            continue;
        };
        for voxel in object.iter_live() {
            let rgba = cell_color(state, object, voxel, layer, palette, binding);
            if color_index.contains_key(&rgba) {
                continue;
            }
            if next <= 255 {
                colors[next] = MVoxColor::new(rgba[0], rgba[1], rgba[2], rgba[3]);
                color_index.insert(rgba, next as u8);
                next += 1;
            } else {
                // The 256-color table is full; collapse the excess onto the
                // last slot rather than overwriting a color already in use.
                color_index.insert(rgba, 255);
            }
        }
    }

    (MVoxPalette { colors }, color_index)
}

/// Builds one model from an object: its grid size and one voxel per live cell
/// in ascending raster order, each taking its global palette index from the
/// object's color. Errors when the grid exceeds MagicaVoxel's
/// 256-voxel-per-axis limit, past which the byte voxel coordinates cannot
/// address it.
fn synthesize_model(
    state: &VoxMain,
    object: &VoxObject,
    color_index: &HashMap<[u8; 4], u8>,
) -> Result<MVoxModel> {
    let bounds = object.bounds();
    if bounds.x > 256 || bounds.y > 256 || bounds.z > 256 {
        return Err(Error::Invalid(format!(
            "object grid {}x{}x{} exceeds MagicaVoxel's limit of 256 voxels per axis",
            bounds.x, bounds.y, bounds.z
        )));
    }

    let color_ref = object_color_ref(state, object);
    let voxels = object
        .iter_live()
        .map(|voxel| {
            let position = object
                .voxel_position(voxel)
                .expect("a live voxel is within the grid");
            let index = color_ref
                .map(|(layer, palette, binding)| {
                    cell_color(state, object, voxel, layer, palette, binding)
                })
                .and_then(|rgba| color_index.get(&rgba).copied())
                .unwrap_or(0);
            MVoxVoxel {
                x: position.x as u8,
                y: position.y as u8,
                z: position.z as u8,
                color_index: index,
            }
        })
        .collect();

    Ok(MVoxModel {
        size: [bounds.x, bounds.y, bounds.z],
        voxels,
    })
}

/// Builds the MagicaVoxel scene graph mirroring the voxcore hierarchy. Every
/// node becomes an `nTRN` carrying its local translation over an `nGRP` of its
/// mapped child nodes and per-object `nTRN` -> `nSHP` placements; all roots
/// hang under one synthetic root `nTRN` -> `nGRP`, the single root MagicaVoxel
/// requires. An object no node places is emitted once at the origin so geometry
/// is never dropped, and an object placed by several nodes is emitted once per
/// placement.
fn synthesize_scene(state: &VoxMain) -> Vec<MVoxSceneNode> {
    let mut builder = SceneBuilder::default();
    let root_transform = builder.allocate();
    let root_group = builder.allocate();

    let mut roots: Vec<i32> = state
        .root_hierarchy_nodes()
        .iter()
        .map(|&node| builder.emit_node(state, node))
        .collect();
    for (id, object) in state.iter_objects() {
        if !builder.placed.contains(&id.to_u32()) {
            roots.push(builder.emit_object(id, object));
        }
    }

    builder.nodes.insert(
        root_transform,
        transform_node(root_transform, [0, 0, 0], root_group),
    );
    builder
        .nodes
        .insert(root_group, group_node(root_group, roots));
    builder.nodes.into_values().collect()
}

/// Allocates scene-node ids top-down and collects nodes by id, so the emitted
/// order lists parents before their children.
#[derive(Default)]
struct SceneBuilder {
    nodes: BTreeMap<i32, MVoxSceneNode>,
    placed: HashSet<u32>,
    next_id: i32,
}

impl SceneBuilder {
    /// Reserves the next scene-node id.
    fn allocate(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Emits the `nTRN` -> `nGRP` for a hierarchy node and its whole subtree,
    /// returning the transform node's id for a parent group to list.
    fn emit_node(&mut self, state: &VoxMain, node_id: U32Id<BVoxHierarchyNode>) -> i32 {
        let transform = self.allocate();
        let group = self.allocate();

        let (child_nodes, child_objects, translation) = {
            let node = state
                .hierarchy_node(node_id)
                .expect("a hierarchy id from the state resolves");
            (
                node.child_nodes.clone(),
                node.child_objects.clone(),
                translation_of(&node.transform.position),
            )
        };

        let mut children: Vec<i32> = child_nodes
            .iter()
            .map(|&child| self.emit_node(state, child))
            .collect();
        for &object_id in &child_objects {
            if let Some(object) = state.object(object_id) {
                children.push(self.emit_object(object_id, object));
            }
        }

        self.nodes
            .insert(transform, transform_node(transform, translation, group));
        self.nodes.insert(group, group_node(group, children));
        transform
    }

    /// Emits an `nTRN` -> `nSHP` placing one object, offsetting the transform
    /// by the model's pivot so MagicaVoxel centers it where voxcore's min
    /// corner sits.
    fn emit_object(&mut self, object_id: U32Id<BVoxObject>, object: &VoxObject) -> i32 {
        self.placed.insert(object_id.to_u32());
        let transform = self.allocate();
        let shape = self.allocate();

        let bounds = object.bounds();
        let pivot = [
            (bounds.x / 2) as i32,
            (bounds.y / 2) as i32,
            (bounds.z / 2) as i32,
        ];
        self.nodes
            .insert(transform, transform_node(transform, pivot, shape));
        self.nodes
            .insert(shape, shape_node(shape, object_id.to_u32()));
        transform
    }
}

/// An identity-rotation transform node at `translation` over its single child.
fn transform_node(id: i32, translation: [i32; 3], child: i32) -> MVoxSceneNode {
    MVoxSceneNode {
        id,
        attributes: MVoxNodeAttributes::default(),
        body: MVoxSceneNodeBody::Transform(MVoxTransformNode {
            child,
            layer: -1,
            frames: vec![MVoxFrame {
                translation,
                ..MVoxFrame::default()
            }],
        }),
    }
}

/// A group node over the given child transform nodes.
fn group_node(id: i32, children: Vec<i32>) -> MVoxSceneNode {
    MVoxSceneNode {
        id,
        attributes: MVoxNodeAttributes::default(),
        body: MVoxSceneNodeBody::Group(MVoxGroupNode { children }),
    }
}

/// A shape node drawing one model on its first frame.
fn shape_node(id: i32, model: u32) -> MVoxSceneNode {
    MVoxSceneNode {
        id,
        attributes: MVoxNodeAttributes::default(),
        body: MVoxSceneNodeBody::Shape(MVoxShapeNode {
            models: vec![MVoxShapeModel {
                model,
                frame_index: Some(0),
                extra: MVoxDict::default(),
            }],
        }),
    }
}

/// One scene-frame translation rounded from a node's local position.
fn translation_of(position: &TyVector3F64) -> [i32; 3] {
    position.round().to_i32().to_array()
}
