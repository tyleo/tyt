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
///
/// Blender may emit non-JSON lines to stdout before and after the JSON object.
/// This function extracts the substring from the first `{` to the last `}` before parsing.
pub fn parse_mesh_json(bytes: &[u8]) -> Result<(Vec<TyVector3>, Vec<[usize; 3]>)> {
    let start = bytes
        .iter()
        .position(|&b| b == b'{')
        .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "no '{' found in output"))?;
    let end = bytes
        .iter()
        .rposition(|&b| b == b'}')
        .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "no '}' found in output"))?;
    let json = &bytes[start..=end];

    let data: MeshData =
        serde_json::from_slice(json).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;

    let vertices = data.vertices.into_iter().map(TyVector3::from).collect();

    Ok((vertices, data.triangles))
}
