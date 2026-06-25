use crate::{
    Error, Result, VoxelMaxColorFormat, VoxelMaxExtWrapper, VoxelMaxNode, VoxelMaxPalette,
    from_vox_value,
};
use branded_id::U32Id;
use std::collections::{BTreeMap, HashMap};
use ty_math::{TyTransformF64, TyVector3F64};
use vmax::{
    VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion, VMaxObject,
    VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile,
};
use vmax_codec::{VMaxVoxel, encode_vmax_snapshots};
use voxcore::{
    BVoxPalette, BVoxPaletteRef, VoxHierarchyNode, VoxObject, VoxPalette, VoxState, VoxValue,
};

/// Usable colors in a Voxel Max palette: indices 0..254. Index 255 is a reserved
/// transparent terminator no voxel references, so the table holds this many
/// entries and the image pads to 256 by appending the terminator.
const PALETTE_COLORS: usize = 255;

/// Codable version stamped on a rebuilt contents file when the state carries no
/// preserved object version.
const FALLBACK_CONTENT_VERSION: i64 = 4;

/// Axis-angle stored on a node with no preserved rotation; a degenerate axis
/// decodes to the identity quaternion.
const IDENTITY_AXIS_ANGLE: [f64; 4] = [0.0, 0.0, 0.0, 0.0];

/// Writes a [`VoxState`] back to a Voxel Max document, the inverse of
/// [`from_vmax_file`](crate::from_vmax_file).
/// `voxel_max_color_format` selects where each palette's colors are stored, as
/// described on [`VoxelMaxColorFormat`].
///
/// Requires the `voxel-max` ext the forward path writes; without it the
/// scene-level state cannot be rebuilt. Editor session artifacts that voxcore
/// does not model are dropped.
pub fn to_vmax_file(
    state: &VoxState,
    voxel_max_color_format: VoxelMaxColorFormat,
) -> Result<VMaxFile> {
    let voxel_max = match state.ext() {
        Some(ext) => from_vox_value::<VoxelMaxExtWrapper>(ext)?.voxel_max,
        None => {
            return Err(Error::invalid(
                "state has no voxel-max ext; cannot rebuild a Voxel Max document",
            ));
        }
    };

    let mut objects: Vec<VMaxObject> = Vec::new();
    let mut groups: Vec<VMaxGroup> = Vec::new();
    // Color palette id -> the `pal` filename written for it.
    let mut palette_files: HashMap<usize, String> = HashMap::new();
    // Object id -> its `data` filename, shared by every node that places it.
    let mut contents_by_object: HashMap<usize, String> = HashMap::new();

    let mut contents_files: BTreeMap<String, VMaxContentsVmaxbFile> = BTreeMap::new();
    let mut palette_settings_files: BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile> =
        BTreeMap::new();
    let mut palette_png_files: BTreeMap<String, VMaxPalettePngFile> = BTreeMap::new();

    for (index, (_, node)) in state.iter_hierarchy_nodes().enumerate() {
        let ext_node = voxel_max
            .hierarchy_nodes
            .get(index)
            .cloned()
            .unwrap_or_default();

        if node.child_objects.is_empty() {
            groups.push(group_from_node(node, &ext_node));
            continue;
        }

        let object_ref = node.child_objects[0];
        let object_id = object_ref.to_u32() as usize;
        let object = state.object(object_ref).expect("a valid node child object");
        let (color, material) = object_palette_refs(state, object);
        let suffix = suffix(object_id);

        // Instances share one contents file: rebuild it once.
        let data = match contents_by_object.get(&object_id) {
            Some(data) => data.clone(),
            None => {
                let voxels = reconstruct_voxels(object, color, material, &ext_node);
                let data = format!("contents{suffix}.vmaxb");
                // Voxels re-encode into snapshots; serde-only state the decoded
                // voxcore object does not model (`pal`) stays absent.
                let contents = match voxel_max
                    .object_states
                    .get(object_id)
                    .and_then(|s| s.clone())
                {
                    Some(state) => VMaxContentsVmaxbFile {
                        snapshots: encode_vmax_snapshots(&voxels),
                        uuid: state.uuid,
                        v: state.v,
                        tools: state.tools,
                        brush: state.brush,
                        cam: state.cam,
                        pal: None,
                    },
                    None => VMaxContentsVmaxbFile {
                        snapshots: encode_vmax_snapshots(&voxels),
                        uuid: ext_node.id.clone(),
                        v: FALLBACK_CONTENT_VERSION,
                        tools: None,
                        brush: None,
                        cam: None,
                        pal: None,
                    },
                };
                contents_files.insert(data.clone(), contents);
                contents_by_object.insert(object_id, data.clone());
                data
            }
        };

        let pal = build_palette(
            state,
            &voxel_max.palettes,
            color.map(|(_, palette)| palette),
            material.map(|(_, palette)| palette),
            &mut palette_files,
            &mut palette_settings_files,
            &mut palette_png_files,
            voxel_max_color_format,
        );

        objects.push(object_from_node(node, &ext_node, data, pal, &suffix));
    }

    let mut scene = voxel_max.scene;
    scene.groups = groups;
    scene.objects = objects;

    Ok(VMaxFile {
        scene_json_file: scene,
        contents_files,
        palette_settings_files,
        palette_png_files,
        history_vmaxhb_files: BTreeMap::new(),
        history_vmaxhvsb_files: BTreeMap::new(),
        history_vmaxhvsc_files: BTreeMap::new(),
        selection_vmaxb_files: BTreeMap::new(),
        thumbnail_png: None,
        contents_vmax_pngs: BTreeMap::new(),
        group_pngs: BTreeMap::new(),
    })
}

