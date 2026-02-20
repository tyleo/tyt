use serde::Serialize;
use std::io::{ErrorKind, Result};
use ty_math::{TyRgbaColor, TyVector3};
use ty_math_serde::{TyRgbaColorSerde, TyVector3Serde};

#[derive(Serialize)]
struct PointsAndColors {
    points: Vec<TyVector3Serde>,
    colors: Vec<Vec<TyRgbaColorSerde>>,
}

/// Serializes points and per-texture color layers to JSON bytes.
pub fn serialize_points_and_colors_json(
    points: &[TyVector3],
    colors: &[Vec<TyRgbaColor>],
) -> Result<Vec<u8>> {
    let data = PointsAndColors {
        points: points.iter().copied().map(TyVector3Serde::from).collect(),
        colors: colors
            .iter()
            .map(|layer| layer.iter().copied().map(TyRgbaColorSerde::from).collect())
            .collect(),
    };

    let mut bytes =
        serde_json::to_vec(&data).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
    bytes.push(b'\n');

    Ok(bytes)
}
