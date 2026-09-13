use vmax::{VMaxMaterial, VMaxMaterialDispersion};

/// Each material's dispersion field `read`, or zero where dispersion is absent.
pub(crate) fn dispersion(
    materials: &[VMaxMaterial],
    read: impl Fn(&VMaxMaterialDispersion) -> f64,
) -> Vec<f64> {
    materials
        .iter()
        .map(|m| m.md.as_ref().map_or(0.0, &read))
        .collect()
}
