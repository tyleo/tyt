use crate::{TreeGridJsonValue, TreeGridValue, number_json};
use serde_json::Value;
use std::fmt::Display;
use ty_math::{TyFloatExt, TyLinSrgb, TyLinSrgba, TySrgb, TySrgba};

impl TreeGridJsonValue {
    /// Creates a float-component sRGB color value:
    /// [`TreeGridValue::srgb`] paired with a number array.
    pub fn srgb<T: Copy + Display + TyFloatExt + Into<f64>>(color: TySrgb<T>) -> Self {
        Self {
            value: TreeGridValue::srgb(color),
            json: Some(components_json(color.to_array().map(Into::into))),
        }
    }

    /// Creates a float-component sRGB color value with alpha:
    /// [`TreeGridValue::srgba`] paired with a number array.
    pub fn srgba<T: Copy + Display + TyFloatExt + Into<f64>>(color: TySrgba<T>) -> Self {
        Self {
            value: TreeGridValue::srgba(color),
            json: Some(components_json(color.to_array().map(Into::into))),
        }
    }

    /// Creates a float-component linear RGB color value:
    /// [`TreeGridValue::lin_rgb`] paired with a number array.
    pub fn lin_rgb<T: Copy + Display + TyFloatExt + Into<f64>>(color: TyLinSrgb<T>) -> Self {
        Self {
            value: TreeGridValue::lin_rgb(color),
            json: Some(components_json(color.to_array().map(Into::into))),
        }
    }

    /// Creates a float-component linear RGB color value with alpha:
    /// [`TreeGridValue::lin_rgba`] paired with a number array.
    pub fn lin_rgba<T: Copy + Display + TyFloatExt + Into<f64>>(color: TyLinSrgba<T>) -> Self {
        Self {
            value: TreeGridValue::lin_rgba(color),
            json: Some(components_json(color.to_array().map(Into::into))),
        }
    }
}

/// A component array as JSON: each component a number collapsing like
/// [`TreeGridJsonValue::float`].
fn components_json<const N: usize>(components: [f64; N]) -> Value {
    Value::Array(components.iter().map(|c| number_json(*c)).collect())
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridJsonValue, TreeGridValue};
    use serde_json::json;
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TySrgbF32, TySrgbF64, TySrgbaF64};

    #[test]
    fn srgb_pairs_a_number_array() {
        let value = TreeGridJsonValue::srgb(TySrgbF64::new(1.0, 0.0, 0.5));

        assert_eq!(
            value.value,
            TreeGridValue::srgb(TySrgbF64::new(1.0, 0.0, 0.5))
        );
        assert_eq!(value.json, Some(json!([1, 0, 0.5])));
    }

    #[test]
    fn srgba_pairs_a_four_component_array() {
        let value = TreeGridJsonValue::srgba(TySrgbaF64::new(1.0, 0.0, 0.5, 0.25));

        assert_eq!(value.json, Some(json!([1, 0, 0.5, 0.25])));
    }

    #[test]
    fn lin_rgb_pairs_the_linear_components() {
        let value = TreeGridJsonValue::lin_rgb(TyLinSrgbF64::new(0.0, 1.0, 0.5));

        // The JSON carries the linear components, not the encoded
        // swatch bytes.
        assert_eq!(value.json, Some(json!([0, 1, 0.5])));
    }

    #[test]
    fn lin_rgba_keeps_an_hdr_component_at_full_fidelity() {
        let value = TreeGridJsonValue::lin_rgba(TyLinSrgbaF64::new(2.0, 1.0, 0.5, 1.0));

        assert_eq!(
            value.value,
            TreeGridValue::lin_rgba(TyLinSrgbaF64::new(2.0, 1.0, 0.5, 1.0))
        );
        assert_eq!(value.json, Some(json!([2, 1, 0.5, 1])));
    }

    #[test]
    fn f32_components_widen_losslessly() {
        let value = TreeGridJsonValue::srgb(TySrgbF32::new(0.25, 0.5, 1.0));

        assert_eq!(value.json, Some(json!([0.25, 0.5, 1])));
    }
}
