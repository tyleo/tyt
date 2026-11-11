use crate::{
    BoxArm, BoxFaces, Error, HitPacking, HitTargets, RasterArm, Result, Tree64, live_boxes,
    world_to_clip,
};
use branded_id::U32Id;
use std::{collections::HashMap, num::NonZeroU64};
use ty_math::{TyMatrix4x4F64, TyVector3F64, TyVector3U32};
use voxrender::{
    BRenderView, Error as RenderError, RenderPlacement, RenderProjection, RenderScene, RenderView,
    RenderViewRays, grid_point,
};
use voxrender_wgpu::{GRID_WALK_WGSL, PixelHit, placement_bytes};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ColorTargetState,
    ColorWrites, CommandEncoder, CompareFunction, DepthBiasState, DepthStencilState, Device, Face,
    FragmentState, FrontFace, IndexFormat, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, QuerySet, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderRuntimeChecks, ShaderSource,
    ShaderStages, StencilState, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The cells along each side of the tile that each box bounds.
const TILE: u32 = 16;

/// One pass draws each eye.
const PASSES: RasterArm = RasterArm {
    multiview: false,
    samples: 1,
};

/// The cells past a box's cells within which a ray's start counts as inside
/// the box. The margin covers `box_raster.wgsl`'s pad and the near plane.
const MARGIN: f64 = 1.0 / 32.0;

/// A box's 12 triangles over its 8 corners, wound counterclockwise from
/// outside. Bit `a` of a corner picks the box's high face on axis `a`.
const TRIANGLES: [u16; 36] = [
    0, 4, 6, 0, 6, 2, 1, 3, 7, 1, 7, 5, 0, 1, 5, 0, 5, 4, 2, 6, 7, 2, 7, 3, 0, 2, 3, 0, 3, 1, 4, 5,
    7, 4, 7, 6,
];

/// A raster of one view over two `width` by `height` eyes. It draws the box
/// around each tile's live cells with a march in each fragment. Each pixel's
/// texel holds its first hit: the placement, the cell, and the
/// face. Both eyes draw the scene's view, as B2's traces do.
pub struct BoxRaster {
    /// Draws the front faces of the boxes before `fronts`.
    front: Option<RenderPipeline>,

    /// Draws the back faces of the boxes from `fronts` on.
    back: RenderPipeline,

    bind_group: BindGroup,

    corners: Buffer,

    indices: Buffer,

    /// Two `vec4<u32>`s per box: its low cell and its placement's index,
    /// then the corner past its high cell.
    boxes: Buffer,

    fronts: u32,

    count: u32,

    packing: HitPacking,

    targets: HitTargets,
}

/// One placement's box around a tile's live cells.
struct PlacedBox {
    placement: u32,

    low: TyVector3U32,

    high: TyVector3U32,

    /// Whether a ray of the view can start inside the box.
    inside: bool,

    /// How far the box's nearest point lies from the view.
    distance: f64,
}

