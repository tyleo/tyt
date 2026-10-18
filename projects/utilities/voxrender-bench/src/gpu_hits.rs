use crate::{Error, Gpu, GpuWalk, PixelHit, Result, shadow_targets, view_grid_rays};
use branded_id::U32Id;
use std::{collections::HashMap, sync::mpsc};
use voxrender::{
    BRenderView, Error as RenderError, GRID_GUARD_BITS, RenderScene, grid_point, grid_vector,
    quantize_direction, quantize_point,
};
use voxrender_cpu::ShadowTarget;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, Device, MapMode,
    PipelineCompilationOptions, PollType, ShaderModuleDescriptor, ShaderSource,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The shader's marker for a pixel without a hit.
const NO_HIT: u32 = u32::MAX;

/// The bytes of one hit in the shader's output.
const HIT_BYTES: u64 = 16;

/// The GPU's first hit through each pixel of `view_id` over a `size` square
/// image, row by row, from `first_hit.wgsl` under `walk`.
pub fn gpu_hits(
    gpu: &Gpu,
    scene: &RenderScene,
    view_id: U32Id<BRenderView>,
    size: u32,
    walk: GpuWalk,
) -> Result<Vec<Option<PixelHit>>> {
    let grid_rays = view_grid_rays(scene, view_id, size)?;
    let targets = shadow_targets(scene);

    // Each object's cells in raster order: 0 when empty, else its material
    // id plus one.
    let mut cells = Vec::new();
    let mut offsets = HashMap::new();

    for (object_id, object) in scene.iter_objects() {
        offsets.insert(
            object_id,
            u32::try_from(cells.len()).expect("a scene's cells number under 2^32"),
        );

        let bounds = object.bounds();

        for index in 0..bounds.x * bounds.y * bounds.z {
            let material_id = object.voxel_material(U32Id::from_u32(index));

            cells.push(material_id.map_or(0, |material_id| material_id.to_u32() + 1));
        }
    }

    let mut placements = Vec::new();

    for ((_, placement), rays) in scene.iter_placements().zip(&grid_rays) {
        let object = scene
            .object(placement.object_id)
            .expect("a placement's object is one of the scene's");
        let bounds = object
            .bounds()
            .to_array()
            .map(|side| i32::try_from(side).expect("a grid's sides are within range"));
        let scale = placement
            .transform
            .scale
            .to_array()
            .map(|scale| scale as f32);

        let rotation = placement.transform.rotation;

        push_i32s(&mut placements, bounds);
        push_u32s(&mut placements, [offsets[&placement.object_id], 0, 0]);
        push_f32s(&mut placements, scale);
        push_f32s(
            &mut placements,
            placement
                .transform
                .position
                .to_array()
                .map(|coordinate| coordinate as f32),
        );

        for value in [rotation.x, rotation.y, rotation.z, rotation.w] {
            placements.extend((value as f32).to_le_bytes());
        }

        for steps in [
            [rays.origin, rays.origin_x, rays.origin_y],
            [rays.direction, rays.direction_x, rays.direction_y],
        ] {
            for vector in steps {
                let (whole, fraction) = split(vector);

                push_i32s(&mut placements, whole);
                push_i32s(&mut placements, fraction);
            }
        }
    }

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

    for value in [size, size, placement_count, light_count] {
        frame.extend(value.to_le_bytes());
    }

    let device = &gpu.device;

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
        &cells
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

    let pixels = u64::from(size) * u64::from(size);
    let hit_bytes = pixels * HIT_BYTES;
    let shadow_bytes = (pixels * u64::from(light_count)).max(1) * 4;

    let hits = output(device, "hits", hit_bytes);
    let shadows = output(device, "shadows", shadow_bytes);
    let hit_readback = readback(device, "hit readback", hit_bytes);
    let shadow_readback = readback(device, "shadow readback", shadow_bytes);

    let module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("first_hit"),
        source: ShaderSource::Wgsl(include_str!("first_hit.wgsl").into()),
    });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("first_hit"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: PipelineCompilationOptions {
            constants: &[("FLOAT_WALK", f64::from(u8::from(walk == GpuWalk::Float)))],
            ..PipelineCompilationOptions::default()
        },
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

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("first_hit"),
    });

    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("first_hit"),
            timestamp_writes: None,
        });

        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(size.div_ceil(8), size.div_ceil(8), 1);
    }

    encoder.copy_buffer_to_buffer(&hits, 0, &hit_readback, 0, hit_bytes);
    encoder.copy_buffer_to_buffer(&shadows, 0, &shadow_readback, 0, shadow_bytes);
    gpu.queue.submit([encoder.finish()]);

    let hits = read_words(device, &hit_readback)?;
    let shadows = read_words(device, &shadow_readback)?;
    let lights = targets.len();

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

/// `vector`, which carries guard bits, as whole units and a fraction in
/// `[0, 2^GRID_GUARD_BITS)`, so the shader can sum steps in i32.
fn split(vector: [i64; 3]) -> ([i32; 3], [i32; 3]) {
    let mask = (1 << GRID_GUARD_BITS) - 1;

    (
        vector.map(|component| {
            i32::try_from(component >> GRID_GUARD_BITS).expect("an in-range ray's steps fit i32")
        }),
        vector.map(|component| (component & mask) as i32),
    )
}

/// Appends `values` and a zero as one `vec4<i32>`.
fn push_i32s(bytes: &mut Vec<u8>, values: [i32; 3]) {
    for value in values.into_iter().chain([0]) {
        bytes.extend(value.to_le_bytes());
    }
}

/// Appends `values` and a zero as one `vec4<u32>`.
fn push_u32s(bytes: &mut Vec<u8>, values: [u32; 3]) {
    for value in values.into_iter().chain([0]) {
        bytes.extend(value.to_le_bytes());
    }
}

/// Appends `values` and a zero as one `vec4<f32>`.
fn push_f32s(bytes: &mut Vec<u8>, values: [f32; 3]) {
    for value in values.into_iter().chain([0.0]) {
        bytes.extend(value.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use crate::gpu_hits::split;

    #[test]
    fn a_split_keeps_its_value_with_a_fraction_under_the_guard() {
        let vector = [(1 << 40) + 4095, -1, -(1 << 40) - 4096];
        let (whole, fraction) = split(vector);

        assert_eq!(whole, [1 << 28, -1, -(1 << 28) - 1]);
        assert_eq!(fraction, [4095, 4095, 0]);

        for a in 0..3 {
            assert_eq!(
                (i64::from(whole[a]) << 12) + i64::from(fraction[a]),
                vector[a]
            );
        }
    }
}
