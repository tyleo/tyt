use crate::{Error, Result, XrGpu};
use std::sync::mpsc;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages, Color,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, ComputePassDescriptor,
    ComputePipelineDescriptor, Device, Extent3d, FragmentState, IndexFormat, LoadOp, MapMode,
    MultisampleState, Operations, Origin3d, PipelineCompilationOptions, PollType, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource, StoreOp, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor, VertexState,
    util::{
        BufferInitDescriptor, DeviceExt, DispatchIndirectArgs, DrawIndexedIndirectArgs,
        DrawIndirectArgs,
    },
};

/// The side of the square target the draws cover. One row of
/// `Rgba8Unorm` texels fills the 256 bytes a copy row needs.
const SIDE: u32 = 64;

/// The bytes of one texel row.
const ROW_BYTES: u32 = SIDE * 4;

/// The workgroups the indirect dispatch asks for.
const GROUPS: [u32; 3] = [3, 2, 1];

/// The texel the draws write.
const RED: [u8; 4] = [255, 0, 0, 255];

/// How a probe draw issues its triangle.
#[derive(Clone, Copy, Debug)]
enum DrawCall {
    /// `draw`, the control the indirect calls compare against.
    Direct,

    Indirect,

    IndexedIndirect,
}

/// Logs how much of its work each B0 indirect call does, beside a direct
/// draw as the control.
pub fn probe_indirect(gpu: &XrGpu) -> Result<()> {
    let shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
        label: Some("b0 indirect"),
        source: ShaderSource::Wgsl(include_str!("indirect.wgsl").into()),
    });

    for call in [
        DrawCall::Direct,
        DrawCall::Indirect,
        DrawCall::IndexedIndirect,
    ] {
        log::info!(
            "b0 draw {call:?}: {} of {} texels red",
            red_texels(gpu, &shader, call)?,
            SIDE * SIDE
        );
    }

    log::info!(
        "b0 dispatch_workgroups_indirect: {} of {} workgroups ran",
        dispatched_groups(gpu, &shader)?,
        GROUPS.iter().product::<u32>()
    );

    Ok(())
}

/// The texels a covering triangle issued by `call` turns red.
fn red_texels(gpu: &XrGpu, shader: &ShaderModule, call: DrawCall) -> Result<usize> {
    let device = &gpu.device;

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("b0 draw"),
        layout: None,
        vertex: VertexState {
            module: shader,
            entry_point: Some("cover"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: shader,
            entry_point: Some("red"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format: TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let size = Extent3d {
        width: SIDE,
        height: SIDE,
        depth_or_array_layers: 1,
    };

    let target = device.create_texture(&TextureDescriptor {
        label: Some("b0 draw"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let view = target.create_view(&TextureViewDescriptor::default());

    let args = if let DrawCall::IndexedIndirect = call {
        DrawIndexedIndirectArgs {
            index_count: 3,
            instance_count: 1,
            first_index: 0,
            base_vertex: 0,
            first_instance: 0,
        }
        .as_bytes()
        .to_vec()
    } else {
        DrawIndirectArgs {
            vertex_count: 3,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
        }
        .as_bytes()
        .to_vec()
    };

    let args = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("b0 draw args"),
        contents: &args,
        usage: BufferUsages::INDIRECT,
    });

    let indices = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("b0 draw indices"),
        contents: &[0u32, 1, 2].map(u32::to_le_bytes).concat(),
        usage: BufferUsages::INDEX,
    });

    let readback = readback(device, "b0 draw readback", u64::from(ROW_BYTES * SIDE));

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("b0 draw"),
    });

    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("b0 draw"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            ..RenderPassDescriptor::default()
        });

        pass.set_pipeline(&pipeline);

        match call {
            DrawCall::Direct => pass.draw(0..3, 0..1),

            DrawCall::Indirect => pass.draw_indirect(&args, 0),

            DrawCall::IndexedIndirect => {
                pass.set_index_buffer(indices.slice(..), IndexFormat::Uint32);
                pass.draw_indexed_indirect(&args, 0);
            }
        }
    }

    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: &readback,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ROW_BYTES),
                rows_per_image: None,
            },
        },
        size,
    );

    gpu.queue.submit([encoder.finish()]);

    let texels = read_bytes(device, &readback)?;

    Ok(texels.chunks_exact(4).filter(|texel| *texel == RED).count())
}

/// The workgroups an indirect dispatch of `GROUPS` runs.
fn dispatched_groups(gpu: &XrGpu, shader: &ShaderModule) -> Result<u32> {
    let device = &gpu.device;

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("b0 dispatch"),
        layout: None,
        module: shader,
        entry_point: Some("count"),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    });

    let args = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("b0 dispatch args"),
        contents: DispatchIndirectArgs {
            x: GROUPS[0],
            y: GROUPS[1],
            z: GROUPS[2],
        }
        .as_bytes(),
        usage: BufferUsages::INDIRECT,
    });

    let groups = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("b0 dispatch groups"),
        contents: &0u32.to_le_bytes(),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
    });

    let readback = readback(device, "b0 dispatch readback", 4);

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("b0 dispatch"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[BindGroupEntry {
            binding: 0,
            resource: groups.as_entire_binding(),
        }],
    });

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("b0 dispatch"),
    });

    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups_indirect(&args, 0);
    }

    encoder.copy_buffer_to_buffer(&groups, 0, &readback, 0, 4);

    gpu.queue.submit([encoder.finish()]);

    let bytes = read_bytes(device, &readback)?;
    Ok(u32::from_le_bytes(
        bytes.try_into().expect("the readback holds one word"),
    ))
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

/// The bytes of `buffer` once the GPU has written it.
fn read_bytes(device: &Device, buffer: &Buffer) -> Result<Vec<u8>> {
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

    Ok(view.to_vec())
}
