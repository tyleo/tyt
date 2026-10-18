use crate::normal_reflectance;
use ty_math::TyLinSrgbF64;
use voxrender::RenderMaterial;

/// The share of the light that `material`'s transmitted share reflects for a
/// ray at `cosine` to the face normal: Schlick's Fresnel.
pub fn transmitted_reflectance(material: &RenderMaterial, cosine: f64) -> TyLinSrgbF64 {
    let f0 = normal_reflectance(material);
    let smooth = 1.0 - material.roughness;
    let cap = TyLinSrgbF64::new(
        smooth.max(f0.red),
        smooth.max(f0.green),
        smooth.max(f0.blue),
    );

    f0 + (cap - f0) * (1.0 - cosine).powi(5)
}

#[cfg(test)]
mod tests {
    use crate::{normal_reflectance, transmitted_reflectance};
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};
    use voxrender::{RenderMaterial, fixtures::glass};

    #[test]
    fn the_reflectance_rises_from_f0_toward_the_roughness_cap() {
        let clear = glass(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0));
        let f0 = normal_reflectance(&clear);

        assert_eq!(transmitted_reflectance(&clear, 1.0), f0);
        assert_eq!(
            transmitted_reflectance(&clear, 0.0),
            TyLinSrgbF64::new(0.8, 0.8, 0.8)
        );

        let oblique = transmitted_reflectance(&clear, 0.5);
        assert!(oblique.red > f0.red && oblique.red < 0.8, "{oblique:?}");

        let rough = RenderMaterial {
            roughness: 1.0,
            ..clear
        };
        assert_eq!(transmitted_reflectance(&rough, 0.0), f0);
    }
}
