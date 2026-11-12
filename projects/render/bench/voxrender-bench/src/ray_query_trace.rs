use crate::{Error, Occupancy, QuadTlas, RayQueryArm, Result, Tree64};
use branded_id::U32Id;
use std::{collections::HashMap, sync::mpsc};
use ty_math::{TyVector3F64, TyVector3U32};
use voxcore::BVoxObject;
use voxrender::{BRenderView, Error as RenderError, RenderObject, RenderScene, RenderViewRays};
use voxrender_wgpu::{GRID_WALK_WGSL, PixelHit, dense_cells, placement_bytes};
use voxsurface::{SurfaceMethod, mesh_grid};
use wgpu::{
    AccelerationStructureFlags, AccelerationStructureGeometryFlags,
    AccelerationStructureUpdateMode, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindingResource, Blas, BlasAABBGeometrySizeDescriptor, BlasAabbGeometry, BlasBuildEntry,
    BlasGeometries, BlasGeometrySizeDescriptors, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, CreateBlasDescriptor, CreateTlasDescriptor, Device, Features,
    MapMode, PipelineCompilationOptions, PollType, Queue, ShaderModuleDescriptor,
    ShaderRuntimeChecks, ShaderSource, Tlas, TlasInstance,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The shader's marker for a pixel without a hit.
const NO_HIT: u32 = u32::MAX;

/// The cells each brick's AABB reaches past the brick, so a brick the f32
/// world ray only grazes still reaches the integer march.
const PAD: f32 = 1.0 / 64.0;

/// The bytes of one brick's AABB: two `vec3<f32>` corners.
const AABB_BYTES: u64 = 24;

/// One view's first hits through a hardware ray query over a TLAS of the
/// scene's placements, uploaded once and dispatched as often as its caller
/// records it. The scene's lights cast no shadow rays.
pub struct RayQueryTrace {
    pipeline: ComputePipeline,

    bind_group: BindGroup,

    hits: Buffer,

    width: u32,

    height: u32,
}

impl RayQueryTrace {
    /// Uploads `scene`, its bricks, and the rays of `view_id` over a `width`
    /// by `height` image, and builds the acceleration structures and the
    /// pipeline of `arm`. Each object's BLAS holds one AABB per non-empty
    /// brick. A candidate marches its brick. Errors unless the device has ray
    /// queries.
    pub fn new(
        device: &Device,
        queue: &Queue,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        width: u32,
        height: u32,
        arm: RayQueryArm,
    ) -> Result<Self> {
        if !device.features().contains(Features::EXPERIMENTAL_RAY_QUERY) {
            return Err(Error::NoRayQuery);
        }

        let view = scene
            .view(view_id)
            .ok_or(Error::Render(RenderError::UnknownView { view_id }))?;

        let view_rays = RenderViewRays::new(view, width, height);
        let grid_rays = view_rays.to_scene_grid_rays(scene).map_err(Error::Render)?;

        let mut words = Vec::new();
        let mut cell_offsets = HashMap::new();
        let mut brick_words = Vec::new();
        let mut brick_offsets = HashMap::new();
        let mut blases = HashMap::new();

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ray query structures"),
        });

        let mut aabbs = Vec::new();

        for (object_id, object) in scene.iter_objects() {
            cell_offsets.insert(
                object_id,
                u32::try_from(words.len()).expect("a scene's cells number under 2^32"),
            );

            words.extend(Tree64::new(object)?.words());

            let corners = bricks(object, arm.brick);

            if corners.is_empty() {
                continue;
            }

            brick_offsets.insert(
                object_id,
                u32::try_from(brick_words.len() / 8).expect("a scene's bricks number under 2^32"),
            );

            let mut bytes = Vec::new();

            for corner in &corners {
                let extent = (*corner + arm.brick).min(object.bounds()) - *corner;

                for value in corner.to_array().into_iter().chain([0]) {
                    brick_words.push(value);
                }

                for value in extent.to_array().into_iter().chain([0]) {
                    brick_words.push(value);
                }

                for value in corner.as_vec3().to_array().map(|c| c - PAD) {
                    bytes.extend(value.to_le_bytes());
                }

                for value in (*corner + extent).as_vec3().to_array().map(|c| c + PAD) {
                    bytes.extend(value.to_le_bytes());
                }
            }

            let size = BlasAABBGeometrySizeDescriptor {
                primitive_count: u32::try_from(corners.len())
                    .expect("an object's bricks number under 2^32"),
                flags: AccelerationStructureGeometryFlags::OPAQUE,
            };

            let blas = device.create_blas(
                &CreateBlasDescriptor {
                    label: Some("bricks"),
                    flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
                    update_mode: AccelerationStructureUpdateMode::Build,
                },
                BlasGeometrySizeDescriptors::AABBs {
                    descriptors: vec![size.clone()],
                },
            );

            let buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("brick aabbs"),
                contents: &bytes,
                usage: BufferUsages::BLAS_INPUT,
            });

            aabbs.push((object_id, size, buffer));
            blases.insert(object_id, blas);
        }

        let entries: Vec<_> = aabbs
            .iter()
            .map(|(object_id, size, buffer)| BlasBuildEntry {
                blas: &blases[object_id],
                geometry: BlasGeometries::AabbGeometries(vec![BlasAabbGeometry {
                    size,
                    stride: AABB_BYTES,
                    aabb_buffer: buffer,
                    primitive_offset: 0,
                }]),
            })
            .collect();

        let tlas = tlas_of(device, scene, &blases);

        encoder.build_acceleration_structures(&entries, [&tlas]);
        queue.submit([encoder.finish()]);

        let placement_cells: Vec<_> = scene
            .iter_placements()
            .map(|(_, placement)| cell_offsets[&placement.object_id])
            .collect();

        let first_bricks: Vec<u32> = scene
            .iter_placements()
            .map(|(_, placement)| {
                brick_offsets
                    .get(&placement.object_id)
                    .copied()
                    .unwrap_or(0)
            })
            .collect();

        let frame = frame_bytes(&view_rays, width, height);

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

        let frame = init("frame", &frame, BufferUsages::UNIFORM);
        let placements = init(
            "placements",
            &placement_bytes(scene, &grid_rays, &placement_cells),
            BufferUsages::STORAGE,
        );
        let cells = init("cells", &bytes(&words), BufferUsages::STORAGE);

        // A binding cannot be empty.
        if brick_words.is_empty() {
            brick_words.resize(8, 0);
        }

        let bricks = init("bricks", &bytes(&brick_words), BufferUsages::STORAGE);
        let first_bricks = init("first bricks", &bytes(&first_bricks), BufferUsages::STORAGE);

        let hits = device.create_buffer(&BufferDescriptor {
            label: Some("hits"),
            size: u64::from(width) * u64::from(height) * 16,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // SAFETY: each march reads only within its placements' cells and
        // bricks, and its loops end within a grid's extent. Each query
        // proceeds before it reads a candidate, and its rays hold finite
        // values.
        let module = unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some("ray_query"),
                    source: ShaderSource::Wgsl(
                        format!(
                            "{}\n{GRID_WALK_WGSL}\n{}\n{}",
                            include_str!("ray_query.wgsl"),
                            include_str!("tree_walk.wgsl"),
                            arm.march.wgsl()
                        )
                        .into(),
                    ),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("ray_query"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ray_query"),
            layout: &pipeline.get_bind_group_layout(0),
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
                    resource: bricks.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: first_bricks.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::AccelerationStructure(&tlas),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: hits.as_entire_binding(),
                },
            ],
        });

        Ok(RayQueryTrace {
            pipeline,
            bind_group,
            hits,
            width,
            height,
        })
    }

    /// Uploads `scene` and the rays of `view_id` over a `width` by `height`
    /// image, with each object's greedy quads as the triangles of its BLAS.
    /// The ray tracing unit then finds a ray's first hit, and no march
    /// runs. Errors unless the device has ray queries.
    pub fn triangles(
        device: &Device,
        queue: &Queue,
        scene: &RenderScene,
        view_id: U32Id<BRenderView>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        if !device.features().contains(Features::EXPERIMENTAL_RAY_QUERY) {
            return Err(Error::NoRayQuery);
        }

        let view = scene
            .view(view_id)
            .ok_or(Error::Render(RenderError::UnknownView { view_id }))?;

        let view_rays = RenderViewRays::new(view, width, height);

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

        let quads = QuadTlas::new(
            device,
            queue,
            scene,
            |object| mesh_grid(&Occupancy(object), SurfaceMethod::Greedy),
            |placement, _| placement,
        );

        let tlas = quads.tlas;
        let quad_offsets = quads.first_quads;
        let mut faces = quads.faces;

        let mut bounds: Vec<u32> = Vec::new();
        let mut first_quads: Vec<u32> = Vec::new();

        for (_, placement) in scene.iter_placements() {
            let object = scene
                .object(placement.object_id)
                .expect("a placement's object is one of the scene's");

            bounds.extend(object.bounds().to_array().into_iter().chain([0]));
            first_quads.push(quad_offsets.get(&placement.object_id).copied().unwrap_or(0));
        }

        // A binding cannot be empty.
        for words in [&mut bounds, &mut faces, &mut first_quads] {
            if words.is_empty() {
                words.resize(4, 0);
            }
        }

        let frame = init(
            "frame",
            &frame_bytes(&view_rays, width, height),
            BufferUsages::UNIFORM,
        );
        let bounds = init("bounds", &bytes(&bounds), BufferUsages::STORAGE);
        let faces = init("faces", &bytes(&faces), BufferUsages::STORAGE);
        let first_quads = init("first quads", &bytes(&first_quads), BufferUsages::STORAGE);

        let hits = device.create_buffer(&BufferDescriptor {
            label: Some("hits"),
            size: u64::from(width) * u64::from(height) * 16,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // SAFETY: a hit's placement indexes the per-placement buffers, and its
        // primitive lies within its object's quads. The query proceeds before
        // its hit is read, and its rays hold finite values.
        let module = unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some("triangle_query"),
                    source: ShaderSource::Wgsl(include_str!("triangle_query.wgsl").into()),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("triangle_query"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("triangle_query"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: frame.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: bounds.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: faces.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: first_quads.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::AccelerationStructure(&tlas),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: hits.as_entire_binding(),
                },
            ],
        });

        Ok(RayQueryTrace {
            pipeline,
            bind_group,
            hits,
            width,
            height,
        })
    }

    /// Records the trace of every pixel into `pass`.
    pub fn dispatch(&self, pass: &mut ComputePass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.width.div_ceil(8), self.height.div_ceil(8), 1);
    }

    /// Traces once and reads back the first hit through each pixel, row by
    /// row, with no shadow bits.
    pub fn trace(&self, device: &Device, queue: &Queue) -> Result<Vec<Option<PixelHit>>> {
        let readback = device.create_buffer(&BufferDescriptor {
            label: Some("hit readback"),
            size: self.hits.size(),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ray_query"),
        });

        self.dispatch(&mut encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("ray_query"),
            timestamp_writes: None,
        }));

        encoder.copy_buffer_to_buffer(&self.hits, 0, &readback, 0, None);
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

        let words: Vec<_> = view
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().expect("a chunk holds four bytes")))
            .collect();

        Ok(words
            .chunks_exact(4)
            .map(|hit| {
                (hit[0] != NO_HIT).then(|| PixelHit {
                    placement: hit[0],
                    voxel: hit[1],
                    face: hit[2],
                    lit: Vec::new(),
                })
            })
            .collect())
    }
}

