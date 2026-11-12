use crate::{BoxFaces, BoxPart, HitTargets};
use std::{
    num::{NonZeroU32, NonZeroU64},
    ops::Range,
};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages,
    ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Device,
    Face, FragmentState, FrontFace, IndexFormat, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerBindingType, ShaderModule, ShaderStages,
    StencilState, TextureFormat, TextureSampleType, TextureView, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The vertices a box's shaders take: its 8 corners, each a number whose
/// bit `a` picks a side of the box on an axis.
const VERTICES: u32 = 8;

/// A box's three faces toward the view, two triangles each over the 7
/// corners the faces hold. Bit `b` of a corner picks the far side on the
/// axis the bit stands for. A face's triangles start at its corner that no
/// other face holds.
const FRONTS: [u16; 18] = [6, 0, 4, 6, 2, 0, 5, 0, 1, 5, 4, 0, 3, 0, 2, 3, 1, 0];

/// A box's 12 triangles over its 8 corners, wound counterclockwise from
/// outside. Bit `a` of a corner picks the box's high face on axis `a`.
///
/// Both triangles of a face start at one corner, and no other face's
/// triangles start there. A flat output of that corner then reaches the
/// face's fragments alone. The low face on axis `a` starts at corner `a`,
/// and the high face at corner `7 - a`.
const TRIANGLES: [u16; 36] = [
    0, 4, 6, 0, 6, 2, 7, 5, 1, 7, 1, 3, 1, 5, 4, 1, 4, 0, 6, 7, 3, 6, 3, 2, 2, 3, 1, 2, 1, 0, 5, 7,
    6, 5, 6, 4,
];

/// A view's boxes and the pipelines that draw their faces, with depth in
/// [`HitTargets::DEPTH`].
///
/// Each run draws in a pass of its own, which loads what the pass before it
/// stored. On Quest a second draw in a pass costs every box and pixel of
/// the pass: over the plain ids it cost S5 0.65 ms for 4096 boxes and S2
/// 2.8 ms for 25,301, and a shaded model of 800 boxes drew a third slower
/// in one pass than in a pass a run. The passes between cost under 0.02 ms
/// each.
///
/// A run of front faces draws each box's three faces toward the view: 6
/// triangles over 7 corners, where the box makes 12 over 8.
pub struct BoxDraw {
    /// The runs in draw order.
    parts: Vec<Run>,

    bind_group: BindGroup,

    corners: Buffer,

    indices: Buffer,

    /// Three `vec4<u32>`s per box.
    boxes: Buffer,
}

/// One run of a draw's boxes.
struct Run {
    pipeline: RenderPipeline,

    /// Whether the run draws each box's three faces toward the view, where
    /// any other run draws over its boxes' corners.
    fronts: bool,

    boxes: Range<u32>,
}

impl BoxDraw {
    /// The bytes of a box's three `vec4<u32>`s.
    pub const BOX_BYTES: usize = 48;

    /// Builds the pipelines over `module`, whose vertex entries take a
    /// vertex's bits and a box's three `vec4<u32>`s: `place`, `place_near`,
    /// and `place_held` a corner of the box, and `place_front` a vertex of
    /// one of its three faces toward the view. A module needs the entries
    /// its parts' faces draw through.
    ///
    /// # Arguments
    ///
    /// 1. `format`: the format of the target the fragments write
    /// 2. `parts`: the runs of boxes in draw order
    /// 3. `bindings`: each binding's stages, type, and buffer, in binding
    ///    order
    /// 4. `sampled`: a two-layer texture of filterable floats and its
    ///    sampler, which the fragments bind after the buffers
    /// 5. `multiview_mask`: the mask of the passes the boxes draw in
    pub fn new(
        device: &Device,
        module: &ShaderModule,
        format: TextureFormat,
        parts: &[BoxPart<'_>],
        bindings: &[(ShaderStages, BufferBindingType, &Buffer)],
        sampled: Option<(&TextureView, &Sampler)>,
        multiview_mask: Option<NonZeroU32>,
    ) -> Self {
        let mut box_bytes: Vec<u8> = parts
            .iter()
            .flat_map(|part| part.boxes.iter().flatten())
            .flat_map(|word| word.to_le_bytes())
            .collect();

        // A buffer cannot be empty.
        if box_bytes.is_empty() {
            box_bytes.resize(Self::BOX_BYTES, 0);
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
            &(0..VERTICES).flat_map(u32::to_le_bytes).collect::<Vec<_>>(),
            BufferUsages::VERTEX,
        );
        let indices = init(
            "box indices",
            &TRIANGLES
                .iter()
                .chain(&FRONTS)
                .flat_map(|index| index.to_le_bytes())
                .collect::<Vec<_>>(),
            BufferUsages::INDEX,
        );
        let boxes = init(
            "boxes",
            &box_bytes,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
        );

        let buffers = u32::try_from(bindings.len()).expect("a draw's bindings number under 2^32");

        let mut entries: Vec<_> = (0u32..)
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

        if sampled.is_some() {
            entries.extend([
                BindGroupLayoutEntry {
                    binding: buffers,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: buffers + 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ]);
        }

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

        let pipeline = |vertex: &str, fragment: &str, backs: bool| {
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(fragment),
                layout: Some(&layout),
                vertex: VertexState {
                    module,
                    entry_point: Some(vertex),
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
                            array_stride: Self::BOX_BYTES as u64,
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
                                VertexAttribute {
                                    format: VertexFormat::Uint32x4,
                                    offset: 32,
                                    shader_location: 3,
                                },
                            ],
                        }),
                    ],
                },
                primitive: PrimitiveState {
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(if backs { Face::Front } else { Face::Back }),
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
                    entry_point: Some(fragment),
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
        let mut runs = Vec::new();

        // Each pipeline built so far with its entries and faces, for a later
        // run to share.
        let mut built: Vec<((&str, &str, bool), RenderPipeline)> = Vec::new();

        let mut shared = |vertex: &'static str, fragment, backs| {
            let key = (vertex, fragment, backs);

            let found = built
                .iter()
                .find(|(built, _)| *built == key)
                .map(|(_, pipeline)| pipeline.clone());

            found.unwrap_or_else(|| {
                let pipeline = pipeline(vertex, fragment, backs);

                built.push((key, pipeline.clone()));

                pipeline
            })
        };

