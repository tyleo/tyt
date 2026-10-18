use crate::operations::sdf_doc::{ArgumentCheck, SdfMaterialProperties};
use sdfcore::SdfShades;
use std::result::Result as StdResult;
use ty_math::TyLinSrgbaF64;

/// The lightness step between neighboring shades when the model leaves it out.
const DEFAULT_SPREAD: f64 = 0.05;

/// The shade at `index` of `shades`, whose base material holds `base`. A
/// darker shade mixes the base color with black in linear light, and a lighter
/// one keeps the color's Oklab hue and chroma.
pub fn shade_properties(
    base: &SdfMaterialProperties,
    shades: &SdfShades,
    index: u32,
) -> StdResult<SdfMaterialProperties, String> {
    let spread = shades.spread.unwrap_or(DEFAULT_SPREAD);
    let step = (f64::from(index) - (shades.count - 1.0) / 2.0) * spread;

    if step == 0.0 {
        return Ok(base.clone());
    }

    let color = base.base_color;
    let rgb = [color.red, color.green, color.blue];
    let lightness = oklab(rgb)[0];
    let target = lightness + step;

    if !(0.0..=1.0).contains(&target) {
        return Err(ArgumentCheck::new("shades").fail(
            &format!("shade {index}"),
            "between black and white",
            format!("a lightness of {target}"),
        ));
    }

    let [red, green, blue] = if step < 0.0 {
        toward_black(rgb, lightness, target)
    } else {
        lightened(rgb, target)
    };

    Ok(SdfMaterialProperties {
        base_color: TyLinSrgbaF64::new(red, green, blue, color.alpha),
        ..base.clone()
    })
}

/// The linear sRGB color `rgb` of Oklab lightness `lightness` scaled toward
/// black to the lightness `target`. Scaling a linear color by `k` scales its
/// Oklab coordinates by the cube root of `k`.
fn toward_black(rgb: [f64; 3], lightness: f64, target: f64) -> [f64; 3] {
    let scale = (target / lightness).powi(3);
    rgb.map(|component| component * scale)
}

/// The linear sRGB color at the Oklab lightness `target` with the hue of `rgb`
/// and as much of its chroma as linear sRGB holds there.
fn lightened(rgb: [f64; 3], target: f64) -> [f64; 3] {
    let [_, a, b] = oklab(rgb);
    let at = |share: f64| lin_srgb([target, a * share, b * share]);
    let fits = |color: [f64; 3]| {
        color
            .iter()
            .all(|component| (0.0..=1.0).contains(component))
    };

    if fits(at(1.0)) {
        return at(1.0);
    }

    let (mut low, mut high) = (0.0, 1.0);

    for _ in 0..64 {
        let middle = (low + high) / 2.0;

        if fits(at(middle)) {
            low = middle;
        } else {
            high = middle;
        }
    }

    // The gray at a share of 0 can stray past 1 by float error.
    at(low).map(|component| component.clamp(0.0, 1.0))
}

/// The linear sRGB color `rgb` in Oklab, by Bjorn Ottosson's matrices.
fn oklab([r, g, b]: [f64; 3]) -> [f64; 3] {
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();

    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}