/// A reference into one of an object's palettes: the sample reference id and the
/// palette id it names.
type PaletteRef = (U32Id<BVoxPaletteRef>, U32Id<BVoxPalette>);

/// The color and material palette references for an object, identified by the
/// `rgba` and `metallic` attributes.
fn object_palette_refs(
    state: &VoxState,
    object: &VoxObject,
) -> (Option<PaletteRef>, Option<PaletteRef>) {
    let mut color = None;
    let mut material = None;
    for (reference, palette) in object.iter_palette_refs() {
        let Some(palette_value) = state.palette(palette) else {
            continue;
        };
        if has_attribute(palette_value, "rgba") {
            color = Some((reference, palette));
        } else if has_attribute(palette_value, "metallic") {
            material = Some((reference, palette));
        }
    }
    (color, material)
}

/// Whether `palette` declares an attribute named `name`.
fn has_attribute(palette: &VoxPalette, name: &str) -> bool {
    palette
        .iter_attributes()
        .any(|(_, attribute)| attribute == name)
}

/// Re-bases an object's voxels to absolute model space, reading the color and
/// material indices from their sample references.
fn reconstruct_voxels(
    object: &VoxObject,
    color: Option<PaletteRef>,
    material: Option<PaletteRef>,
    ext_node: &VoxelMaxNode,
) -> Vec<VMaxVoxel> {
    let box_min = box_min(ext_node);
    object
        .iter_live()
        .map(|voxel| {
            let position = object
                .voxel_position(voxel)
                .expect("a live voxel is within the grid");
            let cell = |reference: Option<PaletteRef>| {
                reference.map_or(0, |(reference, _)| {
                    object
                        .voxel_cell(voxel, reference)
                        .expect("a live voxel samples every reference")
                        .to_u32() as u8
                })
            };
            VMaxVoxel {
                position: [
                    position.x as i32 + box_min[0],
                    position.y as i32 + box_min[1],
                    position.z as i32 + box_min[2],
                ],
                material_idx: cell(material),
                color_idx: cell(color),
            }
        })
        .collect()
}

/// Returns the `pal` filename for an object, building its color image and
/// material sidecar the first time the color palette is seen.
#[allow(clippy::too_many_arguments)]
fn build_palette(
    state: &VoxState,
    palette_names: &[Option<VoxelMaxPalette>],
    color: Option<U32Id<BVoxPalette>>,
    material: Option<U32Id<BVoxPalette>>,
    palette_files: &mut HashMap<usize, String>,
    palette_settings_files: &mut BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile>,
    palette_png_files: &mut BTreeMap<String, VMaxPalettePngFile>,
    voxel_max_color_format: VoxelMaxColorFormat,
) -> String {
    let Some(color) = color else {
        return "palette.png".to_owned();
    };
    let color_id = color.to_u32() as usize;
    if let Some(name) = palette_files.get(&color_id) {
        return name.clone();
    }
    let stem = match palette_files.len() {
        0 => String::new(),
        n => n.to_string(),
    };
    let pal = format!("palette{stem}.png");

    let colors = color_palette_colors(state, color);
    if matches!(
        voxel_max_color_format,
        VoxelMaxColorFormat::Png | VoxelMaxColorFormat::All
    ) {
        let mut cells = colors.clone();
        cells.push([0, 0, 0, 0]);
        palette_png_files.insert(pal.clone(), VMaxPalettePngFile(cells));
    }
    if let Some(material) = material {
        let name = palette_names
            .get(material.to_u32() as usize)
            .and_then(|palette| palette.clone())
            .map(|palette| palette.name)
            .unwrap_or_default();
        let sidecar = format!("palette{stem}.settings.vmaxpsb");
        let sidecar_colors = match voxel_max_color_format {
            VoxelMaxColorFormat::Png => Vec::new(),
            VoxelMaxColorFormat::Plist | VoxelMaxColorFormat::All => colors,
        };
        palette_settings_files.insert(
            sidecar,
            material_settings(state, material, name, sidecar_colors),
        );
    }
    palette_files.insert(color_id, pal.clone());
    pal
}