/// A TLAS of `scene`'s placements whose objects have a BLAS in `blases`. An
/// instance's custom data holds its placement's index.
fn tlas_of(
    device: &Device,
    scene: &RenderScene,
    blases: &HashMap<U32Id<BVoxObject>, Blas>,
) -> Tlas {
    let instanced: Vec<_> = (0..)
        .zip(scene.iter_placements())
        .filter(|(_, (_, placement))| blases.contains_key(&placement.object_id))
        .collect();

    let mut tlas = device.create_tlas(&CreateTlasDescriptor {
        label: Some("placements"),
        max_instances: u32::try_from(instanced.len().max(1)).expect("placements number under 2^32"),
        flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: AccelerationStructureUpdateMode::Build,
    });

    for (slot, (index, (_, placement))) in instanced.iter().enumerate() {
        let transform = &placement.transform;

        // Grid cells to world: rotation times scale, then position.
        let columns = [TyVector3F64::X, TyVector3F64::Y, TyVector3F64::Z]
            .map(|axis| transform.rotation * axis);
        let scale = transform.scale.to_array();
        let position = transform.position.to_array();
        let mut rows = [0.0; 12];

        for row in 0..3 {
            for column in 0..3 {
                rows[row * 4 + column] = (columns[column].to_array()[row] * scale[column]) as f32;
            }

            rows[row * 4 + 3] = position[row] as f32;
        }

        tlas[slot] = Some(TlasInstance::new(
            &blases[&placement.object_id],
            rows,
            *index,
            0xff,
        ));
    }

    tlas
}

