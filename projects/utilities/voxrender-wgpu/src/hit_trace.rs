use crate::{Error, GRID_WALK_WGSL, PixelHit, Result, dense_cells, placement_bytes, push_i32s};
use branded_id::U32Id;
use std::{collections::HashMap, sync::mpsc};
use voxrender::{
    BRenderView, Error as RenderError, RenderObject, RenderScene, RenderViewRays, ShadowTarget,
    grid_point, grid_vector, quantize_direction, quantize_point,
};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, MapMode, PipelineCompilationOptions, PollType, Queue,
    ShaderModuleDescriptor, ShaderSource,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The shader's marker for a pixel without a hit.
const NO_HIT: u32 = u32::MAX;

/// The bytes of one hit in the shader's output.
const HIT_BYTES: u64 = 16;

/// One view's first-hit walk over a scene, uploaded once and dispatched as
/// often as its caller records it.
pub struct HitTrace {
    pipeline: ComputePipeline,

    bind_group: BindGroup,

    hits: Buffer,

    shadows: Buffer,

    width: u32,

    height: u32,

    lights: usize,
}

impl HitTrace {
    /// Uploads `scene` and the rays of `view_id` over a `width` by `height`
    /// image, and builds the pipeline of `voxrender`'s integer walk over each
    /// object's [`dense_cells`].
    pub fn new(
        device: &Device,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        Self::with_walk(
            device,
            scene,
            view_id,
            width,
            height,
            include_str!("integer_walk.wgsl"),
            dense_cells,
        )
    }

    /// As [`HitTrace::new`], tracing with `walk` in place of the integer
    /// walk. `walk` is WGSL defining
    /// `fn walk(placement: Placement, ray: Ray) -> Hit` over
    /// [`GRID_WALK_WGSL`] and `first_hit.wgsl`'s helpers. It reads the words
    /// `cells` gives each object.
    pub fn with_walk(
        device: &Device,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        width: u32,
        height: u32,
        walk: &str,
        cells: impl Fn(&RenderObject) -> Vec<u32>,
    ) -> Result<Self> {
        let view = scene
            .view(view_id)
            .ok_or(Error::Render(RenderError::UnknownView { view_id }))?;

        let grid_rays = RenderViewRays::new(view, width, height)
            .to_scene_grid_rays(scene)
            .map_err(Error::Render)?;

        let targets: Vec<_> = scene
            .iter_lights()
            .filter_map(|(_, light)| light.shadow_target())
            .collect();

        let mut words = Vec::new();
        let mut offsets = HashMap::new();

        for (object_id, object) in scene.iter_objects() {
            offsets.insert(
                object_id,
                u32::try_from(words.len()).expect("a scene's cells number under 2^32"),
            );

            words.extend(cells(object));
        }

        let cell_offsets: Vec<_> = scene
            .iter_placements()
            .map(|(_, placement)| offsets[&placement.object_id])
            .collect();

        let placements = placement_bytes(scene, &grid_rays, &cell_offsets);

        let mut lights = Vec::new();

        for target in &targets {
            for (placement_id, placement) in scene.iter_placements() {
                let transform = &placement.transform;

                match *target {
                    ShadowTarget::Direction(direction) => {
                        push_i32s(
                            &mut lights,
                            quantize_direction(grid_vector(transform, direction)),
                        );
                    }

                    ShadowTarget::Position(position) => {
                        let bounds = scene
                            .object(placement.object_id)
                            .expect("a placement's object is one of the scene's")
                            .bounds();

                        let point = quantize_point(grid_point(transform, position), bounds)
                            .ok_or(Error::Render(RenderError::GridRange { placement_id }))?;

                        for value in point.into_iter().chain([1]) {
                            lights.extend(value.to_le_bytes());
                        }
                    }
                }
            }
        }

        let placement_count = u32::try_from(grid_rays.len()).expect("placements number under 2^32");
        let light_count = u32::try_from(targets.len()).expect("lights number under 2^32");

        let mut frame = Vec::new();

        for value in [width, height, placement_count, light_count] {
            frame.extend(value.to_le_bytes());
        }

        let init = |label, contents: &[u8], usage| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
        };

