//! Voxel Max stores metalness and roughness as coefficients on the range 0.1
//! to 0.9, while its material sliders read those as 0 to 100 percent and every
//! other format carries the plain 0 to 1 glTF factor. The slider percent is
//! `(coefficient - 0.1) / 0.8`, so a coefficient of 0.1 reads as 0 percent and
//! 0.9 reads as 100 percent. The Voxel Max reader and writer convert at the
//! boundary so the shared `metallicFactor` and `roughnessFactor` value pools
//! stay on the 0 to 1 glTF range. Writing a bare 0.0 metalness left the
//! selected material's slider reading a negative percent.

/// The lowest coefficient a slider produces, its 0 percent point.
const COEFFICIENT_MIN: f64 = 0.1;

/// The coefficient span a slider covers, from 0.1 at 0 percent to 0.9 at 100
/// percent.
const COEFFICIENT_SPAN: f64 = 0.8;

/// Maps a 0 to 1 glTF metalness or roughness factor to the Voxel Max
/// coefficient whose slider reads the same percent, the inverse of
/// [`vm_coefficient_to_pbr_factor`](crate::vm_coefficient_to_pbr_factor). The
/// factor is clamped to 0 to 1 so the coefficient stays within 0.1 to 0.9 and
/// the slider never reads past its ends.
pub(crate) fn pbr_factor_to_vm_coefficient(factor: f64) -> f64 {
    COEFFICIENT_MIN + factor.clamp(0.0, 1.0) * COEFFICIENT_SPAN
}

/// Maps a Voxel Max metalness or roughness coefficient back to the 0 to 1 glTF
/// factor its slider shows, the inverse of
/// [`pbr_factor_to_vm_coefficient`](crate::pbr_factor_to_vm_coefficient). The
/// result is clamped to 0 to 1, healing a coefficient an earlier build wrote
/// below 0.1.
pub(crate) fn vm_coefficient_to_pbr_factor(coefficient: f64) -> f64 {
    ((coefficient - COEFFICIENT_MIN) / COEFFICIENT_SPAN).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use crate::{pbr_factor_to_vm_coefficient, vm_coefficient_to_pbr_factor};

    /// Whether two coefficients agree within f64 rounding of the linear map.
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn maps_factor_endpoints_onto_the_coefficient_range() {
        assert_eq!(pbr_factor_to_vm_coefficient(0.0), 0.1);
        assert_eq!(pbr_factor_to_vm_coefficient(0.5), 0.5);
        assert!(close(pbr_factor_to_vm_coefficient(1.0), 0.9));
    }

    #[test]
    fn reads_coefficients_back_as_factors() {
        assert_eq!(vm_coefficient_to_pbr_factor(0.1), 0.0);
        assert_eq!(vm_coefficient_to_pbr_factor(0.5), 0.5);
        assert!(close(vm_coefficient_to_pbr_factor(0.9), 1.0));
    }

    #[test]
    fn round_trips_a_factor_through_a_coefficient() {
        for percent in 0..=100 {
            let factor = f64::from(percent) / 100.0;
            let back = vm_coefficient_to_pbr_factor(pbr_factor_to_vm_coefficient(factor));
            assert!(close(back, factor), "{factor} -> {back}");
        }
    }

    #[test]
    fn clamps_out_of_range_values() {
        assert_eq!(pbr_factor_to_vm_coefficient(-1.0), 0.1);
        assert_eq!(pbr_factor_to_vm_coefficient(2.0), 0.9);
        // A coefficient below 0.1, as an earlier build wrote for 0.0 metalness,
        // heals to 0 rather than a negative factor.
        assert_eq!(vm_coefficient_to_pbr_factor(0.0), 0.0);
    }
}
