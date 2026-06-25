use crate::{Error, GoxelExtWrapper, GoxelLayer, Result, from_vox_value};
use branded_id::U32Id;
use goxl::{
    GoxlBlock, GoxlCamera, GoxlDict, GoxlFile, GoxlImage, GoxlLayer, GoxlLayerBlock, GoxlLight,
    GoxlMaterial, GoxlPreview, GoxlShape, GoxlUnknownChunk, GoxlVoxel,
};
use ty_math::TyVector3U32;
use voxcore::{
    BVoxAttribute, BVoxPaletteRef, BVoxVoxel, VoxObject, VoxPalette, VoxState, VoxValue,
};

/// Writes a [`VoxState`] back to a decoded Goxel [`GoxlFile`], the inverse of
/// [`from_goxl_file`](crate::from_goxl_file).
///
/// Requires the `goxel` ext the forward path writes; without it the file cannot
/// be rebuilt. Each object emits one `16 x 16 x 16` block, and the layers,
/// materials, cameras, light, preview, and image are taken from the ext. An
/// empty cell is written back as the transparent zero voxel.
///
/// Errors if the ext is missing.
pub fn to_goxl_file(state: &VoxState) -> Result<GoxlFile> {
    let ext = match state.ext() {
        Some(ext) => from_vox_value::<GoxelExtWrapper>(ext)?.goxel,
        None => {
            return Err(Error::invalid(
                "state has no goxel ext; cannot rebuild a Goxel file",
            ));
        }
    };

    // The forward path adds exactly one palette and references it from every
    // object; the colors live in its cells.
    let palette = state.iter_palettes().next().map(|(_, palette)| palette);

    let blocks = state
        .iter_objects()
        .map(|(_, object)| block_from_object(object, palette))
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

/// Rebuilds a `16 x 16 x 16` block from an object: each cell takes its color
/// from the voxel's palette sample, or the transparent zero voxel when empty.
fn block_from_object(object: &VoxObject, palette: Option<&VoxPalette>) -> GoxlBlock {
    let size = GoxlBlock::SIZE;
    let reference = object.iter_palette_refs().next().map(|(id, _)| id);
    let mut voxels = Vec::with_capacity((size * size * size) as usize);

    // Storage order is x fastest, then y, then z, matching the loop nesting.
    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let voxel = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .filter(|&id| object.is_live(id))
                    .and_then(|id| voxel_color(object, palette, reference, id))
                    .map(|[r, g, b, a]| GoxlVoxel { r, g, b, a })
                    .unwrap_or_default();
                voxels.push(voxel);
            }
        }
    }

    GoxlBlock { voxels }
}

/// The `[r, g, b, a]` color a live voxel samples from the shared palette, or
/// `None` if the reference, cell, or `rgba` attribute is missing.
fn voxel_color(
    object: &VoxObject,
    palette: Option<&VoxPalette>,
    reference: Option<U32Id<BVoxPaletteRef>>,
    voxel: U32Id<BVoxVoxel>,
) -> Option<[u8; 4]> {
    let palette = palette?;
    let reference = reference?;
    let cell = object.voxel_cell(voxel, reference)?;
    let rgba = attribute_id(palette, "rgba")?;
    Some(parse_rgba(palette.cell_value(cell, rgba)))
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

/// The procedural shape for an on-disk shape name, or `None` for an unrecognized
/// one.
fn shape_from_token(token: &str) -> Option<GoxlShape> {
    match token {
        "sphere" => Some(GoxlShape::Sphere),
        "cube" => Some(GoxlShape::Cube),
        "cylinder" => Some(GoxlShape::Cylinder),
        _ => None,
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

/// Parses a `#RRGGBBAA` color cell into `[r, g, b, a]`, defaulting to transparent
/// on a missing or malformed value.
fn parse_rgba(value: Option<&VoxValue>) -> [u8; 4] {
    let Some(VoxValue::Text(hex)) = value else {
        return [0, 0, 0, 0];
    };
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |index: usize| {
        hex.get(index * 2..index * 2 + 2)
            .and_then(|byte| u8::from_str_radix(byte, 16).ok())
            .unwrap_or(0)
    };
    [byte(0), byte(1), byte(2), byte(3)]
}
