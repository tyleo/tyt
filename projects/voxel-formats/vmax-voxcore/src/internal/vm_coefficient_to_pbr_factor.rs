use crate::{COEFFICIENT_MAX, COEFFICIENT_MIN, Error, Result};

/// Maps a Voxel Max metalness or roughness coefficient onto the 0 to 1 glTF
/// factor its slider shows. The coefficient reads in the single precision
/// Voxel Max stores, so [`pbr_factor_to_vm_coefficient`](crate::pbr_factor_to_vm_coefficient())
/// gives it back exactly. A coefficient off the slider span errors with
/// `key`.
pub fn vm_coefficient_to_pbr_factor(coefficient: f64, key: &str) -> Result<f64> {
    let stored = coefficient as f32;
    if !(COEFFICIENT_MIN..=COEFFICIENT_MAX).contains(&stored) {
        return Err(Error::invalid(format!(
            "`{key}` is {coefficient}, outside the Voxel Max slider span 0.1 to 0.9"
        )));
    }

    let (min, max) = (f64::from(COEFFICIENT_MIN), f64::from(COEFFICIENT_MAX));
    Ok((f64::from(stored) - min) / (max - min))
}

#[cfg(test)]
mod tests {
    use crate::{COEFFICIENT_MAX, COEFFICIENT_MIN, vm_coefficient_to_pbr_factor};

    fn factor(coefficient: f64) -> f64 {
        vm_coefficient_to_pbr_factor(coefficient, "rc").unwrap()
    }

    #[test]
    fn reads_the_slider_ends_as_the_factor_ends() {
        assert_eq!(factor(f64::from(COEFFICIENT_MIN)), 0.0);

        assert_eq!(factor(f64::from(COEFFICIENT_MAX)), 1.0);
    }

    /// An older writer's double-precision `0.9` reads as the single Voxel Max
    /// stores for it. The slider span holds that single.
    #[test]
    fn reads_a_double_at_single_precision() {
        assert_eq!(factor(0.9), factor(f64::from(0.9f32)));
    }

    #[test]
    fn a_coefficient_off_the_slider_span_errors() {
        for coefficient in [0.0, 0.09, 0.95, 1.0, f64::INFINITY, f64::NAN] {
            let error = vm_coefficient_to_pbr_factor(coefficient, "rc").unwrap_err();

            assert!(error.to_string().contains("`rc`"), "{error}");
        }
    }
}
