use crate::{COEFFICIENT_MAX, COEFFICIENT_MIN, Error, Result};

/// Maps a 0 to 1 glTF metalness or roughness factor to the Voxel Max
/// coefficient whose slider reads the same percent, in the single precision
/// Voxel Max stores. A `factor` outside 0 to 1 errors with `key`.
pub fn pbr_factor_to_vm_coefficient(factor: f64, key: &str) -> Result<f64> {
    if !(0.0..=1.0).contains(&factor) {
        return Err(Error::invalid(format!(
            "`{key}` is {factor}, outside the glTF range 0 to 1, so no Voxel Max slider \
             coefficient reads it"
        )));
    }

    let (min, max) = (f64::from(COEFFICIENT_MIN), f64::from(COEFFICIENT_MAX));
    Ok(f64::from((min + factor * (max - min)) as f32))
}

#[cfg(test)]
mod tests {
    use crate::{
        COEFFICIENT_MAX, COEFFICIENT_MIN, pbr_factor_to_vm_coefficient,
        vm_coefficient_to_pbr_factor,
    };
    use voxcore::material::METALLIC;

    fn coefficient(factor: f64) -> f64 {
        pbr_factor_to_vm_coefficient(factor, METALLIC).unwrap()
    }

    #[test]
    fn maps_the_factor_ends_onto_the_slider_ends() {
        assert_eq!(coefficient(0.0), f64::from(COEFFICIENT_MIN));

        assert_eq!(coefficient(1.0), f64::from(COEFFICIENT_MAX));
    }

    #[test]
    fn rejects_a_factor_outside_the_gltf_range() {
        assert!(pbr_factor_to_vm_coefficient(-1.0, METALLIC).is_err());

        assert!(pbr_factor_to_vm_coefficient(2.0, METALLIC).is_err());
    }

    /// Every single on the slider span reads to a factor and back exactly,
    /// sampled at a prime stride.
    #[test]
    fn round_trips_a_coefficient_through_a_factor() {
        let ends = [COEFFICIENT_MIN.to_bits(), COEFFICIENT_MAX.to_bits()];
        let stored = (ends[0]..=ends[1])
            .step_by(997)
            .chain(ends)
            .map(f32::from_bits);
        for stored in stored {
            let start = f64::from(stored);

            let back = coefficient(vm_coefficient_to_pbr_factor(start, METALLIC).unwrap());

            assert_eq!(back, start);
        }
    }
}
