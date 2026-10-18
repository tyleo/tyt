use crate::{RenderMaterial, transmitted_reflectance};
use ty_math::TyLinSrgbF64;

const WHITE: TyLinSrgbF64 = TyLinSrgbF64::new(1.0, 1.0, 1.0);

/// The pass of `material` for a ray at `cosine` to the face normal: the share
/// of the light behind its surface that continues, per channel.
pub fn material_pass(material: &RenderMaterial, cosine: f64) -> TyLinSrgbF64 {
    let alpha = material.base_color.alpha;
    let transmitted = (WHITE - transmitted_reflectance(material, cosine))
        * material.base_color.color
        * (material.transmission * (1.0 - material.metallic));

    WHITE * (1.0 - alpha) + transmitted * alpha
}

#[cfg(test)]
mod tests {
    use crate::{
        RenderMaterial,
        fixtures::{glass, matte},
        material_pass,
    };
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};
    use voxcore::material;

    #[test]
    fn the_pass_at_normal_incidence_matches_voxcores_and_a_grazing_ray_passes_less() {
        let red = TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0);
        let half = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.5);

        let materials = [
            glass(red),
            glass(half),
            matte(red),
            RenderMaterial {
                metallic: 0.5,
                ..glass(red)
            },
        ];

        for material in &materials {
            assert_eq!(
                material_pass(material, 1.0),
                material::pass(
                    material.base_color,
                    material.metallic,
                    material.transmission,
                    material.ior
                )
            );
        }

        let grazing = material_pass(&glass(red), 0.25);
        assert!(grazing.red < material_pass(&glass(red), 1.0).red);
        assert_eq!((grazing.green, grazing.blue), (0.0, 0.0));

        // At grazing a half-alpha voxel's covered half passes one minus the
        // 0.8 cap, and the uncovered half passes whole.
        let edge = material_pass(&glass(half), 0.0);
        assert!((edge.red - 0.6).abs() < 1e-12, "{edge:?}");

        let rough = RenderMaterial {
            roughness: 1.0,
            ..glass(red)
        };
        assert_eq!(material_pass(&rough, 0.25), material_pass(&rough, 1.0));
        assert_eq!(
            material_pass(&matte(red), 0.25),
            TyLinSrgbF64::new(0.0, 0.0, 0.0)
        );
    }
}
