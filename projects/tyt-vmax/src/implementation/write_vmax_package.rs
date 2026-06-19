use crate::{Error, Result, VoxelMaxExt, VoxelMaxNode, quat_rotate};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use tyt_injection::{
    lzfse_compress,
    serde_json::{self, Value},
    serialize_bplist, serialize_json_pretty, write_file,
};
use vmax::VMaxVoxel;
use vmax_codec::encode_object_data;
use voxj::{VoxjFile, VoxjHierarchyNode, VoxjObject};
use voxj_codec::{decode_object, from_voxj_or_voxjz_bytes};

/// Reconstructs a `.vmax` package directory at `output` from `.voxj`/`.voxjz`
/// bytes: rebuilds `scene.json` from the `voxel-max` ext plus the voxj hierarchy
/// and writes one `contents*.vmaxb` per object (voxels re-based by
/// `round(e_c + e_mi)`). Palette sidecars are written by the caller.
pub(crate) fn write_vmax_package(voxj_bytes: &[u8], output: &Path) -> Result<()> {
    let (file, ext) = from_voxj_or_voxjz_bytes(voxj_bytes)?;
    let voxel_max_value = ext.get("voxel-max").cloned().ok_or_else(|| {
        invalid("voxj document has no ext.voxel-max; cannot rebuild a .vmax package")
    })?;
    let voxel_max: VoxelMaxExt = serde_json::from_value(voxel_max_value).map_err(invalid)?;

    std::fs::create_dir_all(output)?;

    let mut objects: Vec<Value> = Vec::new();
    let mut groups: Vec<Value> = Vec::new();
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

        // Object node: rebuild and write the voxel payload, then the object entry.
        let object_index = node.child_objects[0];
        let object = &file.main.objects[object_index];
        let suffix = if object_index == 0 {
            String::new()
        } else {
            object_index.to_string()
        };
        let data = format!("contents{suffix}.vmaxb");

        let voxels = reconstruct_voxels(&file, object, &ext_node)?;
        let payload =
            lzfse_compress(&serialize_bplist(&encode_object_data(&voxels)).map_err(invalid)?);
        write_file(&output.join(&data), &payload)?;

        map.insert("n".to_owned(), Value::String(node.name.clone()));
        map.insert("data".to_owned(), Value::String(data));
        map.insert(
            "pal".to_owned(),
            Value::String(format!("palette{suffix}.png")),
        );
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

/// Decodes an object's voxels and re-bases them to absolute model space, reading
/// the color and material indices from the `rgba` and `metallic` sample channels.
fn reconstruct_voxels(
    file: &VoxjFile,
    object: &VoxjObject,
    ext_node: &VoxelMaxNode,
) -> Result<Vec<VMaxVoxel>> {
    let cell_counts: Vec<usize> = object
        .palette_refs
        .iter()
        .map(|&r| file.main.palettes.get(r).map_or(0, |p| p.data.len()))
        .collect();
    let data = decode_object(object, &cell_counts)?;

    let mut color = None;
    let mut material = None;
    for (channel, &r) in object.palette_refs.iter().enumerate() {
        let Some(palette) = file.main.palettes.get(r) else {
            continue;
        };
        if palette.attributes.iter().any(|a| a == "rgba") {
            color = Some(channel);
        } else if palette.attributes.iter().any(|a| a == "metallic") {
            material = Some(channel);
        }
    }

    let box_min = box_min(ext_node);
    Ok(data
        .positions
        .iter()
        .zip(&data.samples)
        .map(|(position, sample)| VMaxVoxel {
            x: position[0] as i32 + box_min[0],
            y: position[1] as i32 + box_min[1],
            z: position[2] as i32 + box_min[2],
            material: material.map_or(0, |c| sample[c] as u8),
            color: color.map_or(0, |c| sample[c] as u8),
        })
        .collect())
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

/// A `[f64; 3]` as a JSON array.
fn array3(values: [f64; 3]) -> Value {
    Value::Array(values.iter().map(|&v| v.into()).collect())
}

/// Wraps any error as invalid data.
fn invalid(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Error {
    IOError::new(ErrorKind::InvalidData, error).into()
}
