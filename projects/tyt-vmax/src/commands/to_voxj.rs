use crate::{Dependencies, Result};
use clap::{Parser, ValueEnum};
use std::{
    collections::HashMap,
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};
use vmax::{VMaxObject, VMaxScene, VMaxVoxel};
use voxj::{
    AttrValue, PositionEncoding, SampleEncoding, VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject,
    VoxjPalette, VoxjTransform,
};
use voxj_codec::VoxelData;

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
    #[value(name = "pretty-json")]
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

    /// Output form: `json` (compact), `zip` (`.voxjz`), or `pretty-json`.
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

        // One geometry object and one wrapping-node transform per vmax object.
        let mut objects: Vec<VoxjObject> = Vec::new();
        let mut object_transforms: Vec<VoxjTransform> = Vec::new();
        for object in &scene.objects {
            let (voxj_object, transform) = self.build_object(
                &dependencies,
                object,
                encoding,
                &mut palettes,
                &mut palette_index,
            )?;
            objects.push(voxj_object);
            object_transforms.push(transform);
        }

        let (hierarchy_nodes, root_hierarchy_nodes) = build_hierarchy(&scene, &object_transforms);

        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                objects,
                palettes,
                hierarchy_nodes,
                root_hierarchy_nodes,
            },
        };

        let bytes = match self.format {
            Format::Json => voxj_codec::to_voxj_bytes(&file, false),
            Format::PrettyJson => voxj_codec::to_voxj_bytes(&file, true),
            Format::Zip => voxj_codec::to_voxjz_bytes(&file),
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
    ) -> Result<(VoxjObject, VoxjTransform)> {
        let voxels = if object.data.is_empty() {
            Vec::new()
        } else {
            let data_bytes = dependencies.read_file(&self.input_vmax.join(&object.data))?;
            dependencies.parse_voxels(&data_bytes)?
        };

        let Some(box_min) = min_corner(&voxels) else {
            // Empty object: no geometry to place or color.
            let empty = encode_voxj_object(
                object.name.clone(),
                Vec::new(),
                VoxelData {
                    positions: Vec::new(),
                    samples: Vec::new(),
                    bounds: [0, 0, 0],
                    palette_cell_counts: Vec::new(),
                },
                encoding,
            );
            return Ok((empty, object_transform(object, [0.0; 3])));
        };

        let bounds = object_bounds(&voxels, box_min);
        let positions: Vec<[u32; 3]> = voxels
            .iter()
            .map(|v| {
                [
                    (v.x - box_min[0]) as u32,
                    (v.y - box_min[1]) as u32,
                    (v.z - box_min[2]) as u32,
                ]
            })
            .collect();

        // Objects normally reference a palette; tolerate one that is absent.
        let color_palette = self.color_palette(dependencies, object, palettes, palette_index)?;
        let material_palette =
            self.material_palette(dependencies, object, palettes, palette_index)?;

        let mut palette_refs = Vec::new();
        let mut palette_cell_counts = Vec::new();
        let mut samples: Vec<Vec<u32>> = vec![Vec::new(); voxels.len()];
        if let Some((index, cell_count)) = color_palette {
            palette_refs.push(index);
            palette_cell_counts.push(cell_count);
            for (sample, voxel) in samples.iter_mut().zip(&voxels) {
                sample.push(voxel.color as u32);
            }
        }
        if let Some((index, cell_count)) = material_palette {
            palette_refs.push(index);
            palette_cell_counts.push(cell_count);
            for (sample, voxel) in samples.iter_mut().zip(&voxels) {
                sample.push(voxel.material as u32);
            }
        }

        let voxj_object = encode_voxj_object(
            object.name.clone(),
            palette_refs,
            VoxelData {
                positions,
                samples,
                bounds,
                palette_cell_counts,
            },
            encoding,
        );
        let box_min_f = [box_min[0] as f64, box_min[1] as f64, box_min[2] as f64];
        Ok((voxj_object, object_transform(object, box_min_f)))
    }

    /// Returns the shared color palette `(index, cell count)` for `object.palette`.
    /// When the `palette*.png` is present its pixels become the `rgba` cells; when
    /// it is absent (some Voxel Max packages keep colors only in the settings
    /// sidecar, which this converter does not read) a uniform placeholder palette
    /// is emitted so color indices are still preserved. Returns `None` only when
    /// the object names no palette at all.
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
        let data: Vec<Vec<AttrValue>> =
            match dependencies.read_file(&self.input_vmax.join(&object.palette)) {
                Ok(png_bytes) => dependencies
                    .load_palette(&png_bytes)?
                    .iter()
                    .map(|&[r, g, b, a]| {
                        vec![AttrValue::Text(format!("#{r:02X}{g:02X}{b:02X}{a:02X}"))]
                    })
                    .collect(),
                Err(_) => (0..COLOR_CELLS)
                    .map(|_| vec![AttrValue::Text(PLACEHOLDER_COLOR.to_owned())])
                    .collect(),
            };
        let cell_count = data.len();
        let index = palettes.len();
        palettes.push(VoxjPalette {
            attributes: vec!["rgba".to_owned()],
            data,
        });
        palette_index.insert(object.palette.clone(), index);
        Ok(Some((index, cell_count)))
    }

    /// Returns the shared material palette `(index, cell count)` for an object's
    /// `palette*.settings.vmaxpsb`, or `None` when that sidecar is absent.
    fn material_palette(
        &self,
        dependencies: &impl Dependencies,
        object: &VMaxObject,
        palettes: &mut Vec<VoxjPalette>,
        palette_index: &mut HashMap<String, usize>,
    ) -> Result<Option<(usize, usize)>> {
        let Some(stem) = object.palette.strip_suffix(".png") else {
            return Ok(None);
        };
        let name = format!("{stem}.settings.vmaxpsb");
        if let Some(&index) = palette_index.get(&name) {
            return Ok(Some((index, palettes[index].data.len())));
        }
        let Ok(bytes) = dependencies.read_file(&self.input_vmax.join(&name)) else {
            return Ok(None);
        };
        let materials = dependencies.parse_materials(&bytes)?;
        if materials.is_empty() {
            return Ok(None);
        }
        let data = materials
            .iter()
            .map(|m| {
                vec![
                    AttrValue::Number(m.metalness),
                    AttrValue::Number(m.roughness),
                    AttrValue::Number(m.emission),
                ]
            })
            .collect();
        let index = palettes.len();
        palettes.push(VoxjPalette {
            attributes: vec![
                "metallic".to_owned(),
                "roughness".to_owned(),
                "emissive".to_owned(),
            ],
            data,
        });
        palette_index.insert(name, index);
        Ok(Some((index, materials.len())))
    }
}

