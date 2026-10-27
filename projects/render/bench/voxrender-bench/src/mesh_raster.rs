use crate::{Error, RasterArm, RasterMesh, Result};
use branded_id::U32Id;
use std::{num::NonZeroU32, ops::Range, sync::mpsc};
use ty_math::{TyMatrix4x4F64, TyVector3F64};
use voxrender::{
    BRenderView, Error as RenderError, RenderProjection, RenderScene, RenderView, RenderViewRays,
};
use voxrender_wgpu::PixelHit;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferAddress, BufferDescriptor,
    BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, Color, ColorTargetState, ColorWrites,
    CommandEncoder, CommandEncoderDescriptor, CompareFunction, DepthBiasState, DepthStencilState,
    Device, Extent3d, Face, FragmentState, FrontFace, IndexFormat, LoadOp, MapMode,
    MultisampleState, Operations, Origin3d, PipelineCompilationOptions, PollType, PrimitiveState,
    QuerySet, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPassTimestampWrites, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderRuntimeChecks, ShaderSource, StencilState, StoreOp,
    TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The format the raster writes. Each texel holds a packed hit in the four
/// bytes a shaded pixel would take.
const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

const DEPTH: TextureFormat = TextureFormat::Depth32Float;

/// The clear's texel, which a pixel without a hit keeps.
const NO_HIT: u32 = u32::MAX;

/// The perspective's near plane in world units. Reversed depth keeps f32's
/// relative precision at every distance. The plane can then sit far nearer
/// than any cell.
const NEAR: f64 = 1.0 / 1024.0 / 1024.0;

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

    voxel_bits: u32,

    multiview_mask: Option<NonZeroU32>,

    /// Both eyes' texels, one layer each.
    output: Texture,

    /// One target per pass.
    targets: Vec<RasterTarget>,

    width: u32,

    height: u32,
}

/// What one pass draws into.
struct RasterTarget {
    color: TextureView,

    /// The single-sample view an MSAA pass resolves into.
    resolve: Option<TextureView>,

    depth: TextureView,
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
        let missing = arm.features() - device.features();

        if !missing.is_empty() {
            return Err(Error::RasterFeatures(missing));
        }

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

        for value in [mesh.voxel_bits, placements, 0, 0] {
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
                            RasterMesh::FACE_BITS,
                            include_str!("raster.wgsl")
                        )
                        .into(),
                    ),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        let multiview_mask = NonZeroU32::new(if arm.multiview { 0b11 } else { 0 });

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
                format: DEPTH,
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
                    format: FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask,
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

        let texture = |label, layers, samples, format, usage| {
            device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: layers,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };

        let output = texture(
            "raster eyes",
            2,
            1,
            FORMAT,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        );

        // A tiler keeps the samples and depth in tile memory because each
        // pass clears and discards them. They skip transient textures because
        // wgpu's hold one layer.
        let samples = (arm.samples > 1).then(|| {
            texture(
                "raster samples",
                2,
                arm.samples,
                FORMAT,
                TextureUsages::RENDER_ATTACHMENT,
            )
        });

        let depth = texture(
            "raster depth",
            if arm.multiview { 2 } else { 1 },
            arm.samples,
            DEPTH,
            TextureUsages::RENDER_ATTACHMENT,
        );

        // The multiview pass draws both layers, and each eye's pass draws
        // one.
        let layers: Vec<Option<u32>> = if arm.multiview {
            vec![None]
        } else {
            vec![Some(0), Some(1)]
        };

        let layer_view = |texture: &Texture, layer: Option<u32>| {
            texture.create_view(&TextureViewDescriptor {
                dimension: Some(match layer {
                    Some(_) => TextureViewDimension::D2,
                    None => TextureViewDimension::D2Array,
                }),
                base_array_layer: layer.unwrap_or(0),
                array_layer_count: Some(if layer.is_some() { 1 } else { 2 }),
                ..TextureViewDescriptor::default()
            })
        };

        let depth = depth.create_view(&TextureViewDescriptor::default());

        let targets = layers
            .into_iter()
            .map(|layer| {
                let resolved = layer_view(&output, layer);

                match &samples {
                    Some(samples) => RasterTarget {
                        color: layer_view(samples, layer),
                        resolve: Some(resolved),
                        depth: depth.clone(),
                    },

                    None => RasterTarget {
                        color: resolved,
                        resolve: None,
                        depth: depth.clone(),
                    },
                }
            })
            .collect();

        Ok(MeshRaster {
            pipeline,
            bind_group,
            vertices: mesh.vertices.clone(),
            indices: mesh.indices.clone(),
            draws: mesh.draws.clone(),
            placements,
            voxel_bits: mesh.voxel_bits,
            multiview_mask,
            output,
            targets,
            width,
            height,
        })
    }

    /// Records the raster of both eyes into `encoder`, between the
    /// timestamps at `first` and `first + 1` of `query_set`.
    pub fn record(&self, encoder: &mut CommandEncoder, query_set: &QuerySet, first: u32) {
        self.draw(encoder, Some((query_set, first)));
    }

    /// Draws once and reads back the first eye's hit through each pixel,
    /// row by row, with no shadow bits. An MSAA arm's resolve blends the
    /// texels of a pixel that a quad's edge crosses.
    pub fn trace(&self, device: &Device, queue: &Queue) -> Result<Vec<Option<PixelHit>>> {
        let row_bytes = (self.width * 4).next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT);

        let readback = device.create_buffer(&BufferDescriptor {
            label: Some("raster readback"),
            size: BufferAddress::from(row_bytes) * BufferAddress::from(self.height),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("raster"),
        });

        self.draw(&mut encoder, None);

        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: &self.output,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &readback,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row_bytes),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit([encoder.finish()]);

        let (sender, receiver) = mpsc::channel();

        readback.map_async(MapMode::Read, .., move |result| {
            sender.send(result).expect("the receiver outlives the wait");
        });

        device
            .poll(PollType::wait_indefinitely())
            .map_err(Error::Poll)?;

        receiver
            .recv()
            .expect("the map callback runs during the wait")
            .map_err(Error::Map)?;

        let view = readback.get_mapped_range(..).map_err(Error::MapRange)?;
        let voxel_mask = (1 << self.voxel_bits) - 1;
        let face_mask = (1 << RasterMesh::FACE_BITS) - 1;

        Ok(view
            .chunks_exact(row_bytes as usize)
            .flat_map(|row| row[..self.width as usize * 4].chunks_exact(4))
            .map(|texel| {
                let word = u32::from_le_bytes(texel.try_into().expect("a texel holds four bytes"));

                (word != NO_HIT).then(|| {
                    let key = word >> self.voxel_bits;

                    PixelHit {
                        placement: key >> RasterMesh::FACE_BITS,
                        voxel: word & voxel_mask,
                        face: key & face_mask,
                        lit: Vec::new(),
                    }
                })
            })
            .collect())
    }

    /// Records every pass into `encoder`. When `timestamps` holds a query set
    /// and a first index, the passes write the timestamps at that index and
    /// the next.
    fn draw(&self, encoder: &mut CommandEncoder, timestamps: Option<(&QuerySet, u32)>) {
        let last = self.targets.len() - 1;

        for (eye, target) in (0u32..).zip(&self.targets) {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("raster"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &target.color,
                    depth_slice: None,
                    resolve_target: target.resolve.as_ref(),
                    ops: Operations {
                        load: LoadOp::Clear(Color::WHITE),
                        store: if target.resolve.is_some() {
                            StoreOp::Discard
                        } else {
                            StoreOp::Store
                        },
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &target.depth,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(0.0),
                        store: StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: timestamps.map(|(query_set, first)| RenderPassTimestampWrites {
                    query_set,
                    beginning_of_pass_write_index: (eye == 0).then_some(first),
                    end_of_pass_write_index: (eye as usize == last).then_some(first + 1),
                }),
                occlusion_query_set: None,
                multiview_mask: self.multiview_mask,
            });

            // A buffer slice cannot be empty.
            if self.vertices.size() == 0 {
                continue;
            }

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint32);

            // The instance picks the placement's matrix. The multiview pass
            // draws as eye 0.
            for (placement, (indices, first)) in (0u32..).zip(&self.draws) {
                let instance = eye * self.placements + placement;

                pass.draw_indexed(indices.clone(), *first, instance..instance + 1);
            }
        }
    }
}

