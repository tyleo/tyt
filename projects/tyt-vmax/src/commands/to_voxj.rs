use crate::{Dependencies, Result, axis_angle_to_quat, quat_rotate};
use clap::{Parser, ValueEnum};
use std::{
    collections::HashMap,
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};
use vmax::{VMaxObject, VMaxSceneJsonFile};
use vmax_codec::Voxel;
use voxj::{
    VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject, VoxjPalette, VoxjTransform, VoxjValue,
};
use voxj_codec::{
    ObjectData, PositionEncoding, SampleEncoding, to_voxj_bytes, to_voxj_pretty_bytes,
    to_voxjz_bytes,
};

/// Number of cells in a color palette; a `palette*.png` is 256×1 RGBA, and a
/// placeholder palette covers every possible color index.
const COLOR_CELLS: usize = 256;

/// Uniform color used for every cell of a placeholder palette when an object's
/// `palette*.png` is absent, so color indices are preserved even though the
/// actual colors are unknown.
const PLACEHOLDER_COLOR: &str = "#FFFFFFFF";

/// Output container and printing form.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Format {
    /// Compact `.voxj` JSON.
    #[value(name = "json")]
    Json,
    /// Compressed `.voxjz` zip archive.
    #[value(name = "zip")]
    Zip,
    /// Pretty-printed `.voxj` JSON.
    #[value(name = "pretty")]
    PrettyJson,
}

/// Automatic encoding strategy that picks the block encodings.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Optimize {
    /// Try every non-raw encoding pairing and keep the smallest.
    #[value(name = "size")]
    Size,
    /// Fast to decode: bitmap positions and packed samples.
    #[value(name = "fast")]
    Fast,
    /// Most readable: raw positions and raw samples.
    #[value(name = "pretty")]
    Pretty,
}

/// CLI choice for the position-block encoding.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum PositionEncodingArg {
    #[value(name = "raw-json")]
    RawJson,
    #[value(name = "bitmap-base64")]
    BitmapBase64,
    #[value(name = "hilbert_index-delta-varint-base64")]
    Hilbert,
}

impl From<PositionEncodingArg> for PositionEncoding {
    fn from(value: PositionEncodingArg) -> Self {
        match value {
            PositionEncodingArg::RawJson => PositionEncoding::RawJson,
            PositionEncodingArg::BitmapBase64 => PositionEncoding::BitmapBase64,
            PositionEncodingArg::Hilbert => PositionEncoding::Hilbert,
        }
    }
}

/// CLI choice for the sample-block encoding.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum SampleEncodingArg {
    #[value(name = "raw-json")]
    RawJson,
    #[value(name = "rle-json")]
    RleJson,
    #[value(name = "packed-base64")]
    PackedBase64,
}

impl From<SampleEncodingArg> for SampleEncoding {
    fn from(value: SampleEncodingArg) -> Self {
        match value {
            SampleEncodingArg::RawJson => SampleEncoding::RawJson,
            SampleEncodingArg::RleJson => SampleEncoding::RleJson,
            SampleEncodingArg::PackedBase64 => SampleEncoding::PackedBase64,
        }
    }
}

/// The resolved encoding strategy for an object: either fixed block encodings or
/// the smallest-deflated search.
#[derive(Clone, Copy)]
enum Encoding {
    Fixed {
        position: PositionEncoding,
        sample: SampleEncoding,
    },
    Smallest,
}

/// Converts a `.vmax` package to a Voxel Json document, written to stdout.
///
/// `--format` chooses the output form (compact `.voxj`, `.voxjz` zip, or
/// pretty-printed `.voxj`). The block encodings come from `--position-encoding`
/// and `--sample-encoding`, or from `--optimize` (which picks them
/// automatically and may not be combined with the explicit encoding flags).
#[derive(Clone, Debug, Parser)]
#[command(name = "to-voxj")]
pub struct ToVoxj {
    /// The input `.vmax` directory to convert.
    #[arg(value_name = "input-vmax")]
    input_vmax: PathBuf,

    /// Output form: `json` (compact), `zip` (`.voxjz`), or `pretty`.
    #[arg(value_name = "format", long, default_value = "json")]
    format: Format,