/// Dispatches to the codec's fixed or smallest-search encoder per `encoding`.
fn encode_voxj_object(
    name: String,
    palette_refs: Vec<usize>,
    data: VoxelData,
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
fn min_corner(voxels: &[VMaxVoxel]) -> Option<[i32; 3]> {
    voxels.iter().fold(None, |acc, v| {
        let acc = acc.unwrap_or([v.x, v.y, v.z]);
        Some([acc[0].min(v.x), acc[1].min(v.y), acc[2].min(v.z)])
    })
}

/// `[X, Y, Z]` bounds: the per-axis extent of `voxels` relative to `box_min`.
fn object_bounds(voxels: &[VMaxVoxel], box_min: [i32; 3]) -> [u32; 3] {
    let mut bounds = [1u32; 3];
    for v in voxels {
        bounds[0] = bounds[0].max((v.x - box_min[0] + 1) as u32);
        bounds[1] = bounds[1].max((v.y - box_min[1] + 1) as u32);
        bounds[2] = bounds[2].max((v.z - box_min[2] + 1) as u32);
    }
    bounds
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

/// Converts a Voxel Max axis-angle rotation `[x, y, z, angle]` to a unit
/// quaternion `[x, y, z, w]`.
fn axis_angle_to_quat(axis_angle: [f64; 4]) -> [f64; 4] {
    let [ax, ay, az, angle] = axis_angle;
    let length = (ax * ax + ay * ay + az * az).sqrt();
    if length < 1e-12 || angle == 0.0 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let half = angle / 2.0;
    let s = half.sin() / length;
    [ax * s, ay * s, az * s, half.cos()]
}

/// Rotates `v` by the unit quaternion `q = [x, y, z, w]`.
fn quat_rotate(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let [qx, qy, qz, qw] = q;
    let [vx, vy, vz] = v;
    let tx = 2.0 * (qy * vz - qz * vy);
    let ty = 2.0 * (qz * vx - qx * vz);
    let tz = 2.0 * (qx * vy - qy * vx);
    [
        vx + qw * tx + (qy * tz - qz * ty),
        vy + qw * ty + (qz * tx - qx * tz),
        vz + qw * tz + (qx * ty - qy * tx),
    ]
}

/// Builds the voxj hierarchy: one node per group and one per object (the latter
/// wrapping its geometry). Returns the nodes and the indices of the roots.
fn build_hierarchy(
    scene: &VMaxScene,
    object_transforms: &[VoxjTransform],
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
            child_objects: vec![object_index],
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
