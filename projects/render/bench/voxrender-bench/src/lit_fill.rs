use crate::{Error, Result, TileMasks};
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{TyVector3F64, TyVector3U32};
use voxcore::{BVoxObject, material};
use voxrender::{RenderScene, RenderViewRays, grid_vector, quantize_direction};
use voxrender_wgpu::{GRID_WALK_WGSL, dense_cells, placement_bytes};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor,
    Device, PipelineCompilationOptions, Queue, ShaderModuleDescriptor, ShaderRuntimeChecks,
    ShaderSource,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// The record cells one dispatch fills. A dispatch then ends well within a
/// frame's time on a phone's GPU.
const BATCH: u32 = 1 << 16;

/// The meters a caster's box grows by before the test against a tile's.
/// The margin covers the rounding of the boxes and of each grid's light.
const MARGIN: f64 = 0.01;

/// A scene's corner shadows under one directional light, filled on the
/// GPU: [`LitCorners`](crate::LitCorners)' words in a buffer.
///
/// A compute pass casts the contract's ray from each corner of each face
/// the light can reach, through voxrender's integer walk. A ray walks
/// only the placements that can shadow its tile: those whose box, seen
/// along the light, covers some of the tile's and reaches as far toward
/// the light.
///
/// A material that passes light tints the light a ray carries through it,
/// as the contract's shadow ray pays each surface. A corner is lit where
/// some light reaches it. A record cell's tint holds the mean of what
/// reaches its lit corners in each channel.
pub struct LitFill {
    /// The first word of each placement's tiles' words.
    pub places: Vec<u32>,

    /// A word for each record cell, then [`TileMasks::SPARE`] spare words.
    /// The fill writes a word's low 24 bits and keeps the rest, which a
    /// caller can write beforehand. With [`tints`](Self::tints), as many
    /// words follow, which the fill writes whole: each record cell's
    /// tint, 8 bits a channel from the low bits up.
    pub lit: Buffer,

    /// How far past a record cell's word in [`lit`](Self::lit) its tint
    /// lies, in a scene with a material that passes light.
    pub tints: Option<u32>,

    pipeline: ComputePipeline,

    bind_group: BindGroup,

    /// The scene's record cells and the first of them a dispatch fills.
    frame: Buffer,

    cells: u32,
}

