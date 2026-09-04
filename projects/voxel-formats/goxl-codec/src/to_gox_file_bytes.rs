use crate::{
    BLOCK_IMAGE_SIZE, ByteWriter, EncodePng, GoxlRgbaImage, push_bool, push_bytes, push_color,
    push_f32, push_i32, push_marker, push_mat4, push_str, push_vec3f, push_vec4f,
};
use goxl::{
    GoxlBlock, GoxlCamera, GoxlFile, GoxlImage, GoxlLayer, GoxlLight, GoxlMaterial, GoxlPreview,
    GoxlShape, GoxlVoxel,
};

/// Serializes a [`GoxlFile`] to the bytes of a Goxel `.gox` file through
/// `dependencies`, the inverse of
/// [`from_gox_file_bytes`](crate::from_gox_file_bytes).
///
/// Chunks are written in Goxel's order: `IMG `, the `PREV` preview, the shared
/// `BL16` blocks, `MATE` materials, `LAYR` layers, `CAMR` cameras, the `LIGH`
/// settings, then any preserved unknown chunks. Each dictionary emits its
/// modeled keys followed by the type's `extra` keys, so a file decoded by
/// [`from_gox_file_bytes`](crate::from_gox_file_bytes) re-encodes to an
/// equivalent file. Blocks and the preview are re-encoded as PNGs; the encoding
/// may differ byte-for-byte from Goxel's, but it is lossless for the pixels, so
/// the decoded model is unchanged.
///
/// The model is trusted, not validated: a block is written as exactly
/// [`GoxlBlock::SIZE`]`^3` voxels and the preview as `width * height` pixels,
/// padding with empty cells or truncating a mis-sized array.
/// [`validate_gox_file`](crate::validate_gox_file) checks these invariants.
pub fn to_gox_file_bytes<D: EncodePng>(dependencies: &D, file: &GoxlFile) -> Vec<u8> {
    let mut out = ByteWriter::new();
    out.write_bytes(b"GOX ");
    out.write_i32(file.version);

    write_image(&mut out, &file.image);
    if let Some(preview) = &file.preview {
        write_preview(dependencies, &mut out, preview);
    }
    for block in &file.blocks {
        write_block(dependencies, &mut out, block);
    }
    for material in &file.materials {
        write_material(&mut out, material);
    }
    for layer in &file.layers {
        write_layer(&mut out, layer);
    }
    for camera in &file.cameras {
        write_camera(&mut out, camera);
    }
    if let Some(light) = &file.light {
        write_light(&mut out, light);
    }
    for chunk in &file.unknown_chunks {
        out.write_chunk(&chunk.id, &chunk.data);
    }

    out.into_bytes()
}

/// Writes the `IMG ` image-metadata chunk. Goxel always writes it, even when its
/// dictionary is empty.
fn write_image(out: &mut ByteWriter, image: &GoxlImage) {
    let mut dict = Vec::new();
    if let Some(bounding_box) = image.bounding_box {
        push_mat4(&mut dict, "box", bounding_box);
    }
    dict.extend(image.extra.0.iter().cloned());
    write_dict_chunk(out, b"IMG ", &dict);
}

/// Writes the `PREV` preview chunk as a PNG. Only a preview with non-zero
/// dimensions whose pixel count is exactly `width * height` is encoded; a
/// zero-area, inconsistent, or implausibly large preview (one no real pixel
/// buffer could match) is skipped rather than encoded, so the writer never
/// allocates an over-large buffer or overflows. The count is taken in `u64` to
/// keep that comparison itself overflow-free.
/// [`validate_gox_file`](crate::validate_gox_file) rejects such a preview, so this
/// skip is not reached for a validated file.
fn write_preview<D: EncodePng>(dependencies: &D, out: &mut ByteWriter, preview: &GoxlPreview) {
    let count = preview.width as u64 * preview.height as u64;
    if preview.width == 0 || preview.height == 0 || preview.pixels.len() as u64 != count {
        return;
    }

    let png = dependencies.encode_png(&GoxlRgbaImage {
        width: preview.width,
        height: preview.height,
        pixels: preview.pixels.clone(),
    });
    out.write_chunk(b"PREV", &png);
}

