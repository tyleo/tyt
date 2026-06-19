use crate::{Error, Result, VoxelMaxExt, VoxelMaxNode, quat_rotate};
use std::{
    collections::HashMap,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use tyt_injection::{
    encode_png_rgba, lzfse_compress,
    serde_json::{self, Value},
    serialize_bplist, serialize_json_pretty, write_file,
};
use vmax::VMaxVoxel;
use vmax_codec::{VXMaterialPaletteSerde, VXMaterialSerde, encode_object_data};
use voxj::{AttrValue, VoxjFile, VoxjHierarchyNode, VoxjObject, VoxjPalette};
use voxj_codec::{decode_object, from_voxj_or_voxjz_bytes};

/// Reconstructs a `.vmax` package directory at `output` from `.voxj`/`.voxjz`
/// bytes: rebuilds `scene.json` from the `voxel-max` ext plus the voxj
/// hierarchy, and writes one `contents*.vmaxb` per object (voxels re-based by
/// `round(e_c + e_mi)`) plus the color `palette*.png` / material
/// `palette*.settings.vmaxpsb` sidecars (one set per distinct palette).
pub(crate) fn write_vmax_package(voxj_bytes: &[u8], output: &Path) -> Result<()> {
    let (file, ext) = from_voxj_or_voxjz_bytes(voxj_bytes)?;
    let voxel_max_value = ext.get("voxel-max").cloned().ok_or_else(|| {
        invalid("voxj document has no ext.voxel-max; cannot rebuild a .vmax package")
    })?;
    let voxel_max: VoxelMaxExt = serde_json::from_value(voxel_max_value).map_err(invalid)?;

    std::fs::create_dir_all(output)?;

    let mut objects: Vec<Value> = Vec::new();
    let mut groups: Vec<Value> = Vec::new();
    // Color palette index -> the `pal` filename written for it (shared by every
    // object that references the same palette).
    let mut palette_files: HashMap<usize, String> = HashMap::new();

    for (index, node) in file.main.hierarchy_nodes.iter().enumerate() {
        let ext_node = voxel_max
            .hierarchy_nodes
            .get(index)
            .cloned()
            .unwrap_or_default();
        let mut entry = serde_json::to_value(&ext_node).map_err(invalid)?;
        let map = entry.as_object_mut().expect("node serializes to an object");
        map.insert("t_s".to_owned(), array3(node.transform.scale));

        if node.child_objects.is_empty() {
            // Group node: name and position carry over from the voxj node directly.
            map.insert("name".to_owned(), Value::String(node.name.clone()));
            map.insert("t_p".to_owned(), array3(node.transform.position));
            groups.push(entry);
            continue;
        }

        let object_index = node.child_objects[0];
        let object = &file.main.objects[object_index];
        let (color, material) = object_palettes(&file, object);
        let suffix = suffix(object_index);

        let voxels = reconstruct_voxels(
            &file,
            object,
            color.map(|(channel, _)| channel),
            material.map(|(channel, _)| channel),
            &ext_node,
        )?;
        let data = format!("contents{suffix}.vmaxb");
        let payload =
            lzfse_compress(&serialize_bplist(&encode_object_data(&voxels)).map_err(invalid)?);
        write_file(&output.join(&data), &payload)?;

        let pal = write_palette(
            &file,
            &voxel_max,
            color.map(|(_, index)| index),
            material.map(|(_, index)| index),
            &mut palette_files,
            output,
        )?;

        map.insert("n".to_owned(), Value::String(node.name.clone()));
        map.insert("data".to_owned(), Value::String(data));
        map.insert("pal".to_owned(), Value::String(pal));
        map.insert(
            "hist".to_owned(),
            Value::String(format!("history{suffix}.vmaxhb")),
        );
        map.insert("t_p".to_owned(), array3(unbake_position(node, &ext_node)));
        objects.push(entry);
    }

    // scene.json = the verbatim renderer/camera/version block plus the rebuilt tree.
    let mut scene = serde_json::to_value(&voxel_max.scene).map_err(invalid)?;
    let scene_map = scene
        .as_object_mut()
        .expect("scene serializes to an object");
    scene_map.insert("objects".to_owned(), Value::Array(objects));
    if !groups.is_empty() {
        scene_map.insert("groups".to_owned(), Value::Array(groups));
    }
    write_file(&output.join("scene.json"), &serialize_json_pretty(&scene)?)?;
    Ok(())
}

/// `(color, material)` palette references for an object — each a `(sample
/// channel, palette index)` — identified by the `rgba` / `metallic` attributes.
fn object_palettes(
    file: &VoxjFile,
    object: &VoxjObject,
) -> (Option<(usize, usize)>, Option<(usize, usize)>) {
    let mut color = None;
    let mut material = None;
    for (channel, &index) in object.palette_refs.iter().enumerate() {
        let Some(palette) = file.main.palettes.get(index) else {
            continue;
        };
        if palette.attributes.iter().any(|a| a == "rgba") {
            color = Some((channel, index));
        } else if palette.attributes.iter().any(|a| a == "metallic") {
            material = Some((channel, index));
        }
    }
    (color, material)
}

/// Decodes an object's voxels and re-bases them to absolute model space, reading
/// the color and material indices from their sample channels.
fn reconstruct_voxels(
    file: &VoxjFile,
    object: &VoxjObject,
    color_channel: Option<usize>,
    material_channel: Option<usize>,
    ext_node: &VoxelMaxNode,
) -> Result<Vec<VMaxVoxel>> {
    let cell_counts: Vec<usize> = object
        .palette_refs
        .iter()
        .map(|&r| file.main.palettes.get(r).map_or(0, |p| p.data.len()))
        .collect();
    let data = decode_object(object, &cell_counts)?;
    let box_min = box_min(ext_node);
    Ok(data
        .positions
        .iter()
        .zip(&data.samples)
        .map(|(position, sample)| VMaxVoxel {
            x: position[0] as i32 + box_min[0],
            y: position[1] as i32 + box_min[1],
            z: position[2] as i32 + box_min[2],
            material: material_channel.map_or(0, |c| sample[c] as u8),
            color: color_channel.map_or(0, |c| sample[c] as u8),
        })
        .collect())
}

/// Writes the color `.png` and material `.vmaxpsb` for a color palette the first
/// time it is seen and returns the `pal` filename to share across objects.
fn write_palette(
    file: &VoxjFile,
    voxel_max: &VoxelMaxExt,
    color: Option<usize>,
    material: Option<usize>,
    palette_files: &mut HashMap<usize, String>,
    output: &Path,
) -> Result<String> {
    let Some(color_index) = color else {
        return Ok("palette.png".to_owned());
    };
    if let Some(name) = palette_files.get(&color_index) {
        return Ok(name.clone());
    }
    let stem = match palette_files.len() {
        0 => String::new(),
        n => n.to_string(),
    };
    let pal = format!("palette{stem}.png");
    write_color_palette(&file.main.palettes[color_index], &output.join(&pal))?;
    if let Some(material_index) = material {
        let name = voxel_max
            .palettes
            .get(material_index)
            .and_then(|palette| palette.clone())
            .map(|palette| palette.name)
            .unwrap_or_default();
        let sidecar = output.join(format!("palette{stem}.settings.vmaxpsb"));
        write_material_palette(&file.main.palettes[material_index], name, &sidecar)?;
    }
    palette_files.insert(color_index, pal.clone());
    Ok(pal)
}

/// Writes a color palette's `rgba` cells as a `width × 1` RGBA PNG.
fn write_color_palette(palette: &VoxjPalette, path: &Path) -> Result<()> {
    let mut rgba = Vec::with_capacity(palette.data.len() * 4);
    for cell in &palette.data {
        rgba.extend_from_slice(&parse_rgba(cell));
    }
    write_file(path, &encode_png_rgba(&rgba, palette.data.len() as u32, 1)?)?;
    Ok(())
}

/// Writes a material palette's slots and display name as a `.vmaxpsb` bplist.
fn write_material_palette(palette: &VoxjPalette, name: String, path: &Path) -> Result<()> {
    let materials = palette
        .data
        .iter()
        .map(|cell| parse_material(cell))
        .collect();
    let psb = VXMaterialPaletteSerde { name, materials };
    write_file(path, &serialize_bplist(&psb).map_err(invalid)?)?;
    Ok(())
}

/// Parses a `#RRGGBBAA` color cell into RGBA bytes.
fn parse_rgba(cell: &[AttrValue]) -> [u8; 4] {
    let Some(AttrValue::Text(hex)) = cell.first() else {
        return [0, 0, 0, 0];
    };
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |i: usize| {
        hex.get(i * 2..i * 2 + 2)
            .and_then(|h| u8::from_str_radix(h, 16).ok())
            .unwrap_or(0)
    };
    [byte(0), byte(1), byte(2), byte(3)]
}

/// Parses a `[metallic, roughness, emissive, shadows]` material cell.
fn parse_material(cell: &[AttrValue]) -> VXMaterialSerde {
    let number = |i: usize| match cell.get(i) {
        Some(AttrValue::Number(n)) => *n,
        _ => 0.0,
    };
    VXMaterialSerde {
        mc: number(0),
        rc: number(1),
        sic: number(2),
        sh: matches!(cell.get(3), Some(AttrValue::Bool(true))),
    }
}

/// The absolute model-space origin `round(e_c + e_mi)` the forward path re-based
/// against (`[0, 0, 0]` when the node has no authored bounds).
fn box_min(ext_node: &VoxelMaxNode) -> [i32; 3] {
    let center = ext_node.center.unwrap_or_default();
    let min = ext_node.bounds_min.unwrap_or_default();
    [
        (center[0] + min[0]).round() as i32,
        (center[1] + min[1]).round() as i32,
        (center[2] + min[2]).round() as i32,
    ]
}

/// Recovers the object's `t_p` by undoing the forward placement
/// `position = t_p + R·((box_min − e_c) ⊙ t_s)`.
fn unbake_position(node: &VoxjHierarchyNode, ext_node: &VoxelMaxNode) -> [f64; 3] {
    let center = ext_node.center.unwrap_or_default();
    let scale = node.transform.scale;
    let min = box_min(ext_node);
    let offset = [
        (min[0] as f64 - center[0]) * scale[0],
        (min[1] as f64 - center[1]) * scale[1],
        (min[2] as f64 - center[2]) * scale[2],
    ];
    let rotated = quat_rotate(node.transform.rotation, offset);
    [
        node.transform.position[0] - rotated[0],
        node.transform.position[1] - rotated[1],
        node.transform.position[2] - rotated[2],
    ]
}

/// The filename suffix for object `index`: empty for the first, then the index.
fn suffix(index: usize) -> String {
    if index == 0 {
        String::new()
    } else {
        index.to_string()
    }
}

/// A `[f64; 3]` as a JSON array.
fn array3(values: [f64; 3]) -> Value {
    Value::Array(values.iter().map(|&v| v.into()).collect())
}

/// Wraps any error as invalid data.
fn invalid(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Error {
    IOError::new(ErrorKind::InvalidData, error).into()
}
