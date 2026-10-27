use crate::TreeGridSwatch;

/// One value in a node's data series.
///
/// Its JSON form is always a JSON string of its text; behind the
/// `json` feature, `TreeGridJsonValue` pairs a value with a native
/// JSON form when text and JSON diverge.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeGridValue {
    /// The pre-rendered display text.
    pub text: String,

    /// The color block the default policy renders as this value's
    /// visual.
    pub swatch: Option<TreeGridSwatch>,
}

impl TreeGridValue {
    /// Creates a text-only value: no swatch.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            swatch: None,
        }
    }

    /// Creates an integer value: `Display` text, no swatch.
    pub fn int(value: i64) -> Self {
        Self::new(value.to_string())
    }

    /// Creates a float value: `Display` text, so an integral value
    /// prints without a fractional part (`1.0` -> `1`); no swatch.
    pub fn float(value: f64) -> Self {
        Self::new(value.to_string())
    }

    /// Creates a unit-interval value: `Display` text like
    /// [`float`](Self::float), with a grayscale swatch mapping the
    /// `0..1` range onto `0..255`, values outside it clamped.
    pub fn unorm(value: f64) -> Self {
        Self::float(value).with_swatch(TreeGridSwatch::Gray(unorm_level(value)))
    }

    /// Creates an 8-bit unsigned-normalized value: `Display` text,
    /// with a grayscale swatch of the byte itself.
    pub fn unorm8(value: u8) -> Self {
        Self::new(value.to_string()).with_swatch(TreeGridSwatch::Gray(value))
    }

    /// Creates a boolean value: `true` / `false` text, no swatch.
    pub fn bool(value: bool) -> Self {
        Self::new(value.to_string())
    }

    /// Creates an 8-bit sRGB color value: uppercase `#RRGGBB` hex
    /// text, with a color swatch.
    pub fn srgb8(rgb: [u8; 3]) -> Self {
        let [r, g, b] = rgb;
        Self::new(format!("#{r:02X}{g:02X}{b:02X}")).with_swatch(TreeGridSwatch::Color(rgb))
    }

    /// Creates an 8-bit sRGB color value with alpha: uppercase
    /// `#RRGGBBAA` hex text, with a color swatch of the rgb part.
    pub fn srgba8(rgba: [u8; 4]) -> Self {
        let [r, g, b, a] = rgba;
        Self::new(format!("#{r:02X}{g:02X}{b:02X}{a:02X}"))
            .with_swatch(TreeGridSwatch::Color([r, g, b]))
    }

    /// Sets the swatch.
    pub fn with_swatch(mut self, swatch: TreeGridSwatch) -> Self {
        self.swatch = Some(swatch);
        self
    }
}

/// The gray level for a unit-interval scalar: clamped to `[0, 1]` and
/// scaled to an 8-bit byte, rounding a half away from zero. The rule
/// is ty-math's `to_unorm8`, inlined because the core constructors
/// build without the `ty-math` feature.
fn unorm_level(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridSwatch, TreeGridValue};

    #[test]
    fn new_builds_a_text_only_value() {
        let value = TreeGridValue::new("{node: 0}");

        assert_eq!(value.text, "{node: 0}");
        assert_eq!(value.swatch, None);
    }

    #[test]
    fn int_renders_display_text_with_no_swatch() {
        let value = TreeGridValue::int(-3);

        assert_eq!(value.text, "-3");
        assert_eq!(value.swatch, None);
    }

    #[test]
    fn float_prints_an_integral_value_without_a_fractional_part() {
        assert_eq!(TreeGridValue::float(1.0).text, "1");
        assert_eq!(TreeGridValue::float(0.2).text, "0.2");
        assert_eq!(TreeGridValue::float(0.2).swatch, None);
    }

    #[test]
    fn unorm_maps_the_unit_interval_onto_a_gray_swatch() {
        let value = TreeGridValue::unorm(0.5);

        assert_eq!(value.text, "0.5");
        // 127.5 rounds a half away from zero.
        assert_eq!(value.swatch, Some(TreeGridSwatch::Gray(128)));
    }

    #[test]
    fn unorm_clamps_values_outside_the_unit_interval() {
        assert_eq!(
            TreeGridValue::unorm(2.0).swatch,
            Some(TreeGridSwatch::Gray(255))
        );
        assert_eq!(
            TreeGridValue::unorm(-1.0).swatch,
            Some(TreeGridSwatch::Gray(0))
        );
    }

    #[test]
    fn unorm8_swatches_the_byte_itself() {
        let value = TreeGridValue::unorm8(128);

        assert_eq!(value.text, "128");
        assert_eq!(value.swatch, Some(TreeGridSwatch::Gray(128)));
    }

    #[test]
    fn bool_renders_true_and_false_with_no_swatch() {
        assert_eq!(TreeGridValue::bool(true).text, "true");
        assert_eq!(TreeGridValue::bool(false).text, "false");
        assert_eq!(TreeGridValue::bool(true).swatch, None);
    }

    #[test]
    fn srgb8_renders_uppercase_hex_with_a_color_swatch() {
        let value = TreeGridValue::srgb8([255, 0, 128]);

        assert_eq!(value.text, "#FF0080");
        assert_eq!(value.swatch, Some(TreeGridSwatch::Color([255, 0, 128])));
    }

    #[test]
    fn srgba8_appends_the_alpha_byte_and_swatches_the_rgb_part() {
        let value = TreeGridValue::srgba8([255, 0, 128, 64]);

        assert_eq!(value.text, "#FF008040");
        assert_eq!(value.swatch, Some(TreeGridSwatch::Color([255, 0, 128])));
    }

    #[test]
    fn with_swatch_sets_the_swatch() {
        let value = TreeGridValue::new("1.00").with_swatch(TreeGridSwatch::Gray(255));

        assert_eq!(value.swatch, Some(TreeGridSwatch::Gray(255)));
    }
}
