use vmax::VMaxMaterialDispersion;
use voxcore::material::{IOR, default_scalar};

/// The dispersion values the pools hold for a slot Voxel Max stores with no
/// `md`.
pub fn absent_dispersion() -> VMaxMaterialDispersion {
    VMaxMaterialDispersion {
        absorption: 0.0,
        ior: default_scalar(IOR).expect("ior has a glTF default"),
        transmission: 0.0,
    }
}
