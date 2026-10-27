use ty_math::TyLinSrgbF64;

/// The reflectance at normal incidence of a surface of `base_color`,
/// `metallic`, and `ior`. The dielectric share is `((ior - 1) / (ior + 1))^2`,
/// and the metallic share is the base color.
pub fn normal_reflectance(base_color: TyLinSrgbF64, metallic: f64, ior: f64) -> TyLinSrgbF64 {
    let dielectric = ((ior - 1.0) / (ior + 1.0)).powi(2);
    let dielectric = TyLinSrgbF64::new(dielectric, dielectric, dielectric);

    dielectric * (1.0 - metallic) + base_color * metallic
}

#[cfg(test)]
mod tests {
    use crate::material;
    use ty_math::TyLinSrgbF64;

    #[test]
    fn the_index_of_refraction_sets_the_dielectric_reflectance() {
        let white = TyLinSrgbF64::new(1.0, 1.0, 1.0);

        let f0 = material::normal_reflectance(white, 0.0, 1.5);
        assert!((f0.red - 0.04).abs() < 1e-12);
        assert_eq!(f0.green, f0.red);
        assert_eq!(f0.blue, f0.red);

        assert_eq!(material::normal_reflectance(white, 0.0, 0.0), white);

        let gold = TyLinSrgbF64::new(1.0, 0.8, 0.4);
        assert_eq!(material::normal_reflectance(gold, 1.0, 1.5), gold);
    }
}