    /// Position-block encoding. Ignored when `--optimize` is given.
    #[arg(
        value_name = "position-encoding",
        long,
        default_value = "bitmap-base64",
        conflicts_with = "optimize"
    )]
    position_encoding: PositionEncodingArg,

    /// Sample-block encoding. Ignored when `--optimize` is given.
    #[arg(
        value_name = "sample-encoding",
        long,
        default_value = "rle-json",
        conflicts_with = "optimize"
    )]
    sample_encoding: SampleEncodingArg,

    /// Automatically choose encodings: `size`, `fast`, or `pretty`. Cannot be
    /// combined with `--position-encoding`/`--sample-encoding`.
    #[arg(value_name = "optimize", long)]
    optimize: Option<Optimize>,
}

impl ToVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let scene_bytes = dependencies.read_file(&self.input_vmax.join("scene.json"))?;
        let scene = dependencies.parse_scene(&scene_bytes)?;

        let encoding = match self.optimize {
            Some(Optimize::Size) => Encoding::Smallest,
            Some(Optimize::Fast) => Encoding::Fixed {
                position: PositionEncoding::BitmapBase64,
                sample: SampleEncoding::PackedBase64,
            },
            Some(Optimize::Pretty) => Encoding::Fixed {
                position: PositionEncoding::RawJson,
                sample: SampleEncoding::RawJson,
            },
            None => Encoding::Fixed {
                position: self.position_encoding.into(),
                sample: self.sample_encoding.into(),
            },
        };

        // Palettes are shared across objects and deduped by source filename.
        let mut palettes: Vec<VoxjPalette> = Vec::new();
        let mut palette_index: HashMap<String, usize> = HashMap::new();
        // Display name of each material palette, keyed by its `palettes` index.
        let mut palette_name_by_index: HashMap<usize, String> = HashMap::new();

        // One voxj object (and its `.vmaxb` bytes for the ext) per distinct
        // geometry; instances of one geometry collapse to a single object.
        let mut objects: Vec<VoxjObject> = Vec::new();
        let mut object_vmaxb: Vec<Option<Vec<u8>>> = Vec::new();
        // Per scene object: its node transform and the voxj object it places.
        let mut object_transforms: Vec<VoxjTransform> = Vec::new();
        let mut object_refs: Vec<usize> = Vec::new();
        let mut instances: HashMap<InstanceKey, usize> = HashMap::new();
        for object in &scene.objects {
            let key = instance_key(object);
            if let Some(&existing) = key.as_ref().and_then(|key| instances.get(key)) {
                // Reuse the built geometry; keep this placement, skip re-decoding.
                let (box_min, _) =
                    authored_box(object).expect("instance key implies authored bounds");
                let box_min_f = [box_min[0] as f64, box_min[1] as f64, box_min[2] as f64];
                object_transforms.push(object_transform(object, box_min_f));
                object_refs.push(existing);
                continue;
            }
            let (voxj_object, transform, vmaxb) = self.build_object(
                &dependencies,
                object,
                encoding,
                &mut palettes,
                &mut palette_index,
                &mut palette_name_by_index,
            )?;
            let index = objects.len();
            objects.push(voxj_object);
            object_vmaxb.push(vmaxb);
            object_transforms.push(transform);
            object_refs.push(index);
            if let Some(key) = key {
                instances.insert(key, index);
            }
        }

        let (hierarchy_nodes, root_hierarchy_nodes) =
            build_hierarchy(&scene, &object_transforms, &object_refs);

        // Material-palette names aligned by index with `main.palettes` for the ext.
        let palette_names: Vec<Option<String>> = (0..palettes.len())
            .map(|i| palette_name_by_index.get(&i).cloned())
            .collect();

        // Stash the vmax-specific state that has no native voxj home under the
        // generic `ext` namespace so `from-voxj` can rebuild the package exactly.
        // The voxj codec treats `ext` as opaque; the `voxel-max` shape lives here.
        let ext = dependencies.voxel_max_ext(&scene_bytes, &palette_names, &object_vmaxb)?;

        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                objects,
                palettes,
                hierarchy_nodes,
                root_hierarchy_nodes,
                ext: Some(ext),
            },
        };

        let bytes = match self.format {
            Format::Json => to_voxj_bytes(&file),
            Format::PrettyJson => to_voxj_pretty_bytes(&file),
            Format::Zip => to_voxjz_bytes(&file),
        }
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

        dependencies.write_stdout(&bytes)?;
        Ok(())
    }

    /// Decodes one object's voxels and palettes and returns its geometry plus
    /// the transform of the hierarchy node that places it.
    fn build_object(
        &self,
        dependencies: &impl Dependencies,
        object: &VMaxObject,
        encoding: Encoding,
        palettes: &mut Vec<VoxjPalette>,
        palette_index: &mut HashMap<String, usize>,
        palette_name_by_index: &mut HashMap<usize, String>,
    ) -> Result<(VoxjObject, VoxjTransform, Option<Vec<u8>>)> {
        let (voxels, vmaxb) = if object.data.is_empty() {
            (Vec::new(), None)
        } else {
            let data_bytes = dependencies.read_file(&self.input_vmax.join(&object.data))?;
            let voxels = dependencies.parse_voxels(&data_bytes)?;
            (voxels, Some(data_bytes))
        };

        // Always use the authored box (`e_c + e_mi` .. `e_c + e_ma`) when the
        // object has Voxel Max bounds, so the encoded bounds match vmax exactly
        // (even for empty objects); only objects without authored bounds use the
        // tight extent of their voxels.
        let (box_min, bounds) = object_box(object, &voxels, min_corner(&voxels))?;
        let box_min_f = [box_min[0] as f64, box_min[1] as f64, box_min[2] as f64];

        if voxels.is_empty() {
            // No geometry to place or color, but keep the authored bounds.
            let empty = encode_voxj_object(
                object.name.clone(),
                Vec::new(),
                ObjectData {
                    positions: Vec::new(),
                    samples: Vec::new(),
                    bounds,
                    palette_cell_counts: Vec::new(),
                },
                encoding,
            );
            return Ok((empty, object_transform(object, box_min_f), vmaxb));
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
        let color_palette = self.color_palette(dependencies, object, palettes, palette_index)?;
        let material_palette = self.material_palette(
            dependencies,
            object,
            palettes,
            palette_index,
            palette_name_by_index,
        )?;

        let mut palette_refs = Vec::new();
        let mut palette_cell_counts = Vec::new();
        let mut samples: Vec<Vec<u32>> = vec![Vec::new(); voxels.len()];
        if let Some((index, cell_count)) = color_palette {
            palette_refs.push(index);
            palette_cell_counts.push(cell_count);
            for (sample, voxel) in samples.iter_mut().zip(&voxels) {
                sample.push(voxel.color_idx as u32);
            }
        }
        if let Some((index, cell_count)) = material_palette {
            palette_refs.push(index);
            palette_cell_counts.push(cell_count);
            for (sample, voxel) in samples.iter_mut().zip(&voxels) {
                sample.push(voxel.material_idx as u32);
            }
        }

        let voxj_object = encode_voxj_object(
            object.name.clone(),
            palette_refs,
            ObjectData {
                positions,
                samples,
                bounds,
                palette_cell_counts,
            },
            encoding,
        );
        Ok((voxj_object, object_transform(object, box_min_f), vmaxb))
    }

    /// Returns the shared color palette `(index, cell count)` for `object.palette`.
    /// The `rgba` cells come from the `palette*.png` image when present, otherwise
    /// from the `colors` table of the `palette*.settings.vmaxpsb` sidecar (where
    /// Voxel Max keeps colors when no image is written); a uniform placeholder is
    /// emitted only when neither source carries color, so color indices are still
    /// preserved. Returns `None` only when the object names no palette at all.
    fn color_palette(
        &self,
        dependencies: &impl Dependencies,
        object: &VMaxObject,
        palettes: &mut Vec<VoxjPalette>,
        palette_index: &mut HashMap<String, usize>,
    ) -> Result<Option<(usize, usize)>> {
        if object.palette.is_empty() {
            return Ok(None);
        }
        if let Some(&index) = palette_index.get(&object.palette) {
            return Ok(Some((index, palettes[index].data.len())));
        }
        let data = self.color_cells(dependencies, object)?;
        let cell_count = data.len();
        let index = palettes.len();
        palettes.push(VoxjPalette {
            attributes: vec!["rgba".to_owned()],
            data,
        });
        palette_index.insert(object.palette.clone(), index);
        Ok(Some((index, cell_count)))
    }

    /// The `rgba` cells for an object's color palette: the `palette*.png` pixels
    /// when that image is present, else the RGBA `colors` table of the
    /// `palette*.settings.vmaxpsb` sidecar, and finally a uniform placeholder
    /// when neither source carries color.
    fn color_cells(
        &self,
        dependencies: &impl Dependencies,
        object: &VMaxObject,
    ) -> Result<Vec<Vec<VoxjValue>>> {
        if let Ok(png_bytes) = dependencies.read_file(&self.input_vmax.join(&object.palette)) {
            return Ok(rgba_cells(&strip_color_terminator(
                dependencies.load_palette(&png_bytes)?,
            )));
        }
        if let Some(stem) = object.palette.strip_suffix(".png") {
            let sidecar = format!("{stem}.settings.vmaxpsb");
            if let Ok(bytes) = dependencies.read_file(&self.input_vmax.join(&sidecar)) {
                let palette = dependencies.parse_material_palette(&bytes)?;
                if !palette.colors.is_empty() {
                    return Ok(rgba_cells(&strip_color_terminator(palette.colors)));
                }
            }
        }
        Ok((0..COLOR_CELLS)
            .map(|_| vec![VoxjValue::Text(PLACEHOLDER_COLOR.to_owned())])
            .collect())
    }

    /// Returns the shared material palette `(index, cell count)` for an object's
    /// `palette*.settings.vmaxpsb`, or `None` when that sidecar is absent. The
    /// per-slot material properties (metalness/roughness/emission/shadows) become
    /// native palette cells; the palette's display name is recorded in
    /// `palette_name_by_index` for the `voxel-max` ext.
    fn material_palette(
        &self,
        dependencies: &impl Dependencies,
        object: &VMaxObject,
        palettes: &mut Vec<VoxjPalette>,
        palette_index: &mut HashMap<String, usize>,
        palette_name_by_index: &mut HashMap<usize, String>,
    ) -> Result<Option<(usize, usize)>> {
        let Some(stem) = object.palette.strip_suffix(".png") else {
            return Ok(None);
        };
        let sidecar = format!("{stem}.settings.vmaxpsb");
        if let Some(&index) = palette_index.get(&sidecar) {
            return Ok(Some((index, palettes[index].data.len())));
        }
        let Ok(bytes) = dependencies.read_file(&self.input_vmax.join(&sidecar)) else {
            return Ok(None);
        };
        let palette = dependencies.parse_material_palette(&bytes)?;
        if palette.materials.is_empty() {
            return Ok(None);
        }
        // Dispersion columns (`ior`/`transmission`/`absorption`) are added only
        // when some slot carries an `md` block, so palettes without dispersion
        // stay exactly as before. Slots lacking `md` then fill those columns
        // with `null` so every row spans every attribute.
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
        palette_name_by_index.insert(index, palette.name);
        Ok(Some((index, palette.materials.len())))
    }
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

/// Dispatches to the codec's fixed or smallest-search encoder per `encoding`.
fn encode_voxj_object(
    name: String,
    palette_refs: Vec<usize>,
    data: ObjectData,
    encoding: Encoding,
) -> VoxjObject {
    match encoding {
        Encoding::Fixed { position, sample } => {
            voxj_codec::encode_object(name, palette_refs, data, position, sample)
        }
        Encoding::Smallest => voxj_codec::encode_object_smallest(name, palette_refs, data),
    }
}

/// The minimum `[x, y, z]` corner over `voxels`, or `None` when empty.
fn min_corner(voxels: &[Voxel]) -> Option<[i32; 3]> {
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
fn object_bounds(voxels: &[Voxel], box_min: [i32; 3]) -> [u32; 3] {
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
    voxels: &[Voxel],
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
/// origin: `position = t_p + R·S·(box_min − e_c)`, `rotation = quat(t_r)`,
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
