use crate::{ByteWriter, push_bool, push_f32, push_i32, push_str, push_vec3f, push_vec3i};
use mvox::{
    MVoxCamera, MVoxColor, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer, MVoxMaterial,
    MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette, MVoxRotation, MVoxSceneNode,
    MVoxSceneNodeBody, MVoxShapeNode, MVoxTransformNode,
};

/// Serializes an [`MVoxFile`] to the bytes of a MagicaVoxel `.vox` file, the
/// inverse of [`from_mvox_file_bytes`](crate::from_mvox_file_bytes).
///
/// Chunks are written in a fixed, MagicaVoxel-readable order. The dictionary
/// chunks emit each modeled key only when it is set, followed by the type's
/// `extra` keys, so a file decoded by
/// [`from_mvox_file_bytes`](crate::from_mvox_file_bytes) re-encodes to an
/// equivalent file. A few details are normalized rather than reproduced
/// byte-for-byte, none of which change the decoded model: the legacy `PACK`
/// chunk is not written (the model count is the number of SIZE/XYZI pairs), a
/// palette's reserved color index `0` is not stored, and an empty `NOTE` chunk
/// (no color names) is dropped.
pub fn to_mvox_file_bytes(file: &MVoxFile) -> Vec<u8> {
    let mut children = ByteWriter::new();

    for model in &file.models {
        write_size(&mut children, model);
        write_xyzi(&mut children, model);
    }
    for node in &file.scene_nodes {
        write_scene_node(&mut children, node);
    }
    for layer in &file.layers {
        write_layer(&mut children, layer);
    }
    if let Some(palette) = &file.palette {
        write_rgba(&mut children, palette);
    }
    for material in &file.materials {
        write_material(&mut children, material);
    }
    for render_object in &file.render_objects {
        let mut content = ByteWriter::new();
        content.write_dict(&render_object.attributes.0);
        children.write_chunk(b"rOBJ", &content.into_bytes(), &[]);
    }
    for camera in &file.cameras {
        write_camera(&mut children, camera);
    }
    if !file.palette_notes.is_empty() {
        write_note(&mut children, &file.palette_notes);
    }
    if let Some(index_map) = &file.index_map {
        children.write_chunk(b"IMAP", index_map, &[]);
    }
    for chunk in &file.unknown_chunks {
        children.write_chunk(&chunk.id, &chunk.content, &chunk.children);
    }

    let children = children.into_bytes();
    let mut out = ByteWriter::new();
    out.write_bytes(b"VOX ");
    out.write_u32(file.version);
    out.write_chunk(b"MAIN", &[], &children);
    out.into_bytes()
}

/// Writes a model's `SIZE` chunk.
fn write_size(out: &mut ByteWriter, model: &MVoxModel) {
    // SIZE content is three `u32`s.
    let mut content = ByteWriter::with_capacity(3 * 4);
    content.write_u32(model.size[0]);
    content.write_u32(model.size[1]);
    content.write_u32(model.size[2]);
    out.write_chunk(b"SIZE", &content.into_bytes(), &[]);
}

/// Writes a model's `XYZI` chunk.
fn write_xyzi(out: &mut ByteWriter, model: &MVoxModel) {
    // XYZI content is a `u32` count then four bytes per voxel; size it exactly so
    // a large model does not reallocate as its voxels are appended.
    let mut content = ByteWriter::with_capacity(4 + model.voxels.len() * 4);
    content.write_len(model.voxels.len());
    for voxel in &model.voxels {
        content.write_u8(voxel.x);
        content.write_u8(voxel.y);
        content.write_u8(voxel.z);
        content.write_u8(voxel.color_index);
    }
    out.write_chunk(b"XYZI", &content.into_bytes(), &[]);
}

/// Writes the `RGBA` palette, re-applying the on-disk index shift: palette index
/// `j + 1` becomes file color `j`, and the reserved index `0` is written as the
/// unused final entry so the chunk carries the full 256 colors.
fn write_rgba(out: &mut ByteWriter, palette: &MVoxPalette) {
    // RGBA content is exactly 256 colors of four bytes each.
    let mut content = ByteWriter::with_capacity(256 * 4);
    for index in 1..=255usize {
        write_color(&mut content, palette.colors[index]);
    }
    write_color(&mut content, MVoxColor::default());
    out.write_chunk(b"RGBA", &content.into_bytes(), &[]);
}

/// Writes one RGBA color.
fn write_color(content: &mut ByteWriter, color: MVoxColor) {
    content.write_u8(color.r);
    content.write_u8(color.g);
    content.write_u8(color.b);
    content.write_u8(color.a);
}

/// Writes one scene node by kind.
fn write_scene_node(out: &mut ByteWriter, node: &MVoxSceneNode) {
    match &node.body {
        MVoxSceneNodeBody::Transform(transform) => write_transform_node(out, node, transform),
        MVoxSceneNodeBody::Group(group) => write_group_node(out, node, group),
        MVoxSceneNodeBody::Shape(shape) => write_shape_node(out, node, shape),
    }
}

/// The node-attributes `DICT` pairs: the modeled keys, then `extra`.
fn node_attributes_dict(attributes: &MVoxNodeAttributes) -> Vec<(String, String)> {
    let mut dict = Vec::new();
    push_str(&mut dict, "_name", attributes.name.as_deref());
    push_bool(&mut dict, "_hidden", attributes.hidden);
    dict.extend(attributes.extra.0.iter().cloned());
    dict
}

