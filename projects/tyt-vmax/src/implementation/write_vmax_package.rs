use crate::{
    ColorFormat, Error, Result, VoxelMaxExt, VoxelMaxNode, object_data_from_state, quat_rotate,
};
use std::{
    collections::HashMap,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use tyt_injection::{
    serde_json::{self, Value},
    serialize_json_pretty, write_file,
};
use vmax::{
    VMaxMaterial, VMaxMaterialDispersion, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile,
};
use vmax_codec::{
    Voxel, encode_object_data, encode_snapshots, to_palette_png_bytes, to_vmaxb_bytes,
    to_vmaxpsb_bytes,
};
use voxj::{VoxjFile, VoxjHierarchyNode, VoxjObject, VoxjPalette, VoxjValue};
use voxj_codec::{decode_object, from_voxj_or_voxjz_bytes, palette_cell_counts};

/// Reconstructs a `.vmax` package directory at `output` from `.voxj`/`.voxjz`
/// bytes: rebuilds `scene.json` from the `voxel-max` ext plus the voxj
/// hierarchy, and writes one `contents*.vmaxb` per object (voxels re-based by
/// `round(e_c + e_mi)`) plus the material `palette*.settings.vmaxpsb` sidecar
/// (one set per distinct palette). `color_format` selects where each palette's
/// colors live: a 256x1 `palette*.png` image ([`ColorFormat::Png`]), the
/// material sidecar's `colors` table ([`ColorFormat::Plist`]), or both
/// ([`ColorFormat::All`]). The `pal` references are written in every case.
pub(crate) fn write_vmax_package(
    voxj_bytes: &[u8],
    output: &Path,
    color_format: ColorFormat,
) -> Result<()> {
    let file = from_voxj_or_voxjz_bytes(voxj_bytes)?;
    let missing = || invalid("voxj document has no ext.voxel-max; cannot rebuild a .vmax package");
    let ext = serde_json::to_value(file.main.ext.as_ref().ok_or_else(missing)?).map_err(invalid)?;
    let voxel_max_value = ext.get("voxel-max").cloned().ok_or_else(missing)?;
    let voxel_max: VoxelMaxExt = serde_json::from_value(voxel_max_value).map_err(invalid)?;

    std::fs::create_dir_all(output)?;

    let mut objects: Vec<Value> = Vec::new();
    let mut groups: Vec<Value> = Vec::new();
    // Color palette index -> the `pal` filename written for it (shared by every
    // object that references the same palette).
    let mut palette_files: HashMap<usize, String> = HashMap::new();
    // Voxj object index -> its `data` filename, shared by every node that places
    // the same object (an instance).
    let mut contents_files: HashMap<usize, String> = HashMap::new();

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

        // Instances share one `contents*.vmaxb`: encode and write it once.
        let data = match contents_files.get(&object_index) {
            Some(data) => data.clone(),
            None => {
                let voxels = reconstruct_voxels(
                    &file,
                    object,
                    color.map(|(channel, _)| channel),
                    material.map(|(channel, _)| channel),
                    &ext_node,
                )?;
                let data = format!("contents{suffix}.vmaxb");
                // Restore the object's preserved editor state (tools/brush/cam,
                // content uuid/version) around the rebuilt geometry; fall back to
                // a minimal payload when the voxj document carries no Voxel Max
                // object state.
                let object_data = match voxel_max
                    .object_states
                    .get(object_index)
                    .and_then(|s| s.clone())
                {
                    Some(state) => object_data_from_state(state, encode_snapshots(&voxels)),
                    None => encode_object_data(&voxels, &ext_node.id),
                };
                let payload = to_vmaxb_bytes(&object_data)?;
                write_file(&output.join(&data), &payload)?;
                contents_files.insert(object_index, data.clone());
                data
            }
        };

        let pal = write_palette(
            &file,
            &voxel_max,
            color.map(|(_, index)| index),
            material.map(|(_, index)| index),
            &mut palette_files,
            output,
            color_format,
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

/// A `(sample channel, palette index)` reference into an object's palettes.
type PaletteRef = (usize, usize);

/// `(color, material)` palette references for an object — each a [`PaletteRef`]
/// — identified by the `rgba` / `metallic` attributes.
fn object_palettes(
    file: &VoxjFile,
    object: &VoxjObject,
) -> (Option<PaletteRef>, Option<PaletteRef>) {
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
) -> Result<Vec<Voxel>> {
    let cell_counts = palette_cell_counts(object, &file.main.palettes);
    let data = decode_object(object, &cell_counts)?;
    let box_min = box_min(ext_node);
    Ok(data
        .positions
        .iter()
        .zip(&data.samples)
        .map(|(position, sample)| Voxel {
            x: position[0] as i32 + box_min[0],
            y: position[1] as i32 + box_min[1],
            z: position[2] as i32 + box_min[2],
            material: material_channel.map_or(0, |c| sample[c] as u8),
            color: color_channel.map_or(0, |c| sample[c] as u8),
        })
        .collect())
}

/// Usable colors in a Voxel Max palette: indices 0..254. Index 255 is a
/// reserved transparent terminator that voxels never reference, so the color
/// table holds exactly this many entries. The `palette*.png` pads to 256x1 by
/// appending the terminator; the material sidecar's `colors` table stores these
/// 255 verbatim (Voxel Max ignores any 256th entry).
const PALETTE_COLORS: usize = 255;

/// Writes a palette's color image and/or material `.vmaxpsb` the first time the
/// color palette is seen and returns the `pal` filename to share across objects.
/// `color_format` selects whether the colors land in the `palette*.png` image or
/// the material sidecar's `colors` table; the returned `pal` reference names the
/// image either way.
fn write_palette(
    file: &VoxjFile,
    voxel_max: &VoxelMaxExt,
    color: Option<usize>,
    material: Option<usize>,
    palette_files: &mut HashMap<usize, String>,
    output: &Path,
    color_format: ColorFormat,
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

    // The canonical color table is PALETTE_COLORS entries; the png and the
    // sidecar's `colors` key are two interchangeable homes for it.
    let colors = palette_colors(&file.main.palettes[color_index]);
    if matches!(color_format, ColorFormat::Png | ColorFormat::All) {
        write_color_palette(&colors, &output.join(&pal))?;
    }
    if let Some(material_index) = material {
        let name = voxel_max
            .palettes
            .get(material_index)
            .and_then(|palette| palette.clone())
            .map(|palette| palette.name)
            .unwrap_or_default();
        let sidecar = output.join(format!("palette{stem}.settings.vmaxpsb"));
        // In png mode the colors live in the image, so the sidecar omits its
        // `colors` table (matching a Voxel Max package that ships a png); plist
        // and all modes carry them in the sidecar.
        let psb_colors = match color_format {
            ColorFormat::Png => Vec::new(),
            ColorFormat::Plist | ColorFormat::All => colors,
        };
        write_material_palette(
            &file.main.palettes[material_index],
            name,
            psb_colors,
            &sidecar,
        )?;
    }
    palette_files.insert(color_index, pal.clone());
    Ok(pal)
}

/// Writes a [`PALETTE_COLORS`]-entry color table as the 256x1 RGBA `palette*.png`
/// Voxel Max expects, appending the trailing transparent terminator at index
/// 255.
fn write_color_palette(colors: &[u8], path: &Path) -> Result<()> {
    let mut pixels = colors.to_vec();
    pixels.resize((PALETTE_COLORS + 1) * 4, 0);
    let cells: Vec<[u8; 4]> = pixels
        .chunks_exact(4)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect();
    write_file(path, &to_palette_png_bytes(&VMaxPalettePngFile(cells))?)?;
    Ok(())
}

/// A color palette's `rgba` cells as exactly [`PALETTE_COLORS`] RGBA entries,
/// padded with transparent cells or truncated so the count is fixed.
fn palette_colors(palette: &VoxjPalette) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(PALETTE_COLORS * 4);
    for cell in palette.data.iter().take(PALETTE_COLORS) {
        rgba.extend_from_slice(&parse_rgba(cell));
    }
    rgba.resize(PALETTE_COLORS * 4, 0);
    rgba
}

/// Writes a material palette as a `.vmaxpsb` bplist: the per-slot materials
/// (with `mi`), the display name, the `colors` bytes, and the editor-state keys
/// Voxel Max expects (mostly defaults, since voxj drops them).
fn write_material_palette(
    palette: &VoxjPalette,
    name: String,
    colors: Vec<u8>,
    path: &Path,
) -> Result<()> {
    let materials = palette
        .data
        .iter()
        .enumerate()
        .map(|(slot, cell)| {
            let mut material = parse_material(&palette.attributes, cell);
            material.mi = (slot + 1).to_string();
            material
        })
        .collect();
    let psb = VMaxPaletteSettingsVmaxpsbFile {
        name,
        materials,
        indices: Vec::new(),
        lc: vec![0u8; 256],
        colors,
        palette_type: 0,
        transparency: 1.0,
        r: 0,
        rt: "n".to_owned(),
        cmt: "ng".to_owned(),
        current: 0,
        ali: "1".to_owned(),
    };
    write_file(path, &to_vmaxpsb_bytes(&psb)?)?;
    Ok(())
}

/// Parses a `#RRGGBBAA` color cell into RGBA bytes.
fn parse_rgba(cell: &[VoxjValue]) -> [u8; 4] {
    let Some(VoxjValue::Text(hex)) = cell.first() else {
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

/// Parses a material cell, reading `metallic`/`roughness`/`emissive`/`shadows`
/// by attribute name and reconstructing the optional `md` dispersion block from
/// the `ior`/`transmission`/`absorption` columns. `md` is restored only when at
/// least one of those columns holds a number for this slot (slots without
/// dispersion carry `null` there), so the per-slot presence round-trips.
fn parse_material(attributes: &[String], cell: &[VoxjValue]) -> VMaxMaterial {
    let value = |name: &str| {
        attributes
            .iter()
            .position(|a| a == name)
            .and_then(|i| cell.get(i))
    };
    let number = |name: &str, default: f64| match value(name) {
        Some(VoxjValue::Number(n)) => *n,
        _ => default,
    };
    let dispersed = ["ior", "transmission", "absorption"]
        .iter()
        .any(|name| matches!(value(name), Some(VoxjValue::Number(_))));
    VMaxMaterial {
        mi: String::new(),
        mc: number("metallic", 0.0),
        rc: number("roughness", 0.0),
        sic: number("emissive", 0.0),
        sh: matches!(value("shadows"), Some(VoxjValue::Bool(true))),
        md: dispersed.then(|| VMaxMaterialDispersion {
            absorption: number("absorption", 0.0),
            ior: number("ior", 1.5),
            transmission: number("transmission", 0.0),
        }),
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

#[cfg(test)]
mod tests {
    use super::parse_material;
    use vmax::VMaxMaterialDispersion;
    use voxj::VoxjValue;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn reconstructs_md_from_dispersion_columns() {
        let attributes = names(&[
            "metallic",
            "roughness",
            "emissive",
            "shadows",
            "ior",
            "transmission",
            "absorption",
        ]);
        let cell = vec![
            VoxjValue::Number(0.66),
            VoxjValue::Number(0.58),
            VoxjValue::Number(4.2),
            VoxjValue::Bool(false),
            VoxjValue::Number(1.5),
            VoxjValue::Number(0.83),
            VoxjValue::Number(0.0),
        ];
        let material = parse_material(&attributes, &cell);
        assert_eq!(material.mc, 0.66);
        assert_eq!(material.sic, 4.2);
        assert!(!material.sh);
        assert_eq!(
            material.md,
            Some(VMaxMaterialDispersion {
                absorption: 0.0,
                ior: 1.5,
                transmission: 0.83,
            })
        );
    }

    #[test]
    fn null_dispersion_columns_leave_md_none() {
        let attributes = names(&[
            "metallic",
            "roughness",
            "emissive",
            "shadows",
            "ior",
            "transmission",
            "absorption",
        ]);
        let cell = vec![
            VoxjValue::Number(0.1),
            VoxjValue::Number(0.9),
            VoxjValue::Number(0.0),
            VoxjValue::Bool(true),
            VoxjValue::Null,
            VoxjValue::Null,
            VoxjValue::Null,
        ];
        let material = parse_material(&attributes, &cell);
        assert!(material.sh);
        assert_eq!(material.md, None);
    }

    #[test]
    fn missing_dispersion_columns_leave_md_none() {
        let attributes = names(&["metallic", "roughness", "emissive", "shadows"]);
        let cell = vec![
            VoxjValue::Number(0.1),
            VoxjValue::Number(0.9),
            VoxjValue::Number(0.0),
            VoxjValue::Bool(true),
        ];
        assert_eq!(parse_material(&attributes, &cell).md, None);
    }
}
