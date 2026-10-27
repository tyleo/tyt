use crate::material;
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};

const WHITE: TyLinSrgbF64 = TyLinSrgbF64::new(1.0, 1.0, 1.0);

/// The share of the light behind a surface of `base_color`, `metallic`,
/// `transmission`, and `ior` that continues through it, per channel: the
/// part the base color's alpha leaves uncovered, plus what the covered part
/// transmits past the reflectance at normal incidence, tinted by the base
/// color. Zero for an opaque surface.
pub fn pass(base_color: TyLinSrgbaF64, metallic: f64, transmission: f64, ior: f64) -> TyLinSrgbF64 {
    let alpha = base_color.alpha;
    let transmitted = (WHITE - material::normal_reflectance(base_color.color, metallic, ior))
        * base_color.color
        * (transmission * (1.0 - metallic));

    WHITE * (1.0 - alpha) + transmitted * alpha
}

#[cfg(test)]
mod tests {
    use crate::material::{self, IOR_DEFAULT, pass::WHITE};
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};

    const BLACK: TyLinSrgbF64 = TyLinSrgbF64::new(0.0, 0.0, 0.0);

    fn close(a: TyLinSrgbF64, b: TyLinSrgbF64) -> bool {
        (a.red - b.red).abs() < 1e-9
            && (a.green - b.green).abs() < 1e-9
            && (a.blue - b.blue).abs() < 1e-9
    }

    #[test]
    fn the_pass_is_the_uncovered_part_plus_what_the_covered_part_transmits() {
        let white = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0);
        let red = TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0);
        let half = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.5);
        let clear = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.0);

        // The vocabulary's defaults are opaque.
        assert_eq!(material::pass(white, 1.0, 0.0, IOR_DEFAULT), BLACK);
        assert_eq!(material::pass(clear, 0.0, 0.0, IOR_DEFAULT), WHITE);
        assert!(close(
            material::pass(white, 0.0, 1.0, IOR_DEFAULT),
            TyLinSrgbF64::new(0.96, 0.96, 0.96)
        ));
        assert!(close(
            material::pass(red, 0.0, 1.0, IOR_DEFAULT),
            TyLinSrgbF64::new(0.96, 0.0, 0.0)
        ));
        assert!(close(
            material::pass(half, 0.0, 1.0, IOR_DEFAULT),
            TyLinSrgbF64::new(0.98, 0.98, 0.98)
        ));

        // A metal and a mirror reflect everything they cover.
        assert_eq!(material::pass(white, 1.0, 1.0, IOR_DEFAULT), BLACK);
        assert_eq!(material::pass(white, 0.0, 1.0, 0.0), BLACK);
    }
}