/// The matrix that carries a world point to clip space so that each pixel
/// center samples the ray `RenderViewRays` casts through it.
///
/// A perspective ray reaches `origin + X dx + Y dy + T d` from the steps of
/// [`RenderViewRays::direction_steps`], where `T` is the depth along the
/// view and `(X, Y)` is the pixel times `T`. An orthographic ray reaches
/// `origin + x ox + y oy + t d` from the steps of
/// [`RenderViewRays::origin_steps`], where `t` is its distance. Inverting the
/// steps gives the pixel, and the depth runs reversed.
fn world_to_clip(
    scene: &RenderScene,
    view: &RenderView,
    width: u32,
    height: u32,
) -> TyMatrix4x4F64 {
    let rays = RenderViewRays::new(view, width, height);
    let [origin, origin_x, origin_y] = rays.origin_steps();
    let [direction, direction_x, direction_y] = rays.direction_steps();

    let w = f64::from(width);
    let h = f64::from(height);

    let steps = |x: TyVector3F64, y: TyVector3F64, z: TyVector3F64| {
        TyMatrix4x4F64::from_cols(
            x.extend(0.0),
            y.extend(0.0),
            z.extend(0.0),
            origin.extend(1.0),
        )
        .inverse()
    };

    // Each row gives one clip component.
    let (to_pixels, rows) = match view.projection {
        RenderProjection::Perspective { .. } => (
            steps(direction_x, direction_y, direction),
            [
                [2.0 / w, 0.0, 1.0 / w - 1.0, 0.0],
                [0.0, -2.0 / h, 1.0 - 1.0 / h, 0.0],
                [0.0, 0.0, 0.0, NEAR],
                [0.0, 0.0, 1.0, 0.0],
            ],
        ),

        RenderProjection::Orthographic { .. } => {
            let to_pixels = steps(origin_x, origin_y, direction);

            // The far plane sits past every placement's deepest corner.
            let far = scene
                .iter_placements()
                .flat_map(|(_, placement)| {
                    let bounds = scene
                        .object(placement.object_id)
                        .expect("a placement's object is one of the scene's")
                        .bounds()
                        .as_dvec3();

                    (0..8).map(move |corner| {
                        let point = TyVector3F64::new(
                            if corner & 1 == 0 { 0.0 } else { bounds.x },
                            if corner & 2 == 0 { 0.0 } else { bounds.y },
                            if corner & 4 == 0 { 0.0 } else { bounds.z },
                        );

                        to_pixels
                            .transform_point3(placement.transform.transform_point(point))
                            .z
                    })
                })
                .fold(0.0, f64::max)
                + 1.0;

            (
                to_pixels,
                [
                    [2.0 / w, 0.0, 0.0, 1.0 / w - 1.0],
                    [0.0, -2.0 / h, 0.0, 1.0 - 1.0 / h],
                    [0.0, 0.0, -1.0 / far, 1.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            )
        }
    };

    TyMatrix4x4F64::from_cols_array_2d(&rows).transpose() * to_pixels
}
