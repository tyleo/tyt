use crate::{Error, Result};
use std::{
    result::Result as StdResult,
    sync::mpsc::{self, Receiver, TryRecvError},
};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAsyncError, BufferDescriptor,
    BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder, Device, Extent3d,
    FragmentState, LoadOp, MapMode, MultisampleState, Operations, Origin3d,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PollType, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, TexelCopyBufferInfo,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexState,
};

/// The format of the texel the depth passes through. A copy takes all of a
/// depth texture or none, so a pass draws the one texel a frame reads.
const FORMAT: TextureFormat = TextureFormat::R32Float;

/// The depth a window's frames draw at the middle of their first eye, read
/// back a frame or two after each draws. A frame is skipped while the one
/// before it reads back.
pub struct CenterDepth {
    pipeline: RenderPipeline,

    bound: BindGroup,

    /// The texel a frame's pass draws its depth into.
    texel: Texture,

    texel_view: TextureView,

    /// Where the texel lands.
    buffer: Buffer,

    step: Step,
}

/// Where the buffer stands.
enum Step {
    /// No frame has it.
    Free,

    /// A frame's encoder copies into it.
    Copied,

    /// It maps once the GPU has written it.
    Mapping(Receiver<StdResult<(), BufferAsyncError>>),
}

impl CenterDepth {
    /// The reader of `depth`, a view of the first eye's layer of the depth
    /// a window's frames draw.
    pub fn new(device: &Device, depth: &TextureView) -> Self {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("center depth"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Depth,
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let bound = device.create_bind_group(&BindGroupDescriptor {
            label: Some("center depth"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(depth),
            }],
        });

        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("center depth"),
            source: ShaderSource::Wgsl(include_str!("center_depth.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("center depth"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("center depth"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &module,
                entry_point: Some("corner"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &module,
                entry_point: Some("middle"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let texel = device.create_texture(&TextureDescriptor {
            label: Some("center depth"),
            size: Extent3d::default(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        CenterDepth {
            pipeline,
            bound,
            texel_view: texel.create_view(&TextureViewDescriptor::default()),
            texel,
            buffer: device.create_buffer(&BufferDescriptor {
                label: Some("center depth"),
                size: 4,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            step: Step::Free,
        }
    }

    /// The depth of the last frame to read back since the call before, if
    /// one has. Errors if the read back fails.
    pub fn read(&mut self, device: &Device) -> Result<Option<f32>> {
        let Step::Mapping(mapped) = &self.step else {
            return Ok(None);
        };

        // The map's callback runs within a poll.
        device
            .poll(PollType::Poll)
            .map_err(|error| Error::Depth(error.to_string()))?;

        match mapped.try_recv() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(Error::Depth(error.to_string())),
            Err(TryRecvError::Empty) => return Ok(None),
            Err(TryRecvError::Disconnected) => {
                return Err(Error::Depth("the map's callback never ran".to_owned()));
            }
        }

        let depth = {
            let bytes = self
                .buffer
                .get_mapped_range(..)
                .map_err(|error| Error::Depth(error.to_string()))?;

            f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        };

        self.buffer.unmap();
        self.step = Step::Free;

        Ok(Some(depth))
    }

    /// Encodes the pass that takes the depth a frame's passes have drawn
    /// and its copy, unless a frame before still reads back.
    pub fn copy(&mut self, encoder: &mut CommandEncoder) {
        if !matches!(self.step, Step::Free) {
            return;
        }

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("center depth"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.texel_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: StoreOp::Store,
                    },
                })],
                ..RenderPassDescriptor::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bound, &[]);
            pass.draw(0..3, 0..1);
        }

        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: &self.texel,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &self.buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: None,
                    rows_per_image: None,
                },
            },
            Extent3d::default(),
        );

        self.step = Step::Copied;
    }

    /// Starts the read back of a copy, once its encoder has gone to the
    /// queue.
    pub fn sent(&mut self) {
        if !matches!(self.step, Step::Copied) {
            return;
        }

        let (sender, receiver) = mpsc::channel();

        self.buffer.map_async(MapMode::Read, .., move |result| {
            // The window may have gone, and then nobody waits.
            let _ = sender.send(result);
        });

        self.step = Step::Mapping(receiver);
    }
}
