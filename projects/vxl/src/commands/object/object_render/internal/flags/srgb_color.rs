use serde::Deserialize;
use std::str::FromStr;
use ty_math::{TyLinSrgbF64, TySrgbU8};

/// An sRGB color parsed from a `#RRGGBB` hex.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(try_from = "String")]
pub struct SrgbColor(pub(crate) TySrgbU8);

impl SrgbColor {
    /// The color in linear light.
    pub(crate) fn to_linear(self) -> TyLinSrgbF64 {
        self.0.into_format::<f64>().into_linear()
    }
}

impl FromStr for SrgbColor {
    type Err = String;

    /// Parses a `#RRGGBB` hex.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex = value
            .strip_prefix('#')
            .filter(|hex| hex.len() == 6)
            .ok_or_else(|| format!("`{value}` is not a color; use a #RRGGBB hex"))?;

        let byte = |index: usize| {
            hex.get(index * 2..index * 2 + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                .ok_or_else(|| format!("`{value}` is not a valid hex color"))
        };

        Ok(SrgbColor(TySrgbU8::new(byte(0)?, byte(1)?, byte(2)?)))
    }
}

impl TryFrom<String> for SrgbColor {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::SrgbColor;
    use ty_math::TySrgbU8;

    #[test]
    fn a_six_digit_hex_parses_and_the_rest_is_refused() {
        assert_eq!(
            "#0A0B0C".parse::<SrgbColor>(),
            Ok(SrgbColor(TySrgbU8::new(10, 11, 12)))
        );
        assert!("#0A0B0C0D".parse::<SrgbColor>().is_err());
        assert!("0A0B0C".parse::<SrgbColor>().is_err());
        assert!("#GG0B0C".parse::<SrgbColor>().is_err());
        assert!("white".parse::<SrgbColor>().is_err());

        assert_eq!(
            serde_json::from_str::<SrgbColor>(r##""#FF8000""##).unwrap(),
            SrgbColor(TySrgbU8::new(255, 128, 0))
        );
        assert!(serde_json::from_str::<SrgbColor>(r#""red""#).is_err());
    }

    #[test]
    fn linear_decodes_the_transfer() {
        let linear = SrgbColor(TySrgbU8::new(0, 128, 255)).to_linear();
        assert_eq!(linear.red, 0.0);
        assert!((linear.green - 0.2158605).abs() < 1e-6);
        assert_eq!(linear.blue, 1.0);
    }
}