/// A color palette's cells as exactly [`PALETTE_COLORS`] RGBA entries, padded
/// with transparent cells or truncated so the count is fixed.
fn color_palette_colors(state: &VoxState, palette: U32Id<BVoxPalette>) -> Vec<[u8; 4]> {
    let palette = state.palette(palette).expect("a referenced palette");
    let rgba = palette
        .iter_attributes()
        .find(|(_, name)| *name == "rgba")
        .map(|(id, _)| id);
    let mut cells: Vec<[u8; 4]> = match rgba {
        Some(rgba) => palette
            .iter_cells()
            .take(PALETTE_COLORS)
            .map(|cell| parse_rgba(palette.cell_value(cell, rgba)))
            .collect(),
        None => Vec::new(),
    };
    cells.resize(PALETTE_COLORS, [0, 0, 0, 0]);
    cells
}

/// Builds a material palette from a voxcore palette's cells, reading the
/// `metallic`/`roughness`/`emissive`/`shadows` attributes and the optional
/// dispersion columns. The editor-state keys are filled with the defaults Voxel
/// Max expects, and each slot's `mi` token from its 1-based position.
fn material_settings(
    state: &VoxState,
    palette: U32Id<BVoxPalette>,
    name: String,
    colors: Vec<[u8; 4]>,
) -> VMaxPaletteSettingsVmaxpsbFile {
    let palette = state.palette(palette).expect("a referenced palette");
    let attributes: HashMap<String, _> = palette
        .iter_attributes()
        .map(|(id, attribute)| (attribute.to_owned(), id))
        .collect();

    let materials = palette
        .iter_cells()
        .enumerate()
        .map(|(slot, cell)| {
            let value = |name: &str| {
                attributes
                    .get(name)
                    .and_then(|&id| palette.cell_value(cell, id))
            };
            let number = |name: &str, default: f64| match value(name) {
                Some(VoxValue::Number(n)) => *n,
                _ => default,
            };
            let dispersed = ["ior", "transmission", "absorption"]
                .iter()
                .any(|name| matches!(value(name), Some(VoxValue::Number(_))));
            VMaxMaterial {
                mi: (slot + 1).to_string(),
                mc: number("metallic", 0.0),
                rc: number("roughness", 0.0),
                sic: number("emissive", 0.0),
                sh: matches!(value("shadows"), Some(VoxValue::Bool(true))),
                tc: None,
                md: dispersed.then(|| VMaxMaterialDispersion {
                    absorption: number("absorption", 0.0),
                    ior: number("ior", 1.5),
                    transmission: number("transmission", 0.0),
                }),
            }
        })
        .collect();

    VMaxPaletteSettingsVmaxpsbFile {
        name,
        materials,
        colors: colors.iter().flatten().copied().collect(),
        indices: Vec::new(),
        lc: vec![0u8; 256],
        palette_type: 0,
        transparency: 1.0,
        r: 0,
        rt: "n".to_owned(),
        cmt: "ng".to_owned(),
        current: 0,
        ali: "1".to_owned(),
        voxmats: Vec::new(),
        ls: Vec::new(),
    }
}

