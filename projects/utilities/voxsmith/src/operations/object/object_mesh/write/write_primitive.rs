use crate::{
    Result,
    operations::object::{ArrayDomain, Atlases, MeshGeometry, PrimitiveRecord, table_index},
};
use branded_id::U32Id;
use meshdoc::{MeshPrimitive, MeshTriangle};
use ty_math::{TyVector3F64, TyVector3I32};

/// The primitive drawing `faces` of `geometry` shifted by the grid `origin` and
/// scaled by `scene_scale`, with its streams read off `atlases` and no material
/// yet.
pub fn write_primitive(
    geometry: &MeshGeometry,
    faces: &[usize],
    origin: TyVector3I32,
    scene_scale: f64,
    primitive_record: &PrimitiveRecord,
    stream_list: &[ArrayDomain],
    atlases: &Atlases<'_>,
) -> Result<MeshPrimitive> {
    let vertices = |face: usize| face * 4..face * 4 + 4;

    let positions = faces
        .iter()
        .flat_map(|&face| &geometry.positions[vertices(face)])
        .map(|position| TyVector3F64::from((origin.as_dvec3() + position.as_dvec3()) * scene_scale))
        .collect();

    let triangles = faces
        .iter()
        .enumerate()
        .flat_map(|(quad, &face)| {
            geometry.indices[face * 6..face * 6 + 6]
                .as_chunks::<3>()
                .0
                .iter()
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
