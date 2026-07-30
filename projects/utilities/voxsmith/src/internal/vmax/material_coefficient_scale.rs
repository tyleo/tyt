//! Voxel Max stores metalness and roughness as coefficients on the range 0.1 to
//! 0.9, while its material sliders read those as 0 to 100 percent and every
//! other format carries the plain 0 to 1 glTF factor. The slider percent is
//! `(coefficient - 0.1) / 0.8`, so a coefficient of 0.1 reads as 0 percent and
//! 0.9 reads as 100 percent. The Voxel Max reader and writer convert at the
//! boundary so the shared `metallicFactor` and `roughnessFactor` value pools
//! stay on the 0 to 1 glTF range. An outgoing factor off the glTF range errors,
//! since no slider reads it. An incoming coefficient off the slider span is
//! something a file can carry, so it projects to the nearest end. The exact
//! coefficient rides in the ext block for the write-back, leaving only the
//! derived glTF view narrowed.

use crate::{Error, Result};

/// The lowest coefficient a slider produces, its 0 percent point.
const COEFFICIENT_MIN: f64 = 0.1;

/// The coefficient span a slider covers, from 0.1 at 0 percent to 0.9 at 100
/// percent.
const COEFFICIENT_SPAN: f64 = 0.8;

/// The highest coefficient a slider produces, its 100 percent point.
const COEFFICIENT_MAX: f64 = COEFFICIENT_MIN + COEFFICIENT_SPAN;

/// Maps a 0 to 1 glTF metalness or roughness factor to the Voxel Max
/// coefficient whose slider reads the same percent, the inverse of
/// [`vm_coefficient_to_pbr_factor`](crate::vm_coefficient_to_pbr_factor). A
/// `factor` outside 0 to 1 errors, named by `key`.
pub(crate) fn pbr_factor_to_vm_coefficient(factor: f64, key: &str) -> Result<f64> {
    if !(0.0..=1.0).contains(&factor) {
        return Err(Error::invalid(format!(
            "`{key}` is {factor}, outside the glTF range 0 to 1, so no Voxel Max slider \
             coefficient reads it"
        )));
    }

    Ok(COEFFICIENT_MIN + factor * COEFFICIENT_SPAN)
}

/// Maps a Voxel Max metalness or roughness coefficient onto the 0 to 1 glTF
/// factor its slider shows, the inverse of
/// [`pbr_factor_to_vm_coefficient`](crate::pbr_factor_to_vm_coefficient) over
/// the span a slider produces. A coefficient off that span has no factor on the
/// glTF range, so it projects to the nearest end. The ext block keeps the exact
/// coefficient the write-back needs.
pub(crate) fn vm_coefficient_to_pbr_factor(coefficient: f64) -> f64 {
    let clamped = coefficient.clamp(COEFFICIENT_MIN, COEFFICIENT_MAX);

    (clamped - COEFFICIENT_MIN) / COEFFICIENT_SPAN
}

#[cfg(test)]
mod tests {
    use crate::{METALLIC_FACTOR, pbr_factor_to_vm_coefficient, vm_coefficient_to_pbr_factor};

    /// Whether two coefficients agree within f64 rounding of the linear map.
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// The coefficient for `factor`, read as metalness.
    fn coefficient(factor: f64) -> f64 {
        pbr_factor_to_vm_coefficient(factor, METALLIC_FACTOR).unwrap()
    }

    fn factor(coefficient: f64) -> f64 {
        vm_coefficient_to_pbr_factor(coefficient)
    }

    #[test]
    fn maps_factor_endpoints_onto_the_coefficient_range() {
        assert_eq!(coefficient(0.0), 0.1);
        assert_eq!(coefficient(0.5), 0.5);
        assert!(close(coefficient(1.0), 0.9));
    }

    #[test]
    fn reads_coefficients_back_as_factors() {
        assert_eq!(factor(0.1), 0.0);
        assert_eq!(factor(0.5), 0.5);
        assert!(close(factor(0.9), 1.0));
    }

    #[test]
    fn round_trips_a_factor_through_a_coefficient() {
        for percent in 0..=100 {
            let start = f64::from(percent) / 100.0;
            let back = factor(coefficient(start));
            assert!(close(back, start), "{start} -> {back}");
        }
    }

    #[test]
    fn rejects_a_factor_outside_the_gltf_range() {
        assert!(pbr_factor_to_vm_coefficient(-1.0, METALLIC_FACTOR).is_err());
        assert!(pbr_factor_to_vm_coefficient(2.0, METALLIC_FACTOR).is_err());
    }

    #[test]
    fn projects_a_coefficient_off_the_slider_span_to_the_nearest_end() {
        // A file may carry either. The ext block keeps the exact coefficient.
        assert_eq!(factor(0.0), 0.0);
        assert_eq!(factor(1.0), 1.0);
        assert_eq!(factor(f64::INFINITY), 1.0);
    }
}