/// The Oklab color `lab` in linear sRGB, by Bjorn Ottosson's matrices.
fn lin_srgb([lightness, a, b]: [f64; 3]) -> [f64; 3] {
    let l = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s = (lightness - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);

    [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
    ]
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{
        SdfMaterialProperties, material_properties,
        sdf_doc_voxelize::internal::shade_properties::{lin_srgb, oklab},
        shade_properties,
    };
    use branded_id::U32Id;
    use sdfcore::{SdfProperty, SdfPropertyValue, SdfShades};

    /// A `shades` call of `count` shades `spread` apart.
    fn shades(count: f64, spread: Option<f64>) -> SdfShades {
        SdfShades {
            base_id: U32Id::from_u32(0),
            count,
            spread,
        }
    }

    /// The material of the base color `color`.
    fn base(color: &str) -> SdfMaterialProperties {
        material_properties(&[SdfProperty {
            name: "baseColor".to_owned(),
            value: SdfPropertyValue::Text(color.to_owned()),
        }])
        .unwrap()
    }

    /// The Oklab lightness of the shade at `index` of `call` over `color`.
    fn lightness(color: &str, call: &SdfShades, index: u32) -> f64 {
        let shade = shade_properties(&base(color), call, index)
            .unwrap()
            .base_color;
        oklab([shade.red, shade.green, shade.blue])[0]
    }

    #[test]
    fn oklab_round_trips_and_reads_white_as_full_lightness() {
        let white = oklab([1.0, 1.0, 1.0]);
        assert!((white[0] - 1.0).abs() < 1e-6 && white[1].abs() < 1e-6);

        let color = [0.2, 0.5, 0.7];
        let back = lin_srgb(oklab(color));
        for (component, expected) in back.iter().zip(color) {
            assert!((component - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn the_shades_step_lightness_around_the_original_in_the_middle() {
        let call = shades(3.0, None);
        let middle = lightness("#8A5A2B", &call, 1);

        assert!((lightness("#8A5A2B", &call, 0) - (middle - 0.05)).abs() < 1e-6);
        assert!((lightness("#8A5A2B", &call, 2) - (middle + 0.05)).abs() < 1e-6);

        let even = shades(2.0, Some(0.1));
        assert!((lightness("#8A5A2B", &even, 1) - (middle + 0.05)).abs() < 1e-6);
    }

    #[test]
    fn the_middle_shade_keeps_the_base_and_every_other_property_carries_over() {
        let base = material_properties(&[
            SdfProperty {
                name: "baseColor".to_owned(),
                value: SdfPropertyValue::Text("#8A5A2B80".to_owned()),
            },
            SdfProperty {
                name: "roughness".to_owned(),
                value: SdfPropertyValue::Number(0.25),
            },
        ])
        .unwrap();

        assert_eq!(
            shade_properties(&base, &shades(3.0, None), 1).unwrap(),
            base
        );

        let darker = shade_properties(&base, &shades(3.0, Some(0.01)), 0).unwrap();
        assert_eq!(darker.base_color.alpha, base.base_color.alpha);
        assert_eq!(darker.roughness, 0.25);
    }

    #[test]
    fn a_saturated_base_keeps_every_shade_in_gamut() {
        for color in ["#E6C68A", "#8B0000", "#B05A1A"] {
            let call = shades(3.0, None);
            let middle = lightness(color, &call, 1);

            for index in 0..3 {
                let expected = middle + (f64::from(index) - 1.0) * 0.05;
                assert!((lightness(color, &call, index) - expected).abs() < 1e-6);

                let shade = shade_properties(&base(color), &call, index)
                    .unwrap()
                    .base_color;
                for component in [shade.red, shade.green, shade.blue] {
                    assert!((0.0..=1.0).contains(&component), "{color} shade {index}");
                }
            }
        }
    }

    #[test]
    fn a_lighter_shade_keeps_the_hue_and_the_chroma_srgb_holds() {
        let chroma_and_hue = |[_, a, b]: [f64; 3]| (a.hypot(b), b.atan2(a));
        let call = shades(3.0, None);

        for color in ["#C9A227", "#8A5A2B", "#2E6B30"] {
            let base_color = base(color).base_color;
            let (chroma, hue) =
                chroma_and_hue(oklab([base_color.red, base_color.green, base_color.blue]));

            let shade = shade_properties(&base(color), &call, 2).unwrap().base_color;
            let (shade_chroma, shade_hue) =
                chroma_and_hue(oklab([shade.red, shade.green, shade.blue]));

            assert!((shade_hue - hue).abs() < 1e-6, "{color}");
            assert!((shade_chroma - chroma).abs() < 1e-6, "{color}");
        }

        // A lighter yellow fits sRGB only with less chroma.
        let yellow = base("#FFFF00").base_color;
        let (chroma, hue) = chroma_and_hue(oklab([yellow.red, yellow.green, yellow.blue]));
        let shade = shade_properties(&base("#FFFF00"), &shades(3.0, Some(0.02)), 2)
            .unwrap()
            .base_color;
        let (shade_chroma, shade_hue) = chroma_and_hue(oklab([shade.red, shade.green, shade.blue]));

        assert!((shade_hue - hue).abs() < 1e-6);
        assert!(shade_chroma > 0.0 && shade_chroma < chroma);
    }

    #[test]
    fn a_shade_past_white_errors() {
        let white = base("#FFFFFF");

        let error = shade_properties(&white, &shades(3.0, None), 2).unwrap_err();
        assert!(
            error.starts_with(
                "shades shade 2 must be between black and white, not a lightness of 1.0"
            ),
            "{error}"
        );
        assert!(shade_properties(&white, &shades(3.0, None), 0).is_ok());
    }
}
