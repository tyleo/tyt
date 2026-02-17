use std::io::{ErrorKind, Result};
use ty_math::TyVector3;
use ty_math_serde::TyVector3Serde;

/// Serializes a slice of points to JSON bytes.
pub fn serialize_points_json(points: &[TyVector3]) -> Result<Vec<u8>> {
    let verts: Vec<TyVector3Serde> = points.iter().copied().map(TyVector3Serde::from).collect();

    let mut bytes =
        serde_json::to_vec(&verts).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
    bytes.push(b'\n');

    Ok(bytes)
}
