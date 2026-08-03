use serde::Serialize;
use std::io::{Error as IOError, ErrorKind, Result};
use ty_math::{TySrgbaF32, TyVector3F64};
use ty_math_serde::{TySrgbaF32Serde, TyVector3F64Serde};

#[derive(Serialize)]
struct PointsAndColors {
    points: Vec<TyVector3F64Serde>,
    colors: Vec<Vec<TySrgbaF32Serde>>,
}

/// Serializes points and per-texture color layers to JSON bytes.
pub fn serialize_points_and_colors_json(
    points: &[TyVector3F64],
    colors: &[Vec<TySrgbaF32>],
) -> Result<Vec<u8>> {
    let data = PointsAndColors {
        points: points
            .iter()
            .copied()
            .map(TyVector3F64Serde::from)
            .collect(),
        colors: colors
            .iter()
            .map(|layer| layer.iter().copied().map(TySrgbaF32Serde::from).collect())
            .collect(),
    };

    let mut bytes =
        serde_json::to_vec(&data).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    bytes.push(b'\n');

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::serialize_points_and_colors_json;
    use ty_math::{TySrgbaF32, TyVector3F64};

    #[test]
    fn serializes_to_stable_json() {
        // Pins the wire: the color keys stay `r`/`g`/`b`/`a` as `f32`, since
        // serde keys off the field names, not the struct name. Binary-fraction
        // values keep the float formatting exact.
        let points = [TyVector3F64::new(1.0, 2.0, 3.0)];
        let colors = [vec![TySrgbaF32::new(0.5, 0.25, 0.125, 1.0)]];

        let bytes = serialize_points_and_colors_json(&points, &colors).expect("serializes");
        let text = String::from_utf8(bytes).expect("utf8 json");

        assert_eq!(
            text,
            "{\"points\":[{\"x\":1.0,\"y\":2.0,\"z\":3.0}],\
             \"colors\":[[{\"r\":0.5,\"g\":0.25,\"b\":0.125,\"a\":1.0}]]}\n"
        );
    }
}