impl LitFill {
    /// Uploads what the fill of `scene` reads, for a light along the unit
    /// `toward`. Errors unless the scene has a view, or if a placement
    /// lies out of its rays' range.
    ///
    /// # Arguments
    ///
    /// 1. `tiles`: each object's non-empty tiles in the order of their mask
    ///    words, as a tile's low corner and the indices of its record
    ///    cells
    /// 2. `opaque`: whether a material id passes no light
    pub fn new(
        device: &Device,
        scene: &RenderScene,
        toward: TyVector3F64,
        tiles: &HashMap<U32Id<BVoxObject>, Vec<(TyVector3U32, Vec<u16>)>>,
        opaque: &dyn Fn(u32) -> bool,
    ) -> Result<Self> {
        // The walk's placements carry a view's rays, which the fill leaves
        // unread.
        let (_, view) = scene.iter_views().next().ok_or(Error::NoView)?;

        let grid_rays = RenderViewRays::new(view, 1, 1)
            .to_scene_grid_rays(scene)
            .map_err(Error::Render)?;

        let mut dense = Vec::new();
        let mut offsets = HashMap::new();

        for (object_id, object) in scene.iter_objects() {
            offsets.insert(
                object_id,
                u32::try_from(dense.len()).expect("a scene's cells number under 2^32"),
            );

            dense.extend(dense_cells(object));
        }

        // `lit_fill.wgsl`'s `passes`: three `vec4<f32>`s for each value up
        // to the greatest a cell holds, its material's id plus one. An id
        // the scene lacks passes the light whole.
        let keys = dense.iter().max().map_or(1, |greatest| greatest + 1);

        let passes: Vec<f32> = (0..keys)
            .flat_map(|key| {
                let id = key.checked_sub(1);
                let blocks = if id.is_some_and(opaque) { 1.0 } else { 0.0 };

                id.and_then(|id| scene.material(U32Id::from_u32(id)))
                    .map_or(
                        [
                            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, blocks, 0.0, 0.0, 0.0, 0.0,
                        ],
                        |material| {
                            let base = material.base_color;
                            let most = base.color
                                * (base.alpha * material.transmission * (1.0 - material.metallic));
                            let reflectance = material::normal_reflectance(
                                base.color,
                                material.metallic,
                                material.ior,
                            );

                            // The reflectance rises toward grazing as far as
                            // the roughness lets it.
                            let smooth = 1.0 - material.roughness;
                            let rise = |at_normal: f64| smooth.max(at_normal) - at_normal;

                            [
                                most.red,
                                most.green,
                                most.blue,
                                1.0 - base.alpha,
                                reflectance.red,
                                reflectance.green,
                                reflectance.blue,
                                blocks,
                                rise(reflectance.red),
                                rise(reflectance.green),
                                rise(reflectance.blue),
                                0.0,
                            ]
                        },
                    )
            })
            .map(|value| value as f32)
            .collect();

        let tinted = scene.iter_materials().any(|(id, _)| !opaque(id.to_u32()));

        let cell_offsets: Vec<_> = scene
            .iter_placements()
            .map(|(_, placement)| offsets[&placement.object_id])
            .collect();

        let placement_bytes = placement_bytes(scene, &grid_rays, &cell_offsets);

        // `lit_fill.wgsl`'s `Light` for each placement.
        let lights: Vec<u8> = scene
            .iter_placements()
            .flat_map(|(_, placement)| {
                let [x, y, z] = quantize_direction(grid_vector(&placement.transform, toward));

                let fades = [TyVector3F64::X, TyVector3F64::Y, TyVector3F64::Z].map(|axis| {
                    let cosine = (placement.transform.rotation * axis).dot(toward).abs();

                    (1.0 - cosine).powi(5) as f32
                });

                [x, y, z, 0]
                    .into_iter()
                    .flat_map(i32::to_le_bytes)
                    .chain(fades.into_iter().chain([0.0]).flat_map(f32::to_le_bytes))
            })
            .collect();

        // Each box below spans the light's axis and two axes across it.
        let across = toward.cross(if toward.x.abs() < 0.5 {
            TyVector3F64::X
        } else {
            TyVector3F64::Y
        });
        let across = across.normalize();
        let axes = [across, toward.cross(across), toward];

        // The box around the grid box from `low` to `high` of a placement.
        let boxed = |index: usize, low: TyVector3U32, high: TyVector3U32| {
            let (_, placement) = scene
                .iter_placements()
                .nth(index)
                .expect("the placement is the scene's");

            let mut least = [f64::INFINITY; 3];
            let mut most = [f64::NEG_INFINITY; 3];

            for corner in 0..8u32 {
                let far = TyVector3U32::new(corner & 1, corner >> 1 & 1, corner >> 2);
                let cell = low + (high - low) * far;
                let point = placement.transform.transform_point(cell.as_dvec3());

                for (axis, direction) in axes.iter().enumerate() {
                    let along = direction.dot(point);

                    least[axis] = least[axis].min(along);
                    most[axis] = most[axis].max(along);
                }
            }

            (least, most)
        };

        let bounds: Vec<_> = scene
            .iter_placements()
            .map(|(_, placement)| {
                scene
                    .object(placement.object_id)
                    .expect("a placement's object is one of the scene's")
                    .bounds()
            })
            .collect();

        let casters_of: Vec<_> = (0..bounds.len())
            .map(|index| boxed(index, TyVector3U32::ZERO, bounds[index]))
            .collect();

        let mut places = Vec::new();
        let mut tile_words: Vec<u32> = Vec::new();
        let mut casters: Vec<u32> = Vec::new();
        let mut recorded: Vec<u32> = Vec::new();

        for (index, (_, placement)) in scene.iter_placements().enumerate() {
            places.push(
                u32::try_from(recorded.len()).expect("a scene's lit words number under 2^32"),
            );

            for (corner, cells) in &tiles[&placement.object_id] {
                let tile = u32::try_from(tile_words.len() / 8)
                    .ok()
                    .filter(|&tile| tile < 1 << 20)
                    .expect("a scene's placed tiles number under 2^20");

                let (least, most) = boxed(index, *corner, (*corner + TILE).min(bounds[index]));
                let first = casters.len();

                // A caster's box covers some of the tile's across the light
                // and reaches as far toward it.
                casters.extend(
                    (0u32..)
                        .zip(&casters_of)
                        .filter_map(|(caster, (low, high))| {
                            let covers = (0..2).all(|axis| {
                                low[axis] - MARGIN <= most[axis]
                                    && high[axis] + MARGIN >= least[axis]
                            });

                            (covers && high[2] + MARGIN >= least[2]).then_some(caster)
                        }),
                );

                let count = |value: usize| {
                    u32::try_from(value).expect("a scene's casters number under 2^32")
                };

                tile_words.extend([
                    corner.x,
                    corner.y,
                    corner.z,
                    count(index),
                    count(first),
                    count(casters.len() - first),
                    0,
                    0,
                ]);

                recorded.extend(cells.iter().map(|&cell| tile << 12 | u32::from(cell)));
            }
        }

        let cells = u32::try_from(recorded.len()).expect("a scene's lit words number under 2^32");

        // A binding cannot be empty.
        for words in [&mut tile_words, &mut casters, &mut recorded] {
            if words.is_empty() {
                words.resize(8, 0);
            }
        }

        let init = |label, contents: &[u8], usage| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
        };