impl BoxRaster {
    /// Builds the raster of `scene` through `view_id` over two `width` by
    /// `height` eyes as `arm` draws it. Errors unless the device has the
    /// arm's features and each hit packs into 32 bits.
    pub fn new(
        device: &Device,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        width: u32,
        height: u32,
        arm: BoxArm,
    ) -> Result<Self> {
        let missing = arm.features() - device.features();

        if !missing.is_empty() {
            return Err(Error::RasterFeatures(missing));
        }

        let targets = HitTargets::new(device, width, height, PASSES)?;
        let packing = HitPacking::new(scene)?;

        let view = scene
            .view(view_id)
            .ok_or(Error::Render(RenderError::UnknownView { view_id }))?;

        let view_rays = RenderViewRays::new(view, width, height);
        let grid_rays = view_rays.to_scene_grid_rays(scene).map_err(Error::Render)?;
        let world_to_clip = world_to_clip(scene, view, width, height);

        let mut words = Vec::new();
        let mut cell_offsets = HashMap::new();
        let mut object_boxes = HashMap::new();

        for (object_id, object) in scene.iter_objects() {
            cell_offsets.insert(
                object_id,
                u32::try_from(words.len()).expect("a scene's cells number under 2^32"),
            );

            words.extend(Tree64::new(object)?.words());
            object_boxes.insert(object_id, live_boxes(object, TILE));
        }

        let mut placed = Vec::new();
        let mut clips = Vec::new();

        for (index, (_, placement)) in (0u32..).zip(scene.iter_placements()) {
            let transform = &placement.transform;
            let grid_to_world = TyMatrix4x4F64::from_scale_rotation_translation(
                transform.scale,
                transform.rotation,
                transform.position,
            );

            for value in (world_to_clip * grid_to_world).as_mat4().to_cols_array() {
                clips.extend(value.to_le_bytes());
            }

            for &(low, high) in &object_boxes[&placement.object_id] {
                let (inside, distance) = nearest(view, placement, low, high);

                placed.push(PlacedBox {
                    placement: index,
                    low,
                    high,
                    inside,
                    distance,
                });
            }
        }

        let count = u32::try_from(placed.len()).expect("a scene's boxes number under 2^32");

        let fronts = match arm.faces {
            BoxFaces::Back => 0,

            BoxFaces::Front => {
                placed.sort_by(|a, b| {
                    a.inside
                        .cmp(&b.inside)
                        .then(a.distance.total_cmp(&b.distance))
                });

                u32::try_from(placed.iter().filter(|placed| !placed.inside).count())
                    .expect("the boxes number under 2^32")
            }
        };

        let mut boxes = Vec::new();

        for placed in &placed {
            let corners = [placed.low.extend(placed.placement), placed.high.extend(0)];

            for value in corners.iter().flat_map(|corner| corner.to_array()) {
                boxes.extend(value.to_le_bytes());
            }
        }

        let mut frame = Vec::new();

        for value in [packing.voxel_bits, 0, 0, 0] {
            frame.extend(value.to_le_bytes());
        }

        let steps = view_rays
            .origin_steps()
            .into_iter()
            .chain(view_rays.direction_steps())
            .map(|step| step.extend(0.0));

        for vector in steps.chain([world_to_clip.row(2), world_to_clip.row(3)]) {
            for value in vector.to_array() {
                frame.extend((value as f32).to_le_bytes());
            }
        }

        let placement_cells: Vec<_> = scene
            .iter_placements()
            .map(|(_, placement)| cell_offsets[&placement.object_id])
            .collect();

        // A binding cannot be empty.
        if clips.is_empty() {
            clips.resize(64, 0);
        }

        if words.is_empty() {
            words.push(0);
        }

        if boxes.is_empty() {
            boxes.resize(32, 0);
        }

        let init = |label, contents: &[u8], usage| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
        };

        let bytes = |words: &[u32]| -> Vec<u8> {
            words.iter().flat_map(|word| word.to_le_bytes()).collect()
        };