/// Writes one `BL16` block chunk as a `64 x 64` PNG, one pixel per voxel.
fn write_block<D: EncodePng>(dependencies: &D, out: &mut ByteWriter, block: &GoxlBlock) {
    let count = (BLOCK_IMAGE_SIZE * BLOCK_IMAGE_SIZE) as usize;
    let pixels = (0..count)
        .map(|index| {
            let GoxlVoxel { r, g, b, a } = block.voxels.get(index).copied().unwrap_or_default();
            [r, g, b, a]
        })
        .collect();

    let png = dependencies.encode_png(&GoxlRgbaImage {
        width: BLOCK_IMAGE_SIZE,
        height: BLOCK_IMAGE_SIZE,
        pixels,
    });
    out.write_chunk(b"BL16", &png);
}

/// Writes one `MATE` material chunk.
fn write_material(out: &mut ByteWriter, material: &GoxlMaterial) {
    let mut dict = Vec::new();
    push_str(&mut dict, "name", &material.name);
    push_vec4f(&mut dict, "color", material.base_color);
    push_f32(&mut dict, "metallic", material.metallic);
    push_f32(&mut dict, "roughness", material.roughness);
    push_vec3f(&mut dict, "emission", material.emission);
    dict.extend(material.extra.0.iter().cloned());
    write_dict_chunk(out, b"MATE", &dict);
}

/// Writes one `LAYR` layer chunk: the placed-block list, then the dictionary.
fn write_layer(out: &mut ByteWriter, layer: &GoxlLayer) {
    let mut content = ByteWriter::new();
    content.write_len(layer.blocks.len());
    for block in &layer.blocks {
        content.write_i32(block.block_index);
        content.write_i32(block.position[0]);
        content.write_i32(block.position[1]);
        content.write_i32(block.position[2]);
        content.write_i32(0);
    }

    let mut dict = Vec::new();
    push_str(&mut dict, "name", &layer.name);
    push_mat4(&mut dict, "mat", layer.transform);
    push_i32(&mut dict, "id", layer.id);
    push_i32(&mut dict, "base_id", layer.base_id);
    push_i32(&mut dict, "material", layer.material);
    push_i32(&mut dict, "mode", layer.mode);
    if let Some(image_path) = &layer.image_path {
        push_str(&mut dict, "img-path", image_path);
    }
    if let Some(bounding_box) = layer.bounding_box {
        push_mat4(&mut dict, "box", bounding_box);
    }
    if let Some(shape) = layer.shape {
        push_bytes(&mut dict, "shape", shape_id(shape));
    }
    if let Some(color) = layer.color {
        push_color(&mut dict, "color", color);
    }
    push_bool(&mut dict, "visible", layer.visible);
    dict.extend(layer.extra.0.iter().cloned());

    content.write_dict(&dict);
    out.write_chunk(b"LAYR", &content.into_bytes());
}

/// Writes one `CAMR` camera chunk.
fn write_camera(out: &mut ByteWriter, camera: &GoxlCamera) {
    let mut dict = Vec::new();
    push_str(&mut dict, "name", &camera.name);
    push_f32(&mut dict, "dist", camera.distance);
    push_bool(&mut dict, "ortho", camera.orthographic);
    push_mat4(&mut dict, "mat", camera.transform);
    if camera.active {
        push_marker(&mut dict, "active");
    }
    dict.extend(camera.extra.0.iter().cloned());
    write_dict_chunk(out, b"CAMR", &dict);
}

/// Writes the `LIGH` light-settings chunk.
fn write_light(out: &mut ByteWriter, light: &GoxlLight) {
    let mut dict = Vec::new();
    push_f32(&mut dict, "pitch", light.pitch);
    push_f32(&mut dict, "yaw", light.yaw);
    push_f32(&mut dict, "intensity", light.intensity);
    push_bool(&mut dict, "fixed", light.fixed);
    push_f32(&mut dict, "ambient", light.ambient);
    push_f32(&mut dict, "shadow", light.shadow);
    dict.extend(light.extra.0.iter().cloned());
    write_dict_chunk(out, b"LIGH", &dict);
}

/// Writes a chunk whose whole content is a dictionary.
fn write_dict_chunk(out: &mut ByteWriter, id: &[u8; 4], dict: &[(String, Vec<u8>)]) {
    let mut content = ByteWriter::new();
    content.write_dict(dict);
    out.write_chunk(id, &content.into_bytes());
}

/// The on-disk name Goxel uses for a shape.
fn shape_id(shape: GoxlShape) -> &'static [u8] {
    match shape {
        GoxlShape::Sphere => b"sphere",
        GoxlShape::Cube => b"cube",
        GoxlShape::Cylinder => b"cylinder",
    }
}
