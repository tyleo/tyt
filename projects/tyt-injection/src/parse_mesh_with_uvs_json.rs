use crate::MeshWithUvs;
use serde::Deserialize;
use std::io::{ErrorKind, Result};
use ty_math::TyVector3;
use ty_math_serde::TyVector3Serde;

#[derive(Deserialize)]
struct MeshDataWithUvs {
    vertices: Vec<TyVector3Serde>,
    triangles: Vec<[usize; 3]>,
    uvs: Vec<[[f64; 2]; 3]>,
}

/// Parses JSON mesh data (vertices + triangles + per-triangle UVs) from raw bytes.
pub fn parse_mesh_with_uvs_json(json: &[u8]) -> Result<MeshWithUvs> {
    let data: MeshDataWithUvs =
        serde_json::from_slice(json).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;

    let vertices = data.vertices.into_iter().map(TyVector3::from).collect();

    Ok((vertices, data.triangles, data.uvs))
}
