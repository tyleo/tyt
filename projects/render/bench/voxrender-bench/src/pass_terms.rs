use ty_math::TyLinSrgbF64;
use voxcore::material;
use voxrender::RenderMaterial;

/// The terms of the contract's pass of a material. The pass is the share of
/// the light behind the surface that continues in each channel. For a ray at
/// a cosine `c` to the surface's normal the pass is
/// `uncovered + most * (1 - reflectance - rise * (1 - c)^5)`.
pub struct PassTerms {
    /// What the share of the surface a voxel covers passes of the light it
    /// does not reflect.
    pub most: TyLinSrgbF64,

    /// The share of the surface that no voxel covers.
    pub uncovered: f64,

    /// The reflectance at normal incidence.
    pub reflectance: TyLinSrgbF64,

    /// What the reflectance rises by toward grazing, which the roughness
    /// limits.
    pub rise: TyLinSrgbF64,
}

impl PassTerms {
    /// The terms of `material`'s pass.
    pub fn new(material: &RenderMaterial) -> Self {
        let base = material.base_color;
        let reflectance = material::normal_reflectance(base.color, material.metallic, material.ior);
        let smooth = 1.0 - material.roughness;
        let rise = |at_normal: f64| smooth.max(at_normal) - at_normal;

        PassTerms {
            most: base.color * (base.alpha * material.transmission * (1.0 - material.metallic)),
            uncovered: 1.0 - base.alpha,
            reflectance,
            rise: TyLinSrgbF64::new(
                rise(reflectance.red),
                rise(reflectance.green),
                rise(reflectance.blue),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::PassTerms;
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};
    use voxcore::material;
    use voxrender::{RenderMaterial, fixtures::glass};

    const WHITE: TyLinSrgbF64 = TyLinSrgbF64::new(1.0, 1.0, 1.0);

    /// The pass the terms give at `cosine`.
    fn pass(terms: &PassTerms, cosine: f64) -> TyLinSrgbF64 {
        let fade = (1.0 - cosine).powi(5);

        WHITE * terms.uncovered + terms.most * (WHITE - terms.reflectance - terms.rise * fade)
    }

    #[test]
    fn the_terms_give_voxcores_pass_at_normal_incidence() {
        let materials = [
            glass(TyLinSrgbaF64::new(1.0, 0.2, 0.1, 1.0)),
            glass(TyLinSrgbaF64::new(0.3, 0.6, 0.9, 0.5)),
            RenderMaterial {
                roughness: 0.6,
                metallic: 0.25,
                transmission: 0.8,
                ..glass(TyLinSrgbaF64::new(0.9, 0.9, 0.2, 0.75))
            },
        ];

        for material in &materials {
            let ours = pass(&PassTerms::new(material), 1.0);

            let theirs = material::pass(
                material.base_color,
                material.metallic,
                material.transmission,
                material.ior,
            );

            for (ours, theirs) in [
                (ours.red, theirs.red),
                (ours.green, theirs.green),
                (ours.blue, theirs.blue),
            ] {
                assert!((ours - theirs).abs() < 1e-12, "{ours} {theirs}");
            }
        }
    }

    #[test]
    fn a_grazing_ray_passes_what_the_roughness_keeps_from_the_reflection() {
        // The covered half of a half-alpha voxel reflects up to one minus
        // its roughness at grazing, and the other half passes whole.
        let half = glass(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.5));
        let edge = pass(&PassTerms::new(&half), 0.0);

        assert!(
            (edge.red - (0.5 + 0.5 * half.roughness)).abs() < 1e-12,
            "{edge:?}"
        );

        let rough = RenderMaterial {
            roughness: 1.0,
            ..half
        };
        let terms = PassTerms::new(&rough);

        assert_eq!(pass(&terms, 0.0), pass(&terms, 1.0));
    }
}
