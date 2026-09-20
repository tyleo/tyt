use crate::{
    Result,
    operations::mesh::{ArrayDomain, Atlases, MeshGeometry, PrimitiveRecord, table_index},
};
use branded_id::U32Id;
use meshdoc::{MeshPrimitive, MeshTriangle};
use ty_math::TyVector3F64;

/// The primitive drawing `faces` of `geometry` at `voxel_size` meters per
/// voxel, with its streams read off `atlases`.
pub(crate) fn write_primitive(
    geometry: &MeshGeometry,
    faces: &[usize],
    voxel_size: f64,
    primitive_record: &PrimitiveRecord,
    stream_list: &[ArrayDomain],
    atlases: &Atlases<'_>,
) -> Result<MeshPrimitive> {
    let vertices = |face: usize| face * 4..face * 4 + 4;

    let positions = faces
        .iter()
        .flat_map(|&face| &geometry.positions[vertices(face)])
        .map(|position| TyVector3F64::from(position.as_dvec3() * voxel_size))
        .collect();

    let triangles = faces
        .iter()
        .enumerate()
        .flat_map(|(quad, &face)| {
            geometry.indices[face * 6..face * 6 + 6]
                .chunks_exact(3)
                .map(move |corners| MeshTriangle {
                    vertex_ids: [0, 1, 2].map(|corner| {
                        let index = corners[corner] as usize - face * 4 + quad * 4;
                        U32Id::from_u32(table_index(index))
                    }),
                })
        })
        .collect();

    let mut primitive = MeshPrimitive::new(positions, triangles)?;

    if primitive_record.normal {
        primitive.set_normals(Some(
            faces
                .iter()
                .flat_map(|&face| &geometry.normals[vertices(face)])
                .map(|normal| normal.as_dvec3())
                .collect(),
        ))?;
    }

    if let Some(name) = &primitive_record.name {
        primitive.set_name(name.clone());
    }

    for &domain in stream_list {
        primitive.push_uv_stream(atlases.uvs(domain, faces)?)?;
    }

    Ok(primitive)
}
