use crate::{BoxFaces, BoxPart, HitTargets};
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

/// The vertices a box's shaders take: the 8 corners for its back faces,
/// and 4 vertices for each of its three faces toward the view.
const VERTICES: u32 = 12;

/// A box's three faces toward the view, two triangles each over the 4
/// vertices of a face. A vertex holds its face's axis above two bits.
const FRONTS: [u16; 18] = [0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11];

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
/// A draw of [`APART`](Self::APART) boxes or more counts its cost by the
/// box, and one of fewer by the pixel:
///
/// 1. From `APART` boxes up, each run draws in a pass of its own, which
///    loads what the pass before it stored. On Quest a pass of one draw
///    runs in a mode that costs less a box and more a pixel. A second draw
///    in a pass, even of one box, cost S5 0.65 ms over its 4096 boxes and
///    S2 2.8 ms over its 25,301. The passes between cost under 0.02 ms
///    each. S1's 426 boxes ran 10% faster in one pass
/// 2. From `APART` boxes up, a run of front faces draws each box's three
///    faces toward the view: 6 triangles where its corners make 12. S5's
///    23,296 boxes ran 13% faster and S2's 4%. S1 shaded a tenth more
///    fragments that way and ran 19% slower
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
    /// Draws the run over its boxes' corners.
    corners: RenderPipeline,

    /// Draws a run of front faces over each box's three faces toward the
    /// view.
    fronts: Option<RenderPipeline>,

    boxes: Range<u32>,
}

impl BoxDraw {
    /// The bytes of a box's three `vec4<u32>`s.
    pub const BOX_BYTES: usize = 48;

    /// Builds the pipelines over `module`, whose vertex entries take a
    /// vertex's bits and a box's three `vec4<u32>`s. `place` and
    /// `place_near` take a corner of the box, and `place_front` takes a
    /// vertex of one of its three faces toward the view.
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

            let (corners, fronts) = match part.faces {
                BoxFaces::Front => (
                    shared("place", part.fragment, false),
                    Some(shared("place_front", part.fragment, false)),
                ),
                BoxFaces::Back => (shared("place", part.fragment, true), None),
                BoxFaces::NearBack => (shared("place_near", part.fragment, true), None),
            };

            runs.push(Run {
                corners,
                fronts,
                boxes: first..end,
            });

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

    /// The boxes from which a draw counts its cost by the box.
    pub const APART: u32 = 1024;

    /// How many passes draw the boxes: one for each run that holds boxes
    /// from [`APART`](Self::APART) boxes up, and one below that.
    pub fn passes(&self) -> u32 {
        if self.apart() {
            u32::try_from(self.drawn().count()).expect("a draw's runs number under 2^32")
        } else {
            1
        }
    }

    /// Draws the `at`th of the draw's [`passes`](Self::passes) into `pass`.
    /// Draws nothing past the last of them.
    pub fn draw(&self, pass: &mut RenderPass<'_>, at: u32) {
        let (first, count) = match (self.apart(), at) {
            (true, at) => (at as usize, 1),
            (false, 0) => (0, usize::MAX),
            (false, _) => (0, 0),
        };

        // A box's corners make the first triangles of `indices`, and its
        // three faces toward the view make the rest.
        let corners = 0..TRIANGLES.len() as u32;
        let fronts = corners.end..corners.end + FRONTS.len() as u32;
        let apart = self.apart();

        for (drawn, run) in self.drawn().skip(first).take(count).enumerate() {
            if drawn == 0 {
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.corners.slice(..));
                pass.set_vertex_buffer(1, self.boxes.slice(..));
                pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint16);
            }

            let (pipeline, indices) = match &run.fronts {
                Some(pipeline) if apart => (pipeline, fronts.clone()),
                _ => (&run.corners, corners.clone()),
            };

            pass.set_pipeline(pipeline);
            pass.draw_indexed(indices, 0, run.boxes.clone());
        }
    }

    /// Whether the draw holds [`APART`](Self::APART) boxes or more.
    fn apart(&self) -> bool {
        self.parts
            .iter()
            .map(|run| run.boxes.end - run.boxes.start)
            .sum::<u32>()
            >= Self::APART
    }

    /// The runs that hold boxes, in draw order.
    fn drawn(&self) -> impl Iterator<Item = &Run> {
        self.parts.iter().filter(|run| !run.boxes.is_empty())
    }
}
