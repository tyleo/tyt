use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::TyVector3F64;
use voxcore::{BVoxObject, BVoxVoxel};
use voxrender::{RenderObject, RenderScene};
use voxsurface::SurfaceMesh;
use wgpu::{
    AccelerationStructureFlags, AccelerationStructureGeometryFlags,
    AccelerationStructureUpdateMode, BlasBuildEntry, BlasGeometries, BlasGeometrySizeDescriptors,
    BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor, BufferUsages,
    CommandEncoderDescriptor, CreateBlasDescriptor, CreateTlasDescriptor, Device, IndexFormat,
    Queue, Tlas, TlasInstance, VertexFormat,
    util::{BufferInitDescriptor, DeviceExt},
};

/// A scene's placements as a TLAS for a hardware ray query. Each object's
/// BLAS holds the quads of a mesh of its cells as two triangles each. The
/// ray tracing unit then finds a ray's first hit with no march. A hit's
/// primitive halves to its quad among its object's.
pub struct QuadTlas {
    pub tlas: Tlas,

    /// Each object's first quad among [`faces`](Self::faces) and
    /// [`cells`](Self::cells). An object with no quad has no entry and no
    /// instance.
    pub first_quads: HashMap<U32Id<BVoxObject>, u32>,

    /// Each quad's face: its axis doubled, plus one where the face looks up
    /// the axis.
    pub faces: Vec<u32>,

    /// One of the cells each quad covers.
    pub cells: Vec<U32Id<BVoxVoxel>>,
}

impl QuadTlas {
    /// Builds the structures of `scene` and submits their build to `queue`.
    /// The device must have ray queries.
    ///
    /// # Arguments
    ///
    /// 1. `mesh`: the mesh of an object's cells
    /// 2. `custom`: an instance's custom data, under 2^24, from its
    ///    placement's index and its object's first quad
    pub fn new(
        device: &Device,
        queue: &Queue,
        scene: &RenderScene,
        mesh: impl Fn(&RenderObject) -> SurfaceMesh<U32Id<BVoxVoxel>>,
        custom: impl Fn(u32, u32) -> u32,
    ) -> Self {
        let init = |label, contents: &[u8]| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: BufferUsages::BLAS_INPUT,
            })
        };

        let count = |value: usize| u32::try_from(value).expect("a scene's quads number under 2^30");

        let mut faces: Vec<u32> = Vec::new();
        let mut cells = Vec::new();
        let mut first_quads = HashMap::new();
        let mut blases = HashMap::new();
        let mut inputs = Vec::new();

        for (object_id, object) in scene.iter_objects() {
            let mesh = mesh(object);

            if mesh.spans.is_empty() {
                continue;
            }

            first_quads.insert(object_id, count(faces.len()));

            faces.extend(
                mesh.spans
                    .iter()
                    .map(|span| count(span.d) * 2 + u32::from(span.sign > 0)),
            );
            cells.extend(mesh.face_cells.iter().map(|covered| covered[0]));

            let size = BlasTriangleGeometrySizeDescriptor {
                vertex_format: VertexFormat::Float32x3,
                vertex_count: count(mesh.positions.len()),
                index_format: Some(IndexFormat::Uint32),
                index_count: Some(count(mesh.indices.len())),
                flags: AccelerationStructureGeometryFlags::OPAQUE,
            };

            let blas = device.create_blas(
                &CreateBlasDescriptor {
                    label: Some("quads"),
                    flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
                    update_mode: AccelerationStructureUpdateMode::Build,
                },
                BlasGeometrySizeDescriptors::Triangles {
                    descriptors: vec![size.clone()],
                },
            );

            let vertices = init(
                "quad vertices",
                &mesh
                    .positions
                    .iter()
                    .flat_map(|position| position.to_array())
                    .flat_map(f32::to_le_bytes)
                    .collect::<Vec<_>>(),
            );
            let indices = init(
                "quad indices",
                &mesh
                    .indices
                    .iter()
                    .flat_map(|index| index.to_le_bytes())
                    .collect::<Vec<_>>(),
            );

            inputs.push((object_id, size, vertices, indices));
            blases.insert(object_id, blas);
        }

        let entries: Vec<_> = inputs
            .iter()
            .map(|(object_id, size, vertices, indices)| BlasBuildEntry {
                blas: &blases[object_id],
                geometry: BlasGeometries::TriangleGeometries(vec![BlasTriangleGeometry {
                    size,
                    vertex_buffer: vertices,
                    first_vertex: 0,
                    vertex_stride: 12,
                    index_buffer: Some(indices),
                    first_index: Some(0),
                    transform_buffer: None,
                    transform_buffer_offset: None,
                }]),
            })
            .collect();

        let instanced: Vec<_> = (0..)
            .zip(scene.iter_placements())
            .filter(|(_, (_, placement))| blases.contains_key(&placement.object_id))
            .collect();

        let mut tlas = device.create_tlas(&CreateTlasDescriptor {
            label: Some("placements"),
            max_instances: count(instanced.len().max(1)),
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
                    rows[row * 4 + column] =
                        (columns[column].to_array()[row] * scale[column]) as f32;
                }

                rows[row * 4 + 3] = position[row] as f32;
            }

            tlas[slot] = Some(TlasInstance::new(
                &blases[&placement.object_id],
                rows,
                custom(*index, first_quads[&placement.object_id]),
                0xff,
            ));
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("quad structures"),
        });

        encoder.build_acceleration_structures(&entries, [&tlas]);
        queue.submit([encoder.finish()]);

        QuadTlas {
            tlas,
            first_quads,
            faces,
            cells,
        }
    }
}