        let frame = init("box frame", &frame, BufferUsages::UNIFORM);
        let placements = init(
            "box placements",
            &placement_bytes(scene, &grid_rays, &placement_cells),
            BufferUsages::STORAGE,
        );
        let cells = init("box cells", &bytes(&words), BufferUsages::STORAGE);
        let clips = init("box clips", &clips, BufferUsages::STORAGE);
        let corners = init(
            "box corners",
            &bytes(&[0, 1, 2, 3, 4, 5, 6, 7]),
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
        let boxes = init("boxes", &boxes, BufferUsages::VERTEX);

        // SAFETY: each march reads only within its placement's tree, and
        // each of its loops ends within a box's extent. The rest of the
        // module indexes its bindings by a box's placement.
        let module = unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some("box raster"),
                    source: ShaderSource::Wgsl(
                        format!(
                            "const FACE_BITS: u32 = {}u;\n{GRID_WALK_WGSL}\n{}\n{}\n{}\n{}",
                            HitPacking::FACE_BITS,
                            include_str!("tree_walk.wgsl"),
                            arm.march.wgsl(),
                            include_str!("box_raster.wgsl"),
                            match arm.faces {
                                BoxFaces::Back => "",
                                BoxFaces::Front => include_str!("box_fronts.wgsl"),
                            }
                        )
                        .into(),
                    ),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        let entry = |binding, visibility, ty| BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None::<NonZeroU64>,
            },
            count: None,
        };

        let storage = BufferBindingType::Storage { read_only: true };

        // Both pipelines share the layout, so one bind group serves both.
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("box raster"),
            entries: &[
                entry(0, ShaderStages::FRAGMENT, BufferBindingType::Uniform),
                entry(1, ShaderStages::FRAGMENT, storage),
                entry(2, ShaderStages::FRAGMENT, storage),
                entry(3, ShaderStages::VERTEX, storage),
            ],
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
                    module: &module,
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
                    module: &module,
                    entry_point: Some(fragment),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format: HitTargets::FORMAT,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask: targets.multiview_mask(),
                cache: None,
            })
        };

        let front = match arm.faces {
            BoxFaces::Back => None,
            BoxFaces::Front => Some(pipeline("box fronts", "front", Face::Back)),
        };
        let back = pipeline("box backs", "back", Face::Front);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("box raster"),
            layout: &bind_group_layout,
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
                    resource: clips.as_entire_binding(),
                },
            ],
        });

        Ok(BoxRaster {
            front,
            back,
            bind_group,
            corners,
            indices,
            boxes,
            fronts,
            count,
            packing,
            targets,
        })
    }

    /// Records the raster of both eyes into `encoder`, between the
    /// timestamps at `first` and `first + 1` of `query_set`.
    pub fn record(&self, encoder: &mut CommandEncoder, query_set: &QuerySet, first: u32) {
        self.targets
            .draw(encoder, Some((query_set, first)), |pass, _| self.draw(pass));
    }

    /// Draws once and reads back the first eye's hit through each pixel,
    /// row by row, with no shadow bits.
    pub fn trace(&self, device: &Device, queue: &Queue) -> Result<Vec<Option<PixelHit>>> {
        let words = self
            .targets
            .read(device, queue, |pass, _| self.draw(pass))?;

        Ok(words
            .into_iter()
            .map(|word| self.packing.unpack(word))
            .collect())
    }

    /// Draws every box into `pass`.
    fn draw(&self, pass: &mut RenderPass<'_>) {
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

/// Whether a ray of `view` can start inside the box from `low` to `high` in
/// `placement`'s grid, and how far the box's nearest point lies from the
/// view: from a perspective view's eye, or along an orthographic view's
/// axis.
fn nearest(
    view: &RenderView,
    placement: &RenderPlacement,
    low: TyVector3U32,
    high: TyVector3U32,
) -> (bool, f64) {
    let transform = &placement.transform;
    let low = low.as_dvec3() - MARGIN;
    let high = high.as_dvec3() + MARGIN;
    let position = view.pose.position;

    match view.projection {
        RenderProjection::Perspective { .. } => {
            let eye = grid_point(transform, position);
            let nearest = eye.clamp(low, high);

            (
                nearest == eye,
                (transform.transform_point(nearest) - position).length(),
            )
        }

        RenderProjection::Orthographic { .. } => {
            let forward = view.pose.rotation * -TyVector3F64::Z;

            let depth = (0..8)
                .map(|corner| {
                    let point = TyVector3F64::new(
                        if corner & 1 == 0 { low.x } else { high.x },
                        if corner & 2 == 0 { low.y } else { high.y },
                        if corner & 4 == 0 { low.z } else { high.z },
                    );

                    forward.dot(transform.transform_point(point) - position)
                })
                .fold(f64::INFINITY, f64::min);

            (depth <= 0.0, depth)
        }
    }
}