/// The bytes of a trace's `Frame`: the image's size, then pixel (0, 0)'s
/// world ray and its steps along a row and down a column.
fn frame_bytes(view_rays: &RenderViewRays, width: u32, height: u32) -> Vec<u8> {
    let mut frame = Vec::new();

    for value in [width, height, 0, 0] {
        frame.extend(value.to_le_bytes());
    }

    for vector in view_rays
        .origin_steps()
        .into_iter()
        .chain(view_rays.direction_steps())
    {
        for value in vector.to_array().into_iter().chain([0.0]) {
            frame.extend((value as f32).to_le_bytes());
        }
    }

    frame
}

/// The corners of `object`'s non-empty bricks of `brick` cells a side, in
/// raster order.
fn bricks(object: &RenderObject, brick: u32) -> Vec<TyVector3U32> {
    let bounds = object.bounds();
    let counts = (bounds + brick - 1) / brick;
    let cells = dense_cells(object);
    let mut live = vec![false; (counts.x * counts.y * counts.z) as usize];

    for x in 0..bounds.x {
        for y in 0..bounds.y {
            for z in 0..bounds.z {
                if cells[((x * bounds.y + y) * bounds.z + z) as usize] != 0 {
                    let at = TyVector3U32::new(x, y, z) / brick;

                    live[((at.x * counts.y + at.y) * counts.z + at.z) as usize] = true;
                }
            }
        }
    }

    let mut corners = Vec::new();

    for x in 0..counts.x {
        for y in 0..counts.y {
            for z in 0..counts.z {
                if live[((x * counts.y + y) * counts.z + z) as usize] {
                    corners.push(TyVector3U32::new(x, y, z) * brick);
                }
            }
        }
    }

    corners
}