        let frame = init("frame", &frame, BufferUsages::UNIFORM);
        let placements = init("placements", &placements, BufferUsages::STORAGE);
        let cells = init(
            "cells",
            &words
                .iter()
                .flat_map(|cell: &u32| cell.to_le_bytes())
                .collect::<Vec<_>>(),
            BufferUsages::STORAGE,
        );

        // A binding cannot be empty.
        let lights = init(
            "lights",
            if lights.is_empty() { &[0; 16] } else { &lights },
            BufferUsages::STORAGE,
        );

        let pixels = u64::from(width) * u64::from(height);

        let hits = output(device, "hits", pixels * HIT_BYTES);
        let shadows = output(
            device,
            "shadows",
            (pixels * u64::from(light_count)).max(1) * 4,
        );

        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("first_hit"),
            source: ShaderSource::Wgsl(
                format!(
                    "{GRID_WALK_WGSL}\n{}\n{walk}",
                    include_str!("first_hit.wgsl")
                )
                .into(),
            ),
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("first_hit"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("first_hit"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: frame.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: placements.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: cells.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: lights.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: hits.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: shadows.as_entire_binding(),
                },
            ],
        });

        Ok(HitTrace {
            pipeline,
            bind_group,
            hits,
            shadows,
            width,
            height,
            lights: targets.len(),
        })
    }

    /// Records the walk over every pixel into `pass`.
    pub fn dispatch(&self, pass: &mut ComputePass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.width.div_ceil(8), self.height.div_ceil(8), 1);
    }

    /// Walks once and reads back the first hit through each pixel, row by
    /// row.
    pub fn trace(&self, device: &Device, queue: &Queue) -> Result<Vec<Option<PixelHit>>> {
        let hit_readback = readback(device, "hit readback", self.hits.size());
        let shadow_readback = readback(device, "shadow readback", self.shadows.size());

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("first_hit"),
        });

        self.dispatch(&mut encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("first_hit"),
            timestamp_writes: None,
        }));

        encoder.copy_buffer_to_buffer(&self.hits, 0, &hit_readback, 0, None);
        encoder.copy_buffer_to_buffer(&self.shadows, 0, &shadow_readback, 0, None);
        queue.submit([encoder.finish()]);

        let hits = read_words(device, &hit_readback)?;
        let shadows = read_words(device, &shadow_readback)?;
        let lights = self.lights;

        Ok(hits
            .chunks_exact(4)
            .enumerate()
            .map(|(pixel, hit)| {
                (hit[0] != NO_HIT).then(|| PixelHit {
                    placement: hit[0],
                    voxel: hit[1],
                    face: hit[2],
                    lit: shadows[pixel * lights..(pixel + 1) * lights]
                        .iter()
                        .map(|&bits| u8::try_from(bits).expect("a light has four corner bits"))
                        .collect(),
                })
            })
            .collect())
    }
}

/// A storage buffer of `size` bytes the shader writes.
fn output(device: &Device, label: &str, size: u64) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}

/// A buffer of `size` bytes to copy an output into and map.
fn readback(device: &Device, label: &str, size: u64) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The words of `buffer` once the GPU has written it.
fn read_words(device: &Device, buffer: &Buffer) -> Result<Vec<u32>> {
    let (sender, receiver) = mpsc::channel();

    buffer.map_async(MapMode::Read, .., move |result| {
        sender.send(result).expect("the receiver outlives the wait");
    });

    device
        .poll(PollType::wait_indefinitely())
        .map_err(Error::Poll)?;

    receiver
        .recv()
        .expect("the map callback runs during the wait")
        .map_err(Error::Map)?;

    let view = buffer.get_mapped_range(..).map_err(Error::MapRange)?;

    Ok(view
        .chunks_exact(4)
        .map(|word| u32::from_le_bytes(word.try_into().expect("a chunk holds four bytes")))
        .collect())
}