/// Parses a `#RRGGBBAA` color cell into RGBA bytes.
fn parse_rgba(cell: Option<&VoxValue>) -> [u8; 4] {
    let Some(VoxValue::Text(hex)) = cell else {
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

/// Builds a scene object from its node and preserved provenance.
fn object_from_node(
    node: &VoxHierarchyNode,
    ext_node: &VoxelMaxNode,
    data: String,
    pal: String,
    suffix: &str,
) -> VMaxObject {
    VMaxObject {
        name: node.name.clone(),
        data,
        palette: pal,
        history: format!("history{suffix}.vmaxhb"),
        id: ext_node.id.clone(),
        parent_id: ext_node.parent_id.clone(),
        hidden: None,
        position: unbake_position(&node.transform, ext_node),
        rotation: ext_node.rotation.unwrap_or(IDENTITY_AXIS_ANGLE),
        scale: vector(node.transform.scale),
        ind: ext_node.index.unwrap_or_default(),
        s: ext_node.selected,
        t_al: ext_node.alignment.clone().unwrap_or_default(),
        t_pa: ext_node.pivot_align.clone().unwrap_or_default(),
        t_pf: ext_node.pivot_face.clone().unwrap_or_default(),
        t_po: None,
        center: ext_node.center.unwrap_or_default(),
        bounds_min: ext_node.bounds_min,
        bounds_max: ext_node.bounds_max,
    }
}

/// Builds a scene group from its node and preserved provenance.
fn group_from_node(node: &VoxHierarchyNode, ext_node: &VoxelMaxNode) -> VMaxGroup {
    VMaxGroup {
        name: node.name.clone(),
        id: ext_node.id.clone(),
        parent_id: ext_node.parent_id.clone(),
        hidden: None,
        position: vector(node.transform.position),
        rotation: ext_node.rotation.unwrap_or(IDENTITY_AXIS_ANGLE),
        scale: vector(node.transform.scale),
        ind: ext_node.index.unwrap_or_default(),
        s: ext_node.selected,
        t_al: ext_node.alignment.clone().unwrap_or_default(),
        t_pa: ext_node.pivot_align.clone().unwrap_or_default(),
        t_pf: ext_node.pivot_face.clone().unwrap_or_default(),
        t_po: None,
        center: ext_node.center.unwrap_or_default(),
        bounds_min: ext_node.bounds_min,
        bounds_max: ext_node.bounds_max,
    }
}

/// The absolute model-space origin `round(center + bounds_min)` the forward path
/// re-based against.
fn box_min(ext_node: &VoxelMaxNode) -> [i32; 3] {
    let center = ext_node.center.unwrap_or_default();
    let min = ext_node.bounds_min.unwrap_or_default();
    [
        (center[0] + min[0]).round() as i32,
        (center[1] + min[1]).round() as i32,
        (center[2] + min[2]).round() as i32,
    ]
}

/// Recovers an object's `t_p` by undoing the forward placement that put its
/// voxels at `box_min`.
fn unbake_position(transform: &TyTransformF64, ext_node: &VoxelMaxNode) -> [f64; 3] {
    let center = ext_node.center.unwrap_or_default();
    let min = box_min(ext_node);
    let scale = transform.scale;
    let offset = TyVector3F64::new(
        (min[0] as f64 - center[0]) * scale.x,
        (min[1] as f64 - center[1]) * scale.y,
        (min[2] as f64 - center[2]) * scale.z,
    );
    let rotated = transform.rotation.rotate(offset);
    [
        transform.position.x - rotated.x,
        transform.position.y - rotated.y,
        transform.position.z - rotated.z,
    ]
}

/// A `[f64; 3]` from a vector.
fn vector(vector: TyVector3F64) -> [f64; 3] {
    [vector.x, vector.y, vector.z]
}

/// The filename suffix for object `index`: empty for the first, then the index.
fn suffix(index: usize) -> String {
    if index == 0 {
        String::new()
    } else {
        index.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{VoxelMaxColorFormat, from_vmax_file, to_vmax_file};
    use std::collections::BTreeMap;
    use vmax::{
        VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxObject, VMaxPalettePngFile,
        VMaxSceneCamera, VMaxSceneJsonFile,
    };
    use vmax_codec::{VMaxVoxel, encode_vmax_snapshots};
    use voxcore::VoxState;

    fn material(
        mi: &str,
        metalness: f64,
        roughness: f64,
        emission: f64,
        enable_shadows: bool,
    ) -> VMaxMaterial {
        VMaxMaterial {
            mi: mi.to_owned(),
            mc: metalness,
            rc: roughness,
            sic: emission,
            sh: enable_shadows,
            tc: None,
            md: None,
        }
    }

    /// The fixed editor-state defaults the reverse path writes on a rebuilt
    /// material palette, so a fixture round-trips to equality.
    fn palette_settings(
        name: &str,
        materials: Vec<VMaxMaterial>,
        colors: Vec<[u8; 4]>,
    ) -> vmax::VMaxPaletteSettingsVmaxpsbFile {
        vmax::VMaxPaletteSettingsVmaxpsbFile {
            name: name.to_owned(),
            materials,
            colors: colors.iter().flatten().copied().collect(),
            indices: Vec::new(),
            lc: vec![0u8; 256],
            palette_type: 0,
            transparency: 1.0,
            r: 0,
            rt: "n".to_owned(),
            cmt: "ng".to_owned(),
            current: 0,
            ali: "1".to_owned(),
            voxmats: Vec::new(),
            ls: Vec::new(),
        }
    }

    /// A 256-cell image: 255 distinct colors plus the transparent terminator.
    fn palette_png() -> VMaxPalettePngFile {
        let mut cells: Vec<[u8; 4]> = (0..255u32).map(|i| [i as u8, 0, 0, 255]).collect();
        cells.push([0, 0, 0, 0]);
        VMaxPalettePngFile(cells)
    }

    /// A document with a root group, a child object placed at the origin with
    /// authored bounds, a shared color and material palette, and preserved scene
    /// and object editor state. Built in the canonical form the reverse path
    /// emits so it round-trips to equality.
    fn sample() -> VMaxFile {
        let group = VMaxGroup {
            name: "grp".to_owned(),
            id: "g".to_owned(),
            parent_id: None,
            hidden: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ind: [0, 0, 0],
            s: Some(false),
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            center: [0.0, 0.0, 0.0],
            bounds_min: None,
            bounds_max: None,
        };
        let object = VMaxObject {
            name: "obj".to_owned(),
            data: "contents.vmaxb".to_owned(),
            palette: "palette.png".to_owned(),
            history: "history.vmaxhb".to_owned(),
            id: "o".to_owned(),
            parent_id: Some("g".to_owned()),
            hidden: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ind: [0, 0, 0],
            s: Some(false),
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            center: [0.0, 0.0, 0.0],
            bounds_min: Some([0.0, 0.0, 0.0]),
            bounds_max: Some([2.0, 2.0, 2.0]),
        };

        let scene_json_file = VMaxSceneJsonFile {
            v: 4,
            cam: Some(VMaxSceneCamera::default()),
            background: Some("#101010".to_owned()),
            groups: vec![group],
            objects: vec![object],
            ..Default::default()
        };

        // Canonical snapshots: voxels re-encoded just as the reverse path emits.
        let contents = VMaxContentsVmaxbFile {
            snapshots: encode_vmax_snapshots(&[
                VMaxVoxel {
                    position: [0, 0, 0],
                    material_idx: 1,
                    color_idx: 5,
                },
                VMaxVoxel {
                    position: [1, 1, 1],
                    material_idx: 0,
                    color_idx: 3,
                },
            ]),
            uuid: "u".to_owned(),
            v: 4,
            tools: None,
            brush: None,
            cam: None,
            pal: None,
        };

        let mut contents_files = BTreeMap::new();
        contents_files.insert("contents.vmaxb".to_owned(), contents);
        let mut palette_settings_files = BTreeMap::new();
        palette_settings_files.insert(
            "palette.settings.vmaxpsb".to_owned(),
            palette_settings(
                "mat",
                vec![
                    material("1", 0.0, 1.0, 0.0, true),
                    material("2", 0.5, 0.25, 2.0, false),
                ],
                Vec::new(),
            ),
        );
        let mut palette_png_files = BTreeMap::new();
        palette_png_files.insert("palette.png".to_owned(), palette_png());

        VMaxFile {
            scene_json_file,
            contents_files,
            palette_settings_files,
            palette_png_files,
            history_vmaxhb_files: BTreeMap::new(),
            history_vmaxhvsb_files: BTreeMap::new(),
            history_vmaxhvsc_files: BTreeMap::new(),
            selection_vmaxb_files: BTreeMap::new(),
            thumbnail_png: None,
            contents_vmax_pngs: BTreeMap::new(),
            group_pngs: BTreeMap::new(),
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let original = sample();
        let state = from_vmax_file(&original).unwrap();
        let rebuilt = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(rebuilt, original);
    }

    #[test]
    fn errors_without_voxel_max_ext() {
        let state = VoxState::default();
        assert!(to_vmax_file(&state, VoxelMaxColorFormat::Png).is_err());
    }
}