/// Writes an `nTRN` transform node.
fn write_transform_node(out: &mut ByteWriter, node: &MVoxSceneNode, transform: &MVoxTransformNode) {
    let mut content = ByteWriter::new();
    content.write_i32(node.id);
    content.write_dict(&node_attributes_dict(&node.attributes));
    content.write_i32(transform.child);
    content.write_i32(-1);
    content.write_i32(transform.layer);
    content.write_len(transform.frames.len());
    for frame in &transform.frames {
        content.write_dict(&frame_dict(frame));
    }
    out.write_chunk(b"nTRN", &content.into_bytes(), &[]);
}

/// The frame `DICT` pairs, emitting `_r` and `_t` only when they differ from
/// their defaults, then `_f`, then `extra`.
fn frame_dict(frame: &MVoxFrame) -> Vec<(String, String)> {
    let mut dict = Vec::new();
    if frame.rotation != MVoxRotation::IDENTITY {
        dict.push(("_r".to_owned(), frame.rotation.0.to_string()));
    }
    if frame.translation != [0, 0, 0] {
        push_vec3i(&mut dict, "_t", frame.translation);
    }
    if let Some(frame_index) = frame.frame_index {
        dict.push(("_f".to_owned(), frame_index.to_string()));
    }
    dict.extend(frame.extra.0.iter().cloned());
    dict
}

/// Writes an `nGRP` group node.
fn write_group_node(out: &mut ByteWriter, node: &MVoxSceneNode, group: &MVoxGroupNode) {
    let mut content = ByteWriter::new();
    content.write_i32(node.id);
    content.write_dict(&node_attributes_dict(&node.attributes));
    content.write_len(group.children.len());
    for &child in &group.children {
        content.write_i32(child);
    }
    out.write_chunk(b"nGRP", &content.into_bytes(), &[]);
}

/// Writes an `nSHP` shape node.
fn write_shape_node(out: &mut ByteWriter, node: &MVoxSceneNode, shape: &MVoxShapeNode) {
    let mut content = ByteWriter::new();
    content.write_i32(node.id);
    content.write_dict(&node_attributes_dict(&node.attributes));
    content.write_len(shape.models.len());
    for model in &shape.models {
        content.write_u32(model.model);
        let mut dict = Vec::new();
        if let Some(frame_index) = model.frame_index {
            dict.push(("_f".to_owned(), frame_index.to_string()));
        }
        dict.extend(model.extra.0.iter().cloned());
        content.write_dict(&dict);
    }
    out.write_chunk(b"nSHP", &content.into_bytes(), &[]);
}

/// Writes a `MATL` material.
fn write_material(out: &mut ByteWriter, material: &MVoxMaterial) {
    let mut content = ByteWriter::new();
    content.write_i32(material.id);
    let mut dict = Vec::new();
    if let Some(material_type) = &material.material_type {
        dict.push(("_type".to_owned(), material_type_token(material_type)));
    }
    push_f32(&mut dict, "_weight", material.weight);
    push_f32(&mut dict, "_rough", material.rough);
    push_f32(&mut dict, "_spec", material.spec);
    push_f32(&mut dict, "_ior", material.ior);
    push_f32(&mut dict, "_att", material.att);
    push_f32(&mut dict, "_flux", material.flux);
    dict.extend(material.extra.0.iter().cloned());
    content.write_dict(&dict);
    out.write_chunk(b"MATL", &content.into_bytes(), &[]);
}

/// The `_type` token for a material type, preserving unrecognized values.
fn material_type_token(material_type: &MVoxMaterialType) -> String {
    match material_type {
        MVoxMaterialType::Diffuse => "_diffuse".to_owned(),
        MVoxMaterialType::Metal => "_metal".to_owned(),
        MVoxMaterialType::Glass => "_glass".to_owned(),
        MVoxMaterialType::Emit => "_emit".to_owned(),
        MVoxMaterialType::Other(value) => value.clone(),
    }
}

/// Writes a `LAYR` layer.
fn write_layer(out: &mut ByteWriter, layer: &MVoxLayer) {
    let mut content = ByteWriter::new();
    content.write_i32(layer.id);
    let mut dict = Vec::new();
    push_str(&mut dict, "_name", layer.name.as_deref());
    push_bool(&mut dict, "_hidden", layer.hidden);
    dict.extend(layer.extra.0.iter().cloned());
    content.write_dict(&dict);
    content.write_i32(-1);
    out.write_chunk(b"LAYR", &content.into_bytes(), &[]);
}

/// Writes an `rCAM` render camera.
fn write_camera(out: &mut ByteWriter, camera: &MVoxCamera) {
    let mut content = ByteWriter::new();
    content.write_i32(camera.id);
    let mut dict = Vec::new();
    push_str(&mut dict, "_mode", camera.mode.as_deref());
    push_vec3f(&mut dict, "_focus", camera.focus);
    push_vec3f(&mut dict, "_angle", camera.angle);
    push_i32(&mut dict, "_radius", camera.radius);
    push_f32(&mut dict, "_frustum", camera.frustum);
    push_i32(&mut dict, "_fov", camera.fov);
    dict.extend(camera.extra.0.iter().cloned());
    content.write_dict(&dict);
    out.write_chunk(b"rCAM", &content.into_bytes(), &[]);
}

/// Writes a `NOTE` chunk's palette color names.
fn write_note(out: &mut ByteWriter, notes: &[String]) {
    let mut content = ByteWriter::new();
    content.write_len(notes.len());
    for name in notes {
        content.write_string(name);
    }
    out.write_chunk(b"NOTE", &content.into_bytes(), &[]);
}
