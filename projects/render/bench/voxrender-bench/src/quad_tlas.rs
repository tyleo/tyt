use crate::{ObjectQuads, TilePlacement, to_word};
use ty_math::TyVector3F64;
use wgpu::{
    AccelerationStructureFlags, AccelerationStructureGeometryFlags,
    AccelerationStructureUpdateMode, BlasBuildEntry, BlasGeometries, BlasGeometrySizeDescriptors,
    BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor, BufferUsages,
    CommandEncoderDescriptor, CreateBlasDescriptor, CreateTlasDescriptor, Device, IndexFormat,
    Queue, Tlas, TlasInstance, VertexFormat,
    util::{BufferInitDescriptor, DeviceExt},
};

/// A scene's placements as a TLAS for a hardware ray query, whose build
/// goes to `queue`. Each object's BLAS holds its quads as two triangles
/// each. The ray tracing unit then finds a ray's first hit with no march.
/// A hit's primitive halves to its quad among its object's. An instance
/// holds its object's first quad among the scene's as its custom data. An
/// object with no quad has no instance. The device must have ray queries,
/// and the scene's quads number under 2^24.
///
/// `quads` holds each object's quads, and `placements` each placement of
/// one.
pub fn quad_tlas(
    device: &Device,
    queue: &Queue,
    quads: &[ObjectQuads],
    placements: &[TilePlacement],
) -> Tlas {
    let init = |label, contents: &[u8]| {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: BufferUsages::BLAS_INPUT,
        })
    };

    let mut total = 0;
    let mut first_quads = Vec::new();
    let mut blases = Vec::new();
    let mut inputs = Vec::new();

    for object in quads {
        if object.faces.is_empty() {
            first_quads.push(None);
            blases.push(None);

            continue;
        }

        first_quads.push(Some(to_word(total)));
        total += object.faces.len();

        let size = BlasTriangleGeometrySizeDescriptor {
            vertex_format: VertexFormat::Float32x3,
            vertex_count: to_word(object.positions.len()),
            index_format: Some(IndexFormat::Uint32),
            index_count: Some(to_word(object.indices.len())),
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

        let positions: Vec<_> = (object.positions.iter())
            .flat_map(|position| position.to_array())
            .collect();

        let vertices = init("quad vertices", bytemuck::cast_slice(&positions));
        let indices = init("quad indices", bytemuck::cast_slice(&object.indices));

        inputs.push((blases.len(), size, vertices, indices));
        blases.push(Some(blas));
    }

    assert!(total < 1 << 24, "a scene's quads number under 2^24");

    let entries: Vec<_> = inputs
        .iter()
        .map(|(object, size, vertices, indices)| BlasBuildEntry {
            blas: blases[*object].as_ref().expect("the object has quads"),
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

    let instanced: Vec<_> = placements
        .iter()
        .filter(|placed| blases[placed.object].is_some())
        .collect();

    let mut tlas = device.create_tlas(&CreateTlasDescriptor {
        label: Some("placements"),
        max_instances: to_word(instanced.len().max(1)),
        flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: AccelerationStructureUpdateMode::Build,
    });

    for (slot, TilePlacement { object, transform }) in instanced.into_iter().enumerate() {
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
            blases[*object].as_ref().expect("the object has quads"),
            rows,
            first_quads[*object].expect("the object has quads"),
            0xff,
        ));
    }

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("quad structures"),
    });

    encoder.build_acceleration_structures(&entries, [&tlas]);
    queue.submit([encoder.finish()]);

    tlas
}
