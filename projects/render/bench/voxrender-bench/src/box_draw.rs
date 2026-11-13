use crate::{BoxBindings, BoxFaces, BoxPart};
use std::{
    num::{NonZeroU32, NonZeroU64},
    ops::Range,
};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent, BlendFactor,
    BlendOperation, BlendState, Buffer, BufferUsages, ColorTargetState, ColorWrites,
    CompareFunction, DepthBiasState, DepthStencilState, Device, Face, FragmentState, FrontFace,
    IndexFormat, MultisampleState, PipelineCompilationOptions, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, ShaderModule, ShaderStages, StencilState,
    TextureFormat, TextureSampleType, TextureViewDimension, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexState, VertexStepMode,
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

/// The steps of depth a glass face draws nearer by. A glass box can share
/// a face's plane with the opaque box of its tile and draws before it.
const GLASS_BIAS: i32 = 1024;

/// What picks a run's pipeline: its module, its vertex and fragment entries,
/// whether it draws its boxes' back faces, and whether it draws glass.
type PipelineKey<'a> = (usize, &'a str, &'a str, bool, bool);

/// A view's boxes and the pipelines that draw their faces, with depth in
/// [`DEPTH`](Self::DEPTH).
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

    layout: PipelineLayout,

    /// What [`cover`](Self::cover) draws after the runs, and the pixels
    /// it draws within: a rectangle's left, top, width, and height.
    cover: Option<(RenderPipeline, [u32; 4])>,
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

    /// The depth format. Depth runs reversed in f32.
    pub const DEPTH: TextureFormat = TextureFormat::Depth32Float;

    /// Builds the pipelines over `modules`, whose vertex entries take a
    /// vertex's bits and a box's three `vec4<u32>`s: `place_near` and
    /// `place_held` a corner of the box, and `place_front` a vertex of one
    /// of its three faces toward the view. A module needs the entries
    /// its parts' faces draw through.
    ///
    /// # Arguments
    ///
    /// 1. `format`: the format of the target the fragments write
    /// 2. `parts`: the runs of boxes in draw order
    /// 3. `bindings`: what the shaders bind in group 0
    /// 4. `multiview_mask`: the mask of the passes the boxes draw in
    pub fn new(
        device: &Device,
        modules: &[&ShaderModule],
        format: TextureFormat,
        parts: &[BoxPart<'_>],
        bindings: &BoxBindings<'_>,
        multiview_mask: Option<NonZeroU32>,
    ) -> Self {
        let BoxBindings {
            buffers: bindings,
            sampled,
            tlas,
        } = *bindings;

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

        if tlas.is_some() {
            entries.push(BindGroupLayoutEntry {
                binding: buffers + 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::AccelerationStructure {
                    vertex_return: false,
                },
                count: None,
            });
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

        // Glass keeps the target's color by the second color its fragment
        // writes, and adds the first.
        let tinted = |dst_factor| BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor,
            operation: BlendOperation::Add,
        };

        let pipeline = |module: usize, vertex: &str, fragment: &str, backs: bool, glass: bool| {
            let module = modules[module];

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
                    format: Self::DEPTH,
                    depth_write_enabled: Some(!glass),
                    depth_compare: Some(CompareFunction::Greater),
                    stencil: StencilState::default(),
                    bias: DepthBiasState {
                        constant: if glass { GLASS_BIAS } else { 0 },
                        ..DepthBiasState::default()
                    },
                }),
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module,
                    entry_point: Some(fragment),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format,
                        blend: glass.then(|| BlendState {
                            color: tinted(BlendFactor::Src1),
                            alpha: tinted(BlendFactor::Src1Alpha),
                        }),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask,
                cache: None,
            })
        };

        let mut first = 0;
        let mut runs = Vec::new();

        // Each pipeline built so far with its module, entries, and faces, for
        // a later run to share.
        let mut built: Vec<(PipelineKey<'_>, RenderPipeline)> = Vec::new();

        let mut shared = |module, vertex: &'static str, fragment, backs, glass| {
            let key = (module, vertex, fragment, backs, glass);

            let found = built
                .iter()
                .find(|(built, _)| *built == key)
                .map(|(_, pipeline)| pipeline.clone());

            found.unwrap_or_else(|| {
                let pipeline = pipeline(module, vertex, fragment, backs, glass);

                built.push((key, pipeline.clone()));

                pipeline
            })
        };

        for part in parts {
            let end =
                first + u32::try_from(part.boxes.len()).expect("a scene's boxes number under 2^32");

            let (vertex, backs) = match part.faces {
                BoxFaces::Front => ("place_front", false),
                BoxFaces::NearBack => ("place_near", true),
                BoxFaces::Held => ("place_held", true),
            };

            runs.push(Run {
                pipeline: shared(part.module, vertex, part.fragment, backs, part.glass),
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

        if let Some(tlas) = tlas {
            resources.push(BindGroupEntry {
                binding: buffers + 2,
                resource: BindingResource::AccelerationStructure(tlas),
            });
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
            layout,
            cover: None,
        }
    }

    /// Adds a last pass that draws one triangle where no box drew: three
    /// vertices through `module`'s `vertex` entry, which places them at
    /// the depth a pass clears to, and its `fragment` entry, whose color
    /// the target's blend adds. The entries bind what the boxes' do. The
    /// pass draws nothing until [`cover_within`](Self::cover_within) gives
    /// it pixels.
    pub fn cover(
        &mut self,
        device: &Device,
        module: &ShaderModule,
        vertex: &str,
        fragment: &str,
        format: TextureFormat,
        multiview_mask: Option<NonZeroU32>,
    ) {
        let added = BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        };

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(fragment),
            layout: Some(&self.layout),
            vertex: VertexState {
                module,
                entry_point: Some(vertex),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: Self::DEPTH,
                depth_write_enabled: Some(false),
                depth_compare: Some(CompareFunction::Equal),
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
                    blend: Some(BlendState {
                        color: added,
                        alpha: added,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask,
            cache: None,
        });

        self.cover = Some((pipeline, [0; 4]));
    }

    /// Sets the pixels of the target the [`cover`](Self::cover) draws
    /// within: a rectangle's left, top, width, and height, inside the
    /// target.
    pub fn cover_within(&mut self, pixels: [u32; 4]) {
        if let Some((_, within)) = &mut self.cover {
            *within = pixels;
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
    /// or one for a draw that holds none, then one for a
    /// [`cover`](Self::cover).
    pub fn passes(&self) -> u32 {
        self.runs() + u32::from(self.cover.is_some())
    }

    /// Draws the `at`th of the draw's [`passes`](Self::passes) into `pass`.
    /// Draws nothing past the last.
    pub fn draw(&self, pass: &mut RenderPass<'_>, at: u32) {
        if let Some((cover, [left, top, width, height])) =
            self.cover.as_ref().filter(|_| at == self.runs())
        {
            if *width > 0 && *height > 0 {
                pass.set_scissor_rect(*left, *top, *width, *height);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_pipeline(cover);
                pass.draw(0..3, 0..1);
            }

            return;
        }

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

    /// The passes the runs draw in.
    fn runs(&self) -> u32 {
        u32::try_from(self.drawn().count())
            .expect("a draw's runs number under 2^32")
            .max(1)
    }

    /// The runs that hold boxes, in draw order.
    fn drawn(&self) -> impl Iterator<Item = &Run> {
        self.parts.iter().filter(|run| !run.boxes.is_empty())
    }
}
