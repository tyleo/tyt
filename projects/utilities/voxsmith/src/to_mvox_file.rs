use crate::{
    Error, MagicaVoxelExt, MagicaVoxelExtWrapper, MagicaVoxelFrame, MagicaVoxelNodeBody, Result,
    from_vox_value,
};
use branded_id::U32Id;
use mvox::{
    MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer, MVoxMaterial,
    MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette, MVoxRenderObject, MVoxRotation,
    MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel, MVoxShapeNode, MVoxTransformNode,
    MVoxUnknownChunk, MVoxVoxel,
};
use voxcore::{BVoxAttribute, BVoxPaletteCell, VoxObject, VoxPalette, VoxState, VoxValue};

/// Writes a [`VoxState`] back to a decoded MagicaVoxel [`MVoxFile`], the inverse
/// of [`from_mvox_file`](crate::from_mvox_file).
///
/// Requires the `magica-voxel` ext the forward path writes; without it the file
/// cannot be rebuilt. Each object emits one model whose voxels are listed in
/// ascending raster order, which need not match their original stored order.
///
/// Errors if the ext is missing or its per-node entries do not line up with the
/// hierarchy.
pub fn to_mvox_file(state: &VoxState) -> Result<MVoxFile> {
    let ext = match state.ext() {
        Some(ext) => from_vox_value::<MagicaVoxelExtWrapper>(ext)?.magica_voxel,
        None => {
            return Err(Error::Invalid(
                "state has no magica-voxel ext; cannot rebuild a MagicaVoxel file".to_owned(),
            ));
        }
    };

    // The forward path adds exactly one palette and references it from every
    // object; the colors and material columns live in its cells.
    let palette = state.iter_palettes().next().map(|(_, palette)| palette);
    let file_palette = ext.palette_present.then(|| MVoxPalette {
        colors: colors_from_palette(palette),
    });

    let materials = build_materials(&ext, palette);
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

/// The 256 colors from a palette's `rgba` cells, padded with the transparent
/// empty color where the palette is absent or short.
fn colors_from_palette(palette: Option<&VoxPalette>) -> [MVoxColor; 256] {
    let mut colors = [MVoxColor::default(); 256];
    let Some(palette) = palette else {
        return colors;
    };
    let Some(rgba) = attribute_id(palette, "rgba") else {
        return colors;
    };
    for (index, cell) in palette.iter_cells().take(colors.len()).enumerate() {
        colors[index] = parse_rgba(palette.cell_value(cell, rgba));
    }
    colors
}

/// Rebuilds the materials from the ext, reading each one's type and scalar fields
/// from the palette cell its id names.
fn build_materials(ext: &MagicaVoxelExt, palette: Option<&VoxPalette>) -> Vec<MVoxMaterial> {
    ext.materials
        .iter()
        .map(|material| {
            let cell = U32Id::<BVoxPaletteCell>::from_u32(material.id as u32);
            let read = |name: &str| palette.and_then(|palette| cell_value(palette, cell, name));
            MVoxMaterial {
                id: material.id,
                material_type: match read("type") {
                    Some(VoxValue::Text(token)) => Some(material_type_from_token(token)),
                    _ => None,
                },
                weight: number(read("weight")),
                rough: number(read("rough")),
                spec: number(read("spec")),
                ior: number(read("ior")),
                att: number(read("att")),
                flux: number(read("flux")),
                extra: MVoxDict(material.extra.clone()),
            }
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

/// An `f32` from a [`VoxValue::Number`] cell, or `None` for any other value.
fn number(value: Option<&VoxValue>) -> Option<f32> {
    match value {
        Some(VoxValue::Number(number)) => Some(*number as f32),
        _ => None,
    }
}

/// Builds a model from an object: its size from the grid bounds and one voxel per
/// live cell, in ascending raster order, each taking its color index from the
/// sample of the object's first palette reference.
fn model_from_object(object: &VoxObject) -> MVoxModel {
    let bounds = object.bounds();
    let reference = object.iter_palette_refs().next().map(|(id, _)| id);

    let voxels = object
        .iter_live()
        .map(|voxel_id| {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            let color_index = reference
                .and_then(|reference| object.voxel_cell(voxel_id, reference))
                .map_or(0, |cell| cell.to_u32() as u8);
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
/// draws one model on several frames or any other repeated reference round-trips.
/// Errors if the ext node count does not match the hierarchy.
fn build_scene_nodes(state: &VoxState, ext: &MagicaVoxelExt) -> Result<Vec<MVoxSceneNode>> {
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

/// The id of the attribute named `name`, or `None` if the palette has no such
/// attribute.
fn attribute_id(palette: &VoxPalette, name: &str) -> Option<U32Id<BVoxAttribute>> {
    palette
        .iter_attributes()
        .find(|(_, attribute)| *attribute == name)
        .map(|(id, _)| id)
}

/// The value of `cell` for the attribute named `name`, or `None`.
fn cell_value<'a>(
    palette: &'a VoxPalette,
    cell: U32Id<BVoxPaletteCell>,
    name: &str,
) -> Option<&'a VoxValue> {
    let attribute = attribute_id(palette, name)?;
    palette.cell_value(cell, attribute)
}

/// Parses a `#RRGGBBAA` color cell into a color, defaulting to transparent on a
/// missing or malformed value.
fn parse_rgba(value: Option<&VoxValue>) -> MVoxColor {
    let Some(VoxValue::Text(hex)) = value else {
        return MVoxColor::default();
    };
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |index: usize| {
        hex.get(index * 2..index * 2 + 2)
            .and_then(|byte| u8::from_str_radix(byte, 16).ok())
            .unwrap_or(0)
    };
    MVoxColor::new(byte(0), byte(1), byte(2), byte(3))
}
