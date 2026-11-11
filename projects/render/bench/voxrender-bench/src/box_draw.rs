use crate::{BoxPart, HitTargets};
use std::{
    num::{NonZeroU32, NonZeroU64},
    ops::Range,
};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ColorTargetState,
    ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Device, Face, FragmentState,
    FrontFace, IndexFormat, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderStages, StencilState, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

/// A box's 12 triangles over its 8 corners, wound counterclockwise from
/// outside. Bit `a` of a corner picks the box's high face on axis `a`.
const TRIANGLES: [u16; 36] = [
    0, 4, 6, 0, 6, 2, 1, 3, 7, 1, 7, 5, 0, 1, 5, 0, 5, 4, 2, 6, 7, 2, 7, 3, 0, 2, 3, 0, 3, 1, 4, 5,
    7, 4, 7, 6,
];

/// A view's boxes and the pipelines that draw their faces, with depth in
/// [`HitTargets::DEPTH`].
pub struct BoxDraw {
    /// Each run's pipeline and boxes, in draw order.
    parts: Vec<(RenderPipeline, Range<u32>)>,

    bind_group: BindGroup,

    corners: Buffer,

    indices: Buffer,

    /// Two `vec4<u32>`s per box.
    boxes: Buffer,
}

impl BoxDraw {
    /// Builds the pipelines over `module`, whose vertex entries take a
    /// corner's bits and a box's two `vec4<u32>`s.
    ///
    /// # Arguments
    ///
    /// 1. `format`: the format of the target the fragments write
    /// 2. `parts`: the runs of boxes in draw order
    /// 3. `bindings`: each binding's stages, type, and buffer, in binding
    ///    order
    /// 4. `multiview_mask`: the mask of the passes the boxes draw in
    pub fn new(
        device: &Device,
        module: &ShaderModule,
        format: TextureFormat,
        parts: &[BoxPart<'_>],
        bindings: &[(ShaderStages, BufferBindingType, &Buffer)],
        multiview_mask: Option<NonZeroU32>,
    ) -> Self {
        let mut box_bytes: Vec<u8> = parts
            .iter()
            .flat_map(|part| part.boxes.iter().flatten())
            .flat_map(|word| word.to_le_bytes())
            .collect();

        // A buffer cannot be empty.
        if box_bytes.is_empty() {
            box_bytes.resize(32, 0);
        }

        let init = |label, contents: &[u8], usage| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
        };

        let corners = init(
            "box corners",
            &(0u32..8).flat_map(u32::to_le_bytes).collect::<Vec<_>>(),
            BufferUsages::VERTEX,
        );
        let indices = init(
            "box indices",
            &TRIANGLES
                .iter()
                .flat_map(|index| index.to_le_bytes())
                .collect::<Vec<_>>(),
            BufferUsages::INDEX,
        );
        let boxes = init(
            "boxes",
            &box_bytes,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
        );

        let entries: Vec<_> = (0u32..)
            .zip(bindings)
            .map(|(binding, &(visibility, ty, _))| BindGroupLayoutEntry {
                binding,
                visibility,
                ty: BindingType::Buffer {
                    ty,
                    has_dynamic_offset: false,
                    min_binding_size: None::<NonZeroU64>,
                },
                count: None,
            })
            .collect();

        // The pipelines share the layout, so one bind group serves them all.
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("box raster"),
            entries: &entries,
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("box raster"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = |part: &BoxPart<'_>| {
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(part.fragment),
                layout: Some(&layout),
                vertex: VertexState {
                    module,
                    entry_point: Some(part.vertex),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[
                        Some(VertexBufferLayout {
                            array_stride: 4,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &[VertexAttribute {
                                format: VertexFormat::Uint32,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }),
                        Some(VertexBufferLayout {
                            array_stride: 32,
                            step_mode: VertexStepMode::Instance,
                            attributes: &[
                                VertexAttribute {
                                    format: VertexFormat::Uint32x4,
                                    offset: 0,
                                    shader_location: 1,
                                },
                                VertexAttribute {
                                    format: VertexFormat::Uint32x4,
                                    offset: 16,
                                    shader_location: 2,
                                },
                            ],
                        }),
                    ],
                },
                primitive: PrimitiveState {
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(if part.backs { Face::Front } else { Face::Back }),
                    ..PrimitiveState::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: HitTargets::DEPTH,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(CompareFunction::Greater),
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module,
                    entry_point: Some(part.fragment),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask,
                cache: None,
            })
        };

        let mut first = 0;
        let mut runs: Vec<(RenderPipeline, Range<u32>)> = Vec::new();

        // Each pipeline built so far with its entries and faces, for a later
        // run to share.
        let mut built: Vec<((&str, &str, bool), RenderPipeline)> = Vec::new();

        for part in parts {
            let end =
                first + u32::try_from(part.boxes.len()).expect("a scene's boxes number under 2^32");

            let key = (part.vertex, part.fragment, part.backs);

            let shared = built
                .iter()
                .find(|(built, _)| *built == key)
                .map(|(_, pipeline)| pipeline.clone());

            let pipeline = shared.unwrap_or_else(|| {
                let pipeline = pipeline(part);

                built.push((key, pipeline.clone()));

                pipeline
            });

            runs.push((pipeline, first..end));
            first = end;
        }

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("box raster"),
            layout: &bind_group_layout,
            entries: &(0u32..)
                .zip(bindings)
                .map(|(binding, (_, _, buffer))| BindGroupEntry {
                    binding,
                    resource: buffer.as_entire_binding(),
                })
                .collect::<Vec<_>>(),
        });

        BoxDraw {
            parts: runs,
            bind_group,
            corners,
            indices,
            boxes,
        }
    }

    /// Replaces the boxes with `boxes`, 32 bytes of two `vec4<u32>`s each,
    /// of which each run takes its count of `counts` in draw order. Panics
    /// unless `counts` has a count for each part and `boxes` holds those
    /// boxes and no more than the draw was built with.
    pub fn rewrite(&mut self, queue: &Queue, boxes: &[u8], counts: &[u32]) {
        assert_eq!(counts.len(), self.parts.len(), "each part has its count");
        assert_eq!(
            boxes.len() as u64,
            u64::from(counts.iter().sum::<u32>()) * 32,
            "the counts cover the boxes"
        );
        assert!(
            boxes.len() as u64 <= self.boxes.size(),
            "the runs hold no more boxes than the draw was built with"
        );

        queue.write_buffer(&self.boxes, 0, boxes);

        let mut first = 0;

        for ((_, run), count) in self.parts.iter_mut().zip(counts) {
            *run = first..first + count;
            first += count;
        }
    }

    /// Draws every box into `pass`.
    pub fn draw(&self, pass: &mut RenderPass<'_>) {
        if self.parts.iter().all(|(_, boxes)| boxes.is_empty()) {
            return;
        }

        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.corners.slice(..));
        pass.set_vertex_buffer(1, self.boxes.slice(..));
        pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint16);

        for (pipeline, boxes) in &self.parts {
            if !boxes.is_empty() {
                pass.set_pipeline(pipeline);
                pass.draw_indexed(0..36, 0, boxes.clone());
            }
        }
    }
}
