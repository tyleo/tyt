use serde::Deserialize;
use std::io::{ErrorKind, Result};
use ty_math::TyVector3;
use ty_math_serde::TyVector3Serde;

#[derive(Deserialize)]
struct MeshData {
    vertices: Vec<TyVector3Serde>,
    triangles: Vec<[usize; 3]>,
}

/// Parses JSON mesh data (vertices + triangles) from raw bytes.
pub fn parse_mesh_json(json: &[u8]) -> Result<(Vec<TyVector3>, Vec<[usize; 3]>)> {
    let data: MeshData =
        serde_json::from_slice(json).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;

    let vertices = data
        .vertices
        .into_iter()
        .map(TyVector3::from)
        .collect();

    Ok((vertices, data.triangles))
}
