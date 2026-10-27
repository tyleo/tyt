use crate::{
    Error, Result,
    operations::object::{MeshElement, Transfer},
};

/// One entry's components under `transfer`, which curves each color
/// component and leaves an alpha as it is. A `unit` destination and the curve
/// take components in `[0, 1]`.
pub fn encode_components(
    element: &MeshElement,
    entry: &[f64],
    transfer: Transfer,
    unit: bool,
) -> Result<Vec<f64>> {
    let alpha = match entry.len() {
        2 | 4 => Some(entry.len() - 1),
        _ => None,
    };

    entry
        .iter()
        .enumerate()
        .map(|(index, &component)| {
            if !component.is_finite() {
                return Err(Error::mesh_record(
                    element.clone(),
                    format!("holds the non-finite component {component}"),
                ));
            }

            if (unit || transfer == Transfer::Srgb) && !(0.0..=1.0).contains(&component) {
                return Err(Error::mesh_record(
                    element.clone(),
                    format!("holds {component}, which lies outside [0, 1]"),
                ));
            }

            Ok(if transfer == Transfer::Srgb && Some(index) != alpha {
                srgb_encode(component)
            } else {
                component
            })
        })
        .collect()
}

/// `linear` encoded through the sRGB curve. `palette` evaluates the power
/// segment as `1.055 * p - 0.055` and lands `1.0` an ulp short. The equal
/// `p + 0.055 * (p - 1)` keeps `1.0` exact.
fn srgb_encode(linear: f64) -> f64 {
    if linear <= 0.0031308 {
        return 12.92 * linear;
    }

    let power = linear.powf(1.0 / 2.4);

    0.055f64.mul_add(power - 1.0, power)
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{
        MeshElement, Transfer, encode_components,
        object_mesh::write::encode_components::srgb_encode,
    };
    use ty_math::{TyLinSrgbF64, TySrgbF64};

    fn element() -> MeshElement {
        MeshElement::File {
            file: "bar.png".to_owned(),
        }
    }

    #[test]
    fn srgb_curves_the_color_components_and_leaves_the_alpha() {
        let encoded =
            encode_components(&element(), &[0.0, 1.0, 0.5, 0.5], Transfer::Srgb, false).unwrap();

        assert_eq!(encoded[0], 0.0);
        assert_eq!(encoded[1], 1.0);
        assert!((encoded[2] - 0.7354).abs() < 1e-4);
        assert_eq!(encoded[3], 0.5);

        let grey_alpha = encode_components(&element(), &[0.5, 0.5], Transfer::Srgb, false).unwrap();
        assert!((grey_alpha[0] - 0.7354).abs() < 1e-4);
        assert_eq!(grey_alpha[1], 0.5);
    }

    #[test]
    fn the_srgb_curve_matches_palette_and_keeps_the_endpoints_exact() {
        assert_eq!(srgb_encode(0.0), 0.0);
        assert_eq!(srgb_encode(1.0), 1.0);

        for step in 0..=1000 {
            let linear = f64::from(step) / 1000.0;
            let expected = TySrgbF64::from_linear(TyLinSrgbF64::new(linear, 0.0, 0.0)).red;

            assert!(
                (srgb_encode(linear) - expected).abs() <= f64::EPSILON,
                "{linear}"
            );
        }
    }

    #[test]
    fn linear_passes_any_finite_component_unless_the_destination_is_unit() {
        assert_eq!(
            encode_components(&element(), &[1.5, -2.0], Transfer::Linear, false).unwrap(),
            [1.5, -2.0]
        );
        assert!(encode_components(&element(), &[1.5], Transfer::Linear, true).is_err());
        assert!(encode_components(&element(), &[1.5], Transfer::Srgb, false).is_err());
        assert!(encode_components(&element(), &[f64::NAN], Transfer::Linear, false).is_err());
    }
}
