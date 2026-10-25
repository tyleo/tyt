use wgpu::{
    Color, ColorTargetState, ColorWrites, CommandEncoder, Device, Extent3d, FragmentState, LoadOp,
    MultisampleState, Operations, PipelineCompilationOptions, PrimitiveState, QuerySet,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPassTimestampWrites, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderRuntimeChecks, ShaderSource, StoreOp,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexState,
};

/// The format a shading pass would write.
const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

/// A fragment launch over every pixel of two `width` by `height` eyes, each
/// fragment writing a constant.
pub struct FragmentLaunch {
    pipeline: RenderPipeline,

    eyes: [TextureView; 2],
}

impl FragmentLaunch {
    /// Builds the launch over two `width` by `height` eyes.
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        // SAFETY: the shaders read no bindings and have no loops.
        let module = unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some("launch"),
                    source: ShaderSource::Wgsl(include_str!("launch.wgsl").into()),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("fragment launch"),
            layout: None,
            vertex: VertexState {
                module: &module,
                entry_point: Some("cover"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &module,
                entry_point: Some("fill"),
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

        let target = device.create_texture(&TextureDescriptor {
            label: Some("fragment launch eyes"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let eyes = [0, 1].map(|layer| {
            target.create_view(&TextureViewDescriptor {
                dimension: Some(TextureViewDimension::D2),
                base_array_layer: layer,
                array_layer_count: Some(1),
                ..TextureViewDescriptor::default()
            })
        });

        FragmentLaunch { pipeline, eyes }
    }

    /// Records one pass per eye into `encoder`, between the timestamps at
    /// `first` and `first + 1` of `query_set`.
    pub fn record(&self, encoder: &mut CommandEncoder, query_set: &QuerySet, first: u32) {
        for (eye, view) in self.eyes.iter().enumerate() {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("fragment launch"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: Some(RenderPassTimestampWrites {
                    query_set,
                    beginning_of_pass_write_index: (eye == 0).then_some(first),
                    end_of_pass_write_index: (eye == 1).then_some(first + 1),
                }),
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);
        }
    }
}