        let bytes = |words: &[u32]| -> Vec<u8> {
            words.iter().flat_map(|word| word.to_le_bytes()).collect()
        };

        let tints = tinted.then_some(cells + TileMasks::SPARE);

        let frame = init(
            "lit fill frame",
            &bytes(&[cells, 0, tints.unwrap_or(0), 0]),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let placements = init(
            "lit fill placements",
            &placement_bytes,
            BufferUsages::STORAGE,
        );
        let dense = init("lit fill cells", &bytes(&dense), BufferUsages::STORAGE);
        let lights = init("lit fill lights", &lights, BufferUsages::STORAGE);
        let passes = init(
            "lit fill passes",
            &passes
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<_>>(),
            BufferUsages::STORAGE,
        );
        let tile_words = init("lit fill tiles", &bytes(&tile_words), BufferUsages::STORAGE);
        let casters = init("lit fill casters", &bytes(&casters), BufferUsages::STORAGE);
        let recorded = init(
            "lit fill record cells",
            &bytes(&recorded),
            BufferUsages::STORAGE,
        );

        let lit = device.create_buffer(&BufferDescriptor {
            label: Some("lit corners"),
            size: u64::from(cells + TileMasks::SPARE) * if tinted { 8 } else { 4 },
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // SAFETY: an invocation past the scene's record cells returns. A
        // record cell indexes its placed tile, which indexes its placement
        // and its casters. The walk reads only its placement's cells, and
        // each of its loops ends within a grid's extent. A face's front
        // cell reads the cells only inside the grid. A cell's value
        // indexes `passes`, which holds each value up to the greatest.
        let module = unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some("lit fill"),
                    source: ShaderSource::Wgsl(
                        format!("{GRID_WALK_WGSL}\n{}", include_str!("lit_fill.wgsl")).into(),
                    ),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("lit fill"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("lit fill"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                &frame,
                &placements,
                &dense,
                &lights,
                &tile_words,
                &casters,
                &recorded,
                &lit,
                &passes,
            ]
            .iter()
            .zip(0u32..)
            .map(|(buffer, binding)| BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            })
            .collect::<Vec<_>>(),
        });

        Ok(LitFill {
            places,
            lit,
            tints,
            pipeline,
            bind_group,
            frame,
            cells,
        })
    }

    /// Submits the fill of every record cell with a dispatch for each batch
    /// of them.
    pub fn fill(&self, device: &Device, queue: &Queue) {
        for first in (0..self.cells).step_by(BATCH as usize) {
            // A write lands before the commands of the submit after it.
            queue.write_buffer(
                &self.frame,
                0,
                &[self.cells, first]
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>(),
            );

            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("lit fill"),
            });

            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("lit fill"),
                    timestamp_writes: None,
                });

                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.dispatch_workgroups((self.cells - first).min(BATCH).div_ceil(64), 1, 1);
            }

            queue.submit([encoder.finish()]);
        }
    }
}
