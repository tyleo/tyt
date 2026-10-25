use crate::{Error, Result};
use std::sync::mpsc;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BufferDescriptor, BufferUsages, CommandEncoderDescriptor,
    ComputePassDescriptor, ComputePipelineDescriptor, Device, Features, MapMode,
    PipelineCompilationOptions, PollType, Queue, ShaderModuleDescriptor, ShaderRuntimeChecks,
    ShaderSource,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The least and greatest subgroup sizes of a compute dispatch over a `width`
/// by `height` image in 8x8 workgroups, or `None` when the device lacks
/// subgroups.
pub fn subgroup_sizes(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,
) -> Result<Option<(u32, u32)>> {
    if !device.features().contains(Features::SUBGROUP) {
        return Ok(None);
    }

    let image = [width, height]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>();

    let image = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("subgroup image"),
        contents: &image,
        usage: BufferUsages::UNIFORM,
    });

    let sizes = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("subgroup sizes"),
        contents: &[u32::MAX, 0]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>(),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
    });

    let readback = device.create_buffer(&BufferDescriptor {
        label: Some("subgroup size readback"),
        size: sizes.size(),
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // SAFETY: the shader indexes its bindings by constants and has no loops.
    let module = unsafe {
        device.create_shader_module_trusted(
            ShaderModuleDescriptor {
                label: Some("subgroup_sizes"),
                source: ShaderSource::Wgsl(include_str!("subgroup_sizes.wgsl").into()),
            },
            ShaderRuntimeChecks::unchecked(),
        )
    };

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("subgroup_sizes"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("subgroup_sizes"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: image.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: sizes.as_entire_binding(),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("subgroup_sizes"),
    });

    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("subgroup_sizes"),
            timestamp_writes: None,
        });

        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }

    encoder.copy_buffer_to_buffer(&sizes, 0, &readback, 0, None);
    queue.submit([encoder.finish()]);

    let (sender, receiver) = mpsc::channel();

    readback.map_async(MapMode::Read, .., move |result| {
        sender.send(result).expect("the receiver outlives the wait");
    });

    device
        .poll(PollType::wait_indefinitely())
        .map_err(Error::Poll)?;

    receiver
        .recv()
        .expect("the map callback runs during the wait")
        .map_err(Error::Map)?;

    let view = readback.get_mapped_range(..).map_err(Error::MapRange)?;
    let size = |at: usize| {
        u32::from_le_bytes(
            view[at..at + 4]
                .try_into()
                .expect("a size holds four bytes"),
        )
    };

    Ok(Some((size(0), size(4))))
}
