use crate::{Error, HitPacking, HitTargets, RasterArm, RasterMesh, Result, world_to_clip};
use branded_id::U32Id;
use std::ops::Range;
use ty_math::TyMatrix4x4F64;
use voxrender::{BRenderView, Error as RenderError, RenderScene};
use voxrender_wgpu::PixelHit;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferAddress, BufferUsages,
    ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, DepthBiasState,
    DepthStencilState, Device, Face, FragmentState, FrontFace, IndexFormat, MultisampleState,
    PipelineCompilationOptions, PrimitiveState, QuerySet, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderRuntimeChecks, ShaderSource,
    StencilState, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The bytes of one of [`RasterMesh::vertices`]' vertices.
const VERTEX_BYTES: BufferAddress = 16;

/// A greedy-mesh raster of one view over two `width` by `height` eyes. Each
/// pixel's texel holds the first hit the march would report: the
/// placement, the cell under the hit point, and the face. Both eyes draw
/// the scene's view, as B2's traces do.
pub struct MeshRaster {
    pipeline: RenderPipeline,

    bind_group: BindGroup,

    vertices: Buffer,

    indices: Buffer,

    draws: Vec<(Range<u32>, i32)>,

    placements: u32,

    packing: HitPacking,

    targets: HitTargets,
}

impl MeshRaster {
    /// Builds the raster of `mesh`, a mesh of `scene`, through `view_id`
    /// over two `width` by `height` eyes as `arm` draws them. Errors unless
    /// the device has the arm's features.
    pub fn new(
        device: &Device,
        mesh: &RasterMesh,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        width: u32,
        height: u32,
        arm: RasterArm,
    ) -> Result<Self> {
        let targets = HitTargets::new(device, width, height, arm)?;

        let view = scene
            .view(view_id)
            .ok_or(Error::Render(RenderError::UnknownView { view_id }))?;

        let world_to_clip = world_to_clip(scene, view, width, height);

        let mut clips = Vec::new();

        for _eye in 0..2 {
            for (_, placement) in scene.iter_placements() {
                let transform = &placement.transform;
                let grid_to_world = TyMatrix4x4F64::from_scale_rotation_translation(
                    transform.scale,
                    transform.rotation,
                    transform.position,
                );

                for value in (world_to_clip * grid_to_world).as_mat4().to_cols_array() {
                    clips.extend(value.to_le_bytes());
                }
            }
        }

        // A binding cannot be empty.
        if clips.is_empty() {
            clips.resize(64, 0);
        }

        let placements = u32::try_from(scene.iter_placements().count())
            .expect("a scene's placements number under 2^32");

        let mut frame = Vec::new();

        for value in [mesh.packing.voxel_bits, placements, 0, 0] {
            frame.extend(value.to_le_bytes());
        }

        let init = |label, contents: &[u8], usage| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
        };

        let frame = init("raster frame", &frame, BufferUsages::UNIFORM);
        let clips = init("raster clips", &clips, BufferUsages::STORAGE);

        let entry = if arm.multiview {
            include_str!("raster_views.wgsl")
        } else {
            include_str!("raster_eyes.wgsl")
        };

        // SAFETY: each draw's instance picks one of the scene's placements in
        // one of two eyes, so every read stays within its binding. The
        // shaders have no loops.
        let module = unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some("raster"),
                    source: ShaderSource::Wgsl(
                        format!(
                            "const FACE_BITS: u32 = {}u;\n{}\n{entry}",
                            HitPacking::FACE_BITS,
                            include_str!("raster.wgsl")
                        )
                        .into(),
                    ),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("raster"),
            layout: None,
            vertex: VertexState {
                module: &module,
                entry_point: Some("place"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Some(VertexBufferLayout {
                    array_stride: VERTEX_BYTES,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Uint32x4,
                        offset: 0,
                        shader_location: 0,
                    }],
                })],
            },
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                ..PrimitiveState::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: HitTargets::DEPTH,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Greater),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: arm.samples,
                ..MultisampleState::default()
            },
            fragment: Some(FragmentState {
                module: &module,
                entry_point: Some("hit"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: HitTargets::FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: targets.multiview_mask(),
            cache: None,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("raster"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: frame.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: clips.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: mesh.bounds.as_entire_binding(),
                },
            ],
        });

        Ok(MeshRaster {
            pipeline,
            bind_group,
            vertices: mesh.vertices.clone(),
            indices: mesh.indices.clone(),
            draws: mesh.draws.clone(),
            placements,
            packing: mesh.packing,
            targets,
        })
    }

    /// Records the raster of both eyes into `encoder`, between the
    /// timestamps at `first` and `first + 1` of `query_set`.
    pub fn record(&self, encoder: &mut CommandEncoder, query_set: &QuerySet, first: u32) {
        self.targets
            .draw(encoder, Some((query_set, first)), 1, |pass, eye, _| {
                self.draw(pass, eye)
            });
    }

    /// Draws once and reads back the first eye's hit through each pixel,
    /// row by row, with no shadow bits. An MSAA arm's resolve blends the
    /// texels of a pixel that a quad's edge crosses.
    pub fn trace(&self, device: &Device, queue: &Queue) -> Result<Vec<Option<PixelHit>>> {
        let words = self
            .targets
            .read(device, queue, 1, |pass, eye, _| self.draw(pass, eye))?;

        Ok(words
            .into_iter()
            .map(|word| self.packing.unpack(word))
            .collect())
    }

    /// Draws every placement into `pass` as `eye` sees it.
    fn draw(&self, pass: &mut RenderPass<'_>, eye: u32) {
        // A buffer slice cannot be empty.
        if self.vertices.size() == 0 {
            return;
        }

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint32);

        // The instance picks the placement's matrix.
        for (placement, (indices, first)) in (0u32..).zip(&self.draws) {
            let instance = eye * self.placements + placement;

            pass.draw_indexed(indices.clone(), *first, instance..instance + 1);
        }
    }
}