        for part in parts {
            let end =
                first + u32::try_from(part.boxes.len()).expect("a scene's boxes number under 2^32");

            let (vertex, backs) = match part.faces {
                BoxFaces::Front => ("place_front", false),
                BoxFaces::Back => ("place", true),
                BoxFaces::NearBack => ("place_near", true),
                BoxFaces::Held => ("place_held", true),
            };

            runs.push(Run {
                pipeline: shared(vertex, part.fragment, backs),
                fronts: part.faces == BoxFaces::Front,
                boxes: first..end,
            });

            first = end;
        }

        let mut resources: Vec<_> = (0u32..)
            .zip(bindings)
            .map(|(binding, (_, _, buffer))| BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            })
            .collect();

        if let Some((texture, sampler)) = sampled {
            resources.extend([
                BindGroupEntry {
                    binding: buffers,
                    resource: BindingResource::TextureView(texture),
                },
                BindGroupEntry {
                    binding: buffers + 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ]);
        }

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("box raster"),
            layout: &bind_group_layout,
            entries: &resources,
        });

        BoxDraw {
            parts: runs,
            bind_group,
            corners,
            indices,
            boxes,
        }
    }

    /// Replaces the boxes with `boxes`, [`BOX_BYTES`](Self::BOX_BYTES) each,
    /// of which each run takes its count of `counts` in draw order. Panics
    /// unless `counts` has a count for each part and `boxes` holds those
    /// boxes and no more than the draw was built with.
    pub fn rewrite(&mut self, queue: &Queue, boxes: &[u8], counts: &[u32]) {
        assert_eq!(counts.len(), self.parts.len(), "each part has its count");
        assert_eq!(
            boxes.len() as u64,
            u64::from(counts.iter().sum::<u32>()) * Self::BOX_BYTES as u64,
            "the counts cover the boxes"
        );
        assert!(
            boxes.len() as u64 <= self.boxes.size(),
            "the runs hold no more boxes than the draw was built with"
        );

        queue.write_buffer(&self.boxes, 0, boxes);

        let mut first = 0;

        for (run, count) in self.parts.iter_mut().zip(counts) {
            run.boxes = first..first + count;
            first += count;
        }
    }

    /// How many passes draw the boxes: one for each run that holds boxes,
    /// and one for a draw that holds none.
    pub fn passes(&self) -> u32 {
        u32::try_from(self.drawn().count())
            .expect("a draw's runs number under 2^32")
            .max(1)
    }

    /// Draws the `at`th of the draw's [`passes`](Self::passes) into `pass`.
    /// Draws nothing past the last run that holds boxes.
    pub fn draw(&self, pass: &mut RenderPass<'_>, at: u32) {
        let Some(run) = self.drawn().nth(at as usize) else {
            return;
        };

        // A box's corners make the first triangles of `indices`, and its
        // three faces toward the view make the rest.
        let corners = 0..TRIANGLES.len() as u32;
        let fronts = corners.end..corners.end + FRONTS.len() as u32;

        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.corners.slice(..));
        pass.set_vertex_buffer(1, self.boxes.slice(..));
        pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint16);
        pass.set_pipeline(&run.pipeline);
        pass.draw_indexed(
            if run.fronts { fronts } else { corners },
            0,
            run.boxes.clone(),
        );
    }

    /// The runs that hold boxes, in draw order.
    fn drawn(&self) -> impl Iterator<Item = &Run> {
        self.parts.iter().filter(|run| !run.boxes.is_empty())
    }
}
