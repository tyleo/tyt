use crate::{
    Error, Result, VoxelMaxExt, VoxelMaxNode, VoxelMaxObjectState, VoxelMaxPalette, VoxjEncoding,
    VoxjFormat, VoxjPositionEncoding, VoxjSampleEncoding, axis_angle_to_quat,
    object_state_from_data, quat_rotate,
};
use std::{
    collections::HashMap,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use tyt_injection::serde_json::{self, Map, Value};
use vmax::{VMaxCodecFile, VMaxCodecVoxel, VMaxObject, VMaxSceneJsonFile, VMaxSerdeFile};
use vmax_codec::{decode_vmax_file, from_vmax_file};
use voxj::{
    VoxjCodecFile, VoxjCodecMain, VoxjCodecObject, VoxjHierarchyNode, VoxjPalette, VoxjTransform,
    VoxjValue,
};
use voxj_codec::{
    PositionEncoding, SampleEncoding, encode_voxj_file, encode_voxj_file_smallest,
    to_voxj_file_bytes, to_voxj_pretty_file_bytes, to_voxjz_file_bytes,
};

/// Number of cells in a color palette; a `palette*.png` is 256x1 RGBA, and a
/// placeholder palette covers every possible color index.
const COLOR_CELLS: usize = 256;

/// Uniform color used for every cell of a placeholder palette when an object's
/// `palette*.png` is absent, so color indices are preserved even though the
/// actual colors are unknown.
const PLACEHOLDER_COLOR: &str = "#FFFFFFFF";

/// `scene.json` node keys the voxj document already represents natively, dropped
/// from the `voxel-max` provenance so they aren't duplicated in each node's
/// `extra`: name (`n` / group `name`), position (`t_p`), scale (`t_s`), and the
/// `data` / `pal` / `hist` filenames (regenerated on reconstruction).
const NATIVE_NODE_KEYS: [&str; 7] = ["n", "name", "t_p", "t_s", "data", "pal", "hist"];

/// Converts the `.vmax` package at `input` into a Voxel Json document written to
/// stdout, following decode -> translate -> encode: [`from_vmax_file`] and
/// [`decode_vmax_file`] decode the package, [`vmax_to_voxj`] translates it to the
/// voxj model, and the voxj codec block-encodes and serializes it.
pub(crate) fn write_voxj(input: &Path, encoding: VoxjEncoding, format: VoxjFormat) -> Result<()> {
    // Decode: read the whole package once (serving `scene.json` from the bytes
    // kept for the ext), then decode its geometry and materials.
    let scene_bytes = tyt_injection::read_file(&input.join("scene.json"))?;
    let serde = from_vmax_file(|name| {
        if name == "scene.json" {
            return Ok(Some(scene_bytes.clone()));
        }
        match tyt_injection::read_file(&input.join(name)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    })?;
    let codec = decode_vmax_file(&serde)?;

    // Translate: decoded vmax -> voxj codec document (geometry, palettes,
    // hierarchy, plus the `voxel-max` ext carrying the rest of the provenance).
    let file = vmax_to_voxj(&codec, &serde, &scene_bytes)?;

    // Encode: block-encode the document in one pass, then serialize the chosen
    // container.
    let serialized = match encoding {
        VoxjEncoding::Fixed { position, sample } => {
            encode_voxj_file(&file, position_encoding(position), sample_encoding(sample))
        }
        VoxjEncoding::Smallest => encode_voxj_file_smallest(&file),
    };
    let bytes = match format {
        VoxjFormat::Json => to_voxj_file_bytes(&serialized),
        VoxjFormat::PrettyJson => to_voxj_pretty_file_bytes(&serialized),
        VoxjFormat::Zip => to_voxjz_file_bytes(&serialized),
    }
    .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

    tyt_injection::write_stdout(&bytes)?;
    Ok(())
}

/// Maps a CLI position-encoding choice to the voxj codec encoding.
fn position_encoding(encoding: VoxjPositionEncoding) -> PositionEncoding {
    match encoding {
        VoxjPositionEncoding::RawJson => PositionEncoding::RawJson,
        VoxjPositionEncoding::BitmapBase64 => PositionEncoding::BitmapBase64,
        VoxjPositionEncoding::Hilbert => PositionEncoding::Hilbert,
    }
}

/// Maps a CLI sample-encoding choice to the voxj codec encoding.
fn sample_encoding(encoding: VoxjSampleEncoding) -> SampleEncoding {
    match encoding {
        VoxjSampleEncoding::RawJson => SampleEncoding::RawJson,
        VoxjSampleEncoding::RleJson => SampleEncoding::RleJson,
        VoxjSampleEncoding::PackedBase64 => SampleEncoding::PackedBase64,
    }
}

/// Translates a decoded `.vmax` package into a voxj codec document: one voxj
/// object per distinct geometry (instances of one geometry collapse), shared
/// color/material palettes, the hierarchy, and the `voxel-max` ext carrying the
/// remaining Voxel Max provenance.
fn vmax_to_voxj(
    codec: &VMaxCodecFile,
    serde: &VMaxSerdeFile,
    scene_bytes: &[u8],
) -> Result<VoxjCodecFile> {
    let scene = &codec.scene_json_file;

    // Palettes are shared across objects and deduped by source filename.
    let mut palettes: Vec<VoxjPalette> = Vec::new();
    let mut palette_index: HashMap<String, usize> = HashMap::new();
    // Display name of each material palette, keyed by its `palettes` index.
    let mut palette_name_by_index: HashMap<usize, String> = HashMap::new();

    // One voxj object per distinct geometry; instances of one geometry collapse
    // to a single object.
    let mut objects: Vec<VoxjCodecObject> = Vec::new();
    // The `data` filename backing each voxj object (for the ext's object states).
    let mut object_data: Vec<Option<String>> = Vec::new();
    // Per scene object: its node transform and the voxj object it places.
    let mut object_transforms: Vec<VoxjTransform> = Vec::new();
    let mut object_refs: Vec<usize> = Vec::new();
    let mut instances: HashMap<InstanceKey, usize> = HashMap::new();
    for object in &scene.objects {
        let key = instance_key(object);
        if let Some(&existing) = key.as_ref().and_then(|key| instances.get(key)) {
            // Reuse the built geometry; keep this placement, skip re-translating.
            let (box_min, _) = authored_box(object).expect("instance key implies authored bounds");
            let box_min_f = [box_min[0] as f64, box_min[1] as f64, box_min[2] as f64];
            object_transforms.push(object_transform(object, box_min_f));
            object_refs.push(existing);
            continue;
        }
        let (voxj_object, transform) = build_object(
            codec,
            object,
            &mut palettes,
            &mut palette_index,
            &mut palette_name_by_index,
        )?;
        let index = objects.len();
        objects.push(voxj_object);
        object_data.push((!object.data.is_empty()).then(|| object.data.clone()));
        object_transforms.push(transform);
        object_refs.push(index);
        if let Some(key) = key {
            instances.insert(key, index);
        }
    }

    let (hierarchy_nodes, root_hierarchy_nodes) =
        build_hierarchy(scene, &object_transforms, &object_refs);

    // Material-palette names aligned by index with `main.palettes` for the ext.
    let palette_names: Vec<Option<String>> = (0..palettes.len())
        .map(|i| palette_name_by_index.get(&i).cloned())
        .collect();

    // Each object's preserved editor state, aligned by index with `objects`,
    // read straight off the parsed contents files (no second `.vmaxb` parse).
    let object_states: Vec<Option<VoxelMaxObjectState>> = object_data
        .iter()
        .map(|data| {
            data.as_deref()
                .and_then(|data| serde.contents_files.get(data))
                .map(object_state_from_data)
        })
        .collect();

    // Stash the vmax-specific state that has no native voxj home under the
    // generic `ext` namespace so `from-voxj` can rebuild the package exactly.
    let ext = voxel_max_ext(scene_bytes, &palette_names, &object_states)?;

    Ok(VoxjCodecFile {
        version: 1,
        main: VoxjCodecMain {
            objects,
            palettes,
            hierarchy_nodes,
            root_hierarchy_nodes,
            ext: Some(ext),
        },
    })
}

/// Translates one scene object into its voxj geometry plus the transform of the
/// hierarchy node that places it, reading voxels and palettes from the
/// already-decoded `codec`.
fn build_object(
    codec: &VMaxCodecFile,
    object: &VMaxObject,
    palettes: &mut Vec<VoxjPalette>,
    palette_index: &mut HashMap<String, usize>,
    palette_name_by_index: &mut HashMap<usize, String>,
) -> Result<(VoxjCodecObject, VoxjTransform)> {
    let voxels: &[VMaxCodecVoxel] = if object.data.is_empty() {
        &[]
    } else {
        &codec.contents_files[&object.data].voxels
    };

    // Always use the authored box (`e_c + e_mi` .. `e_c + e_ma`) when the object
    // has Voxel Max bounds, so the encoded bounds match vmax exactly (even for
    // empty objects); only objects without authored bounds use the tight extent
    // of their voxels.
    let (box_min, bounds) = object_box(object, voxels, min_corner(voxels))?;
    let box_min_f = [box_min[0] as f64, box_min[1] as f64, box_min[2] as f64];

    if voxels.is_empty() {
        // No geometry to place or color, but keep the authored bounds.
        let empty = VoxjCodecObject {
            name: object.name.clone(),
            palette_refs: Vec::new(),
            bounds,
            positions: Vec::new(),
            samples: Vec::new(),
        };
        return Ok((empty, object_transform(object, box_min_f)));
    }

    let positions: Vec<[u32; 3]> = voxels
        .iter()
        .map(|v| {
            [
                (v.position[0] - box_min[0]) as u32,
                (v.position[1] - box_min[1]) as u32,
                (v.position[2] - box_min[2]) as u32,
            ]
        })
        .collect();

    // Objects normally reference a palette; tolerate one that is absent.
    let color_palette = color_palette(codec, object, palettes, palette_index);
    let material_palette = material_palette(
        codec,
        object,
        palettes,
        palette_index,
        palette_name_by_index,
    );

    let mut palette_refs = Vec::new();
    let mut samples: Vec<Vec<u32>> = vec![Vec::new(); voxels.len()];
    if let Some(index) = color_palette {
        palette_refs.push(index);
        for (sample, voxel) in samples.iter_mut().zip(voxels) {
            sample.push(voxel.color_idx as u32);
        }
    }
    if let Some(index) = material_palette {
        palette_refs.push(index);
        for (sample, voxel) in samples.iter_mut().zip(voxels) {
            sample.push(voxel.material_idx as u32);
        }
    }

    let voxj_object = VoxjCodecObject {
        name: object.name.clone(),
        palette_refs,
        bounds,
        positions,
        samples,
    };
    Ok((voxj_object, object_transform(object, box_min_f)))
}

/// Returns the shared color palette index for `object.palette`, adding it on
/// first use. The `rgba` cells come from the `palette*.png` pixels when present,
/// otherwise from the `colors` table of the `palette*.settings.vmaxpsb` sidecar,
/// and finally a uniform placeholder when neither carries color (so color
/// indices are still preserved). `None` only when the object names no palette.
fn color_palette(
    codec: &VMaxCodecFile,
    object: &VMaxObject,
    palettes: &mut Vec<VoxjPalette>,
    palette_index: &mut HashMap<String, usize>,
) -> Option<usize> {
    if object.palette.is_empty() {
        return None;
    }
    if let Some(&index) = palette_index.get(&object.palette) {
        return Some(index);
    }
    let data = color_cells(codec, object);
    let index = palettes.len();
    palettes.push(VoxjPalette {
        attributes: vec!["rgba".to_owned()],
        data,
    });
    palette_index.insert(object.palette.clone(), index);
    Some(index)
}

/// The `rgba` cells for an object's color palette: the `palette*.png` pixels when
/// that image is present, else the RGBA `colors` table of the
/// `palette*.settings.vmaxpsb` sidecar, and finally a uniform placeholder when
/// neither source carries color.
fn color_cells(codec: &VMaxCodecFile, object: &VMaxObject) -> Vec<Vec<VoxjValue>> {
    if let Some(png) = codec.palette_png_files.get(&object.palette) {
        return rgba_cells(&strip_color_terminator(png.0.clone()));
    }
    if let Some(stem) = object.palette.strip_suffix(".png") {
        let sidecar = format!("{stem}.settings.vmaxpsb");
        if let Some(palette) = codec.palette_settings_files.get(&sidecar)
            && !palette.colors.is_empty()
        {
            return rgba_cells(&strip_color_terminator(palette.colors.clone()));
        }
    }
    (0..COLOR_CELLS)
        .map(|_| vec![VoxjValue::Text(PLACEHOLDER_COLOR.to_owned())])
        .collect()
}

/// Returns the shared material palette index for an object's
/// `palette*.settings.vmaxpsb`, or `None` when that sidecar is absent or carries
/// no materials. The per-slot material properties
/// (metalness/roughness/emission/shadows) become native palette cells; the
/// palette's display name is recorded in `palette_name_by_index` for the
/// `voxel-max` ext.
fn material_palette(
    codec: &VMaxCodecFile,
    object: &VMaxObject,
    palettes: &mut Vec<VoxjPalette>,
    palette_index: &mut HashMap<String, usize>,
    palette_name_by_index: &mut HashMap<usize, String>,
) -> Option<usize> {
    let stem = object.palette.strip_suffix(".png")?;
    let sidecar = format!("{stem}.settings.vmaxpsb");
    if let Some(&index) = palette_index.get(&sidecar) {
        return Some(index);
    }
    let palette = codec.palette_settings_files.get(&sidecar)?;
    if palette.materials.is_empty() {
        return None;
    }
    // Dispersion columns (`ior`/`transmission`/`absorption`) are added only when
    // some slot carries an `md` block, so palettes without dispersion stay
    // exactly as before. Slots lacking `md` then fill those columns with `null`
    // so every row spans every attribute.
    let has_dispersion = palette.materials.iter().any(|m| m.dispersion.is_some());
    let mut attributes = vec![
        "metallic".to_owned(),
        "roughness".to_owned(),
        "emissive".to_owned(),
        "shadows".to_owned(),
    ];
    if has_dispersion {
        attributes.extend([
            "ior".to_owned(),
            "transmission".to_owned(),
            "absorption".to_owned(),
        ]);
    }
    let data = palette
        .materials
        .iter()
        .map(|m| {
            let mut row = vec![
                VoxjValue::Number(m.metalness),
                VoxjValue::Number(m.roughness),
                VoxjValue::Number(m.emission),
                VoxjValue::Bool(m.enable_shadows),
            ];
            if has_dispersion {
                match m.dispersion {
                    Some(d) => row.extend([
                        VoxjValue::Number(d.ior),
                        VoxjValue::Number(d.transmission),
                        VoxjValue::Number(d.absorption),
                    ]),
                    None => row.extend([VoxjValue::Null, VoxjValue::Null, VoxjValue::Null]),
                }
            }
            row
        })
        .collect();
    let index = palettes.len();
    palettes.push(VoxjPalette { attributes, data });
    palette_index.insert(sidecar, index);
    palette_name_by_index.insert(index, palette.name.clone());
    Some(index)
}

/// Builds the `voxel-max` ext payload stashed at the voxj document's `main.ext`:
/// the scene-level state and per-node provenance with no native voxj home (each
/// node's unmodeled keys preserved verbatim in [`VoxelMaxNode`]'s `extra`), the
/// material-palette names, and the per-object editor states.
fn voxel_max_ext(
    scene_bytes: &[u8],
    palette_names: &[Option<String>],
    object_states: &[Option<VoxelMaxObjectState>],
) -> Result<VoxjValue> {
    let invalid = |e| -> Error { IOError::new(ErrorKind::InvalidData, e).into() };
    let mut value: Value = tyt_injection::parse_json(scene_bytes)?;
    let nodes = |value: &mut Value, key: &str| -> Result<Vec<VoxelMaxNode>> {
        let Some(Value::Array(array)) = value.as_object_mut().and_then(|map| map.remove(key))
        else {
            return Ok(Vec::new());
        };
        array
            .into_iter()
            .map(|mut node| {
                // Drop fields the voxj document already carries natively so they
                // don't fall through into `VoxelMaxNode::extra`.
                if let Some(map) = node.as_object_mut() {
                    for key in NATIVE_NODE_KEYS {
                        map.remove(key);
                    }
                }
                serde_json::from_value(node).map_err(invalid)
            })
            .collect()
    };
    // Aligned with `main.hierarchyNodes`: groups first, then objects.
    let mut hierarchy_nodes = nodes(&mut value, "groups")?;
    hierarchy_nodes.extend(nodes(&mut value, "objects")?);
    let scene = serde_json::from_value(value).map_err(invalid)?;

    let palettes = palette_names
        .iter()
        .map(|name| name.clone().map(|name| VoxelMaxPalette { name }))
        .collect();
    let voxel_max = serde_json::to_value(VoxelMaxExt {
        scene,
        hierarchy_nodes,
        palettes,
        object_states: object_states.to_vec(),
    })
    .map_err(invalid)?;
    let mut ext = Map::new();
    ext.insert("voxel-max".to_owned(), voxel_max);
    serde_json::from_value(Value::Object(ext)).map_err(invalid)
}

/// Drops the trailing transparent terminator Voxel Max appends to fill a color
/// table to [`COLOR_CELLS`]. That last index is a reserved `#00000000` slot no
/// voxel references, so keeping it would pad the model with a non-color and make
/// a `palette*.png`-sourced palette disagree in length with one read from the
/// material sidecar's already-terminator-free `colors` table.
fn strip_color_terminator(mut colors: Vec<[u8; 4]>) -> Vec<[u8; 4]> {
    if colors.len() == COLOR_CELLS && colors.last() == Some(&[0, 0, 0, 0]) {
        colors.pop();
    }
    colors
}

/// One `#RRGGBBAA` text cell per RGBA color.
fn rgba_cells(colors: &[[u8; 4]]) -> Vec<Vec<VoxjValue>> {
    colors
        .iter()
        .map(|&[r, g, b, a]| vec![VoxjValue::Text(format!("#{r:02X}{g:02X}{b:02X}{a:02X}"))])
        .collect()
}

/// The minimum `[x, y, z]` corner over `voxels`, or `None` when empty.
fn min_corner(voxels: &[VMaxCodecVoxel]) -> Option<[i32; 3]> {
    voxels.iter().fold(None, |acc, v| {
        let acc = acc.unwrap_or(v.position);
        Some([
            acc[0].min(v.position[0]),
            acc[1].min(v.position[1]),
            acc[2].min(v.position[2]),
        ])
    })
}

/// `[X, Y, Z]` bounds: the per-axis extent of `voxels` relative to `box_min`.
fn object_bounds(voxels: &[VMaxCodecVoxel], box_min: [i32; 3]) -> [u32; 3] {
    let mut bounds = [1u32; 3];
    for v in voxels {
        bounds[0] = bounds[0].max((v.position[0] - box_min[0] + 1) as u32);
        bounds[1] = bounds[1].max((v.position[1] - box_min[1] + 1) as u32);
        bounds[2] = bounds[2].max((v.position[2] - box_min[2] + 1) as u32);
    }
    bounds
}

/// Identifies voxj objects that are the same geometry placed more than once.
/// Voxel Max instances a model by reusing a `contents*.vmaxb` and its `palette`
/// across scene objects, so objects sharing the `data`/`palette` filenames and
/// the same authored box decode to one identical voxj object (differing only in
/// the placement carried by their wrapping nodes).
type InstanceKey = (String, String, [i32; 3], [u32; 3]);

/// The [`InstanceKey`] for an object, or `None` when it cannot be instanced: it
/// names no `data` file, or has no authored bounds to fix a shared box (such
/// objects always become their own voxj object).
fn instance_key(object: &VMaxObject) -> Option<InstanceKey> {
    if object.data.is_empty() {
        return None;
    }
    let (box_min, size) = authored_box(object)?;
    Some((object.data.clone(), object.palette.clone(), box_min, size))
}

/// The re-basing origin `round(center + bounds_min)` and `[X, Y, Z]` size from
/// an object's authored Voxel Max bounds, or `None` when it has none. Reads only
/// the bounds fields (no voxels), so it can key instancing without decoding.
fn authored_box(object: &VMaxObject) -> Option<([i32; 3], [u32; 3])> {
    let (min, max) = (object.bounds_min?, object.bounds_max?);
    let box_min = [
        (object.center[0] + min[0]).round() as i32,
        (object.center[1] + min[1]).round() as i32,
        (object.center[2] + min[2]).round() as i32,
    ];
    let size = [
        (max[0] - min[0]).round().max(0.0) as u32,
        (max[1] - min[1]).round().max(0.0) as u32,
        (max[2] - min[2]).round().max(0.0) as u32,
    ];
    Some((box_min, size))
}

/// The re-basing origin and `[X, Y, Z]` size for an object. Always uses the
/// object's Voxel Max bounds (`center + bounds_min` .. `center + bounds_max`)
/// when present, so the encoded bounds match vmax exactly and are never
/// recomputed; only objects with no authored bounds fall back to the tight
/// extent of their voxels. Errors if authored bounds do not enclose every voxel.
fn object_box(
    object: &VMaxObject,
    voxels: &[VMaxCodecVoxel],
    tight_min: Option<[i32; 3]>,
) -> Result<([i32; 3], [u32; 3])> {
    let Some((box_min, size)) = authored_box(object) else {
        // No authored bounds: derive a tight box, or origin for an empty object.
        return Ok(match tight_min {
            Some(tight) => (tight, object_bounds(voxels, tight)),
            None => ([0, 0, 0], [0, 0, 0]),
        });
    };
    for v in voxels {
        let local = [
            v.position[0] - box_min[0],
            v.position[1] - box_min[1],
            v.position[2] - box_min[2],
        ];
        if (0..3).any(|k| local[k] < 0 || local[k] as i64 >= i64::from(size[k])) {
            return Err(IOError::new(
                ErrorKind::InvalidData,
                format!(
                    "object '{}' has voxel ({}, {}, {}) outside its Voxel Max bounds \
                     (origin {box_min:?}, size {size:?}); refusing to change the authored bounds",
                    object.name, v.position[0], v.position[1], v.position[2]
                ),
            )
            .into());
        }
    }
    Ok((box_min, size))
}

/// The node transform that places an object whose voxels are authored from the
/// origin: `position = t_p + R*S*(box_min - e_c)`, `rotation = quat(t_r)`,
/// `scale = t_s`. This reproduces Voxel Max's pivot-about-`e_c` placement.
fn object_transform(object: &VMaxObject, box_min: [f64; 3]) -> VoxjTransform {
    let rotation = axis_angle_to_quat(object.rotation);
    let scale = object.scale;
    let offset = [
        (box_min[0] - object.center[0]) * scale[0],
        (box_min[1] - object.center[1]) * scale[1],
        (box_min[2] - object.center[2]) * scale[2],
    ];
    let rotated = quat_rotate(rotation, offset);
    VoxjTransform {
        position: [
            object.position[0] + rotated[0],
            object.position[1] + rotated[1],
            object.position[2] + rotated[2],
        ],
        rotation,
        scale,
    }
}

/// Builds the voxj hierarchy: one node per group and one per object (the latter
/// wrapping its geometry). `object_refs[i]` is the voxj object that scene object
/// `i` places, so instances of one geometry share a `child_objects` index.
/// Returns the nodes and the indices of the roots.
fn build_hierarchy(
    scene: &VMaxSceneJsonFile,
    object_transforms: &[VoxjTransform],
    object_refs: &[usize],
) -> (Vec<VoxjHierarchyNode>, Vec<usize>) {
    let mut nodes: Vec<VoxjHierarchyNode> = Vec::new();
    let mut node_of_id: HashMap<&str, usize> = HashMap::new();
    let mut parents: Vec<Option<&str>> = Vec::new();

    for group in &scene.groups {
        node_of_id.insert(&group.id, nodes.len());
        parents.push(group.parent_id.as_deref());
        nodes.push(VoxjHierarchyNode {
            name: group.name.clone(),
            child_nodes: Vec::new(),
            child_objects: Vec::new(),
            transform: VoxjTransform {
                position: group.position,
                rotation: axis_angle_to_quat(group.rotation),
                scale: group.scale,
            },
        });
    }
    for (object_index, object) in scene.objects.iter().enumerate() {
        node_of_id.insert(&object.id, nodes.len());
        parents.push(object.parent_id.as_deref());
        nodes.push(VoxjHierarchyNode {
            name: object.name.clone(),
            child_nodes: Vec::new(),
            child_objects: vec![object_refs[object_index]],
            transform: object_transforms[object_index],
        });
    }

    let mut roots = Vec::new();
    for (node, parent) in parents.iter().enumerate() {
        match parent.and_then(|pid| node_of_id.get(pid)) {
            Some(&parent_node) => nodes[parent_node].child_nodes.push(node),
            None => roots.push(node),
        }
    }

    (nodes, roots)
}
