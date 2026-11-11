use crate::{BoxFaces, HitTargets, PlacedBox};
use std::num::{NonZeroU32, NonZeroU64};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ColorTargetState,
    ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Device, Face, FragmentState,
    FrontFace, IndexFormat, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    ShaderStages, StencilState, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

/// A box's 12 triangles over its 8 corners, wound counterclockwise from
/// outside. Bit `a` of a corner picks the box's high face on axis `a`.
const TRIANGLES: [u16; 36] = [
    0, 4, 6, 0, 6, 2, 1, 3, 7, 1, 7, 5, 0, 1, 5, 0, 5, 4, 2, 6, 7, 2, 7, 3, 0, 2, 3, 0, 3, 1, 4, 5,
    7, 4, 7, 6,
];

/// A view's boxes and the pipelines that draw their faces into
/// [`HitTargets`].
pub struct BoxDraw {
    /// Draws the front faces of the boxes before `fronts`.
    front: Option<RenderPipeline>,

    /// Draws the back faces of the boxes from `fronts` on.
    back: RenderPipeline,

    bind_group: BindGroup,

    corners: Buffer,

    indices: Buffer,

    /// Two `vec4<u32>`s per box.
    boxes: Buffer,

    fronts: u32,

    count: u32,
}

impl BoxDraw {
    /// Builds the pipelines over `module`, whose vertex entry `place`
    /// takes a corner's bits and a box's two `vec4<u32>`s. Its fragment
    /// entry `back` shades back faces. Under [`BoxFaces::Front`], its
    /// fragment entry `front` shades front faces.
    ///
    /// # Arguments
    ///
    /// 1. `boxes`: the boxes in draw order, of which the first `fronts` draw
    ///    their front faces
    /// 2. `words`: the last word of each of a box's `vec4<u32>`s. The box's
    ///    low cell and the corner past its high cell fill the rest
    /// 3. `bindings`: each binding's stages, type, and buffer, in binding
    ///    order
    /// 4. `multiview_mask`: the mask of the passes the boxes draw in
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &Device,
        module: &ShaderModule,
        faces: BoxFaces,
        boxes: &[PlacedBox],
        fronts: u32,
        words: impl Fn(&PlacedBox) -> [u32; 2],
        bindings: &[(ShaderStages, BufferBindingType, &Buffer)],
        multiview_mask: Option<NonZeroU32>,
    ) -> Self {
        let mut box_bytes = Vec::new();

        for placed in boxes {
            let [low, high] = words(placed);

            for value in placed
                .low
                .extend(low)
                .to_array()
                .into_iter()
                .chain(placed.high.extend(high).to_array())
            {
                box_bytes.extend(value.to_le_bytes());
            }
        }

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
        let box_buffer = init("boxes", &box_bytes, BufferUsages::VERTEX);

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

        // Both pipelines share the layout, so one bind group serves both.
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("box raster"),
            entries: &entries,
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("box raster"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = |label, fragment, culled| {
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: VertexState {
                    module,
                    entry_point: Some("place"),
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
                    cull_mode: Some(culled),
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
                        format: HitTargets::FORMAT,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask,
                cache: None,
            })
        };

        let front = match faces {
            BoxFaces::Back => None,
            BoxFaces::Front => Some(pipeline("box fronts", "front", Face::Back)),
        };

        let back = pipeline("box backs", "back", Face::Front);

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
            front,
            back,
            bind_group,
            corners,
            indices,
            boxes: box_buffer,
            fronts,
            count: u32::try_from(boxes.len()).expect("a scene's boxes number under 2^32"),
        }
    }

    /// Draws every box into `pass`.
    pub fn draw(&self, pass: &mut RenderPass<'_>) {
        if self.count == 0 {
            return;
        }

        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.corners.slice(..));
        pass.set_vertex_buffer(1, self.boxes.slice(..));
        pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint16);

        if let Some(front) = self.front.as_ref().filter(|_| self.fronts > 0) {
            pass.set_pipeline(front);
            pass.draw_indexed(0..36, 0, 0..self.fronts);
        }

        if self.fronts < self.count {
            pass.set_pipeline(&self.back);
            pass.draw_indexed(0..36, 0, self.fronts..self.count);
        }
    }
}
