use std::str::FromStr;

/// Straight RGBA bytes parsed from a `#RRGGBB` or `#RRGGBBAA` hex, a missing
/// alpha defaulting to opaque.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba(pub [u8; 4]);

impl FromStr for Rgba {
    type Err = String;

    /// Parses a `#RRGGBB` or `#RRGGBBAA` hex into straight RGBA bytes, a missing
    /// alpha defaulting to opaque.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let hex = value
            .strip_prefix('#')
            .filter(|hex| hex.len() == 6 || hex.len() == 8)
            .ok_or_else(|| format!("`{value}` is not a color; use a #RRGGBB or #RRGGBBAA hex"))?;

        let byte = |index: usize| {
            hex.get(index * 2..index * 2 + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                .ok_or_else(|| format!("`{value}` is not a valid hex color"))
        };

        Ok(Rgba([
            byte(0)?,
            byte(1)?,
            byte(2)?,
            if hex.len() == 8 { byte(3)? } else { 255 },
        ]))
    }
}

#[cfg(test)]
mod tests {
    use crate::Rgba;

    #[test]
    fn a_hex_color_parses_to_rgba() {
        // Six digits default the alpha to opaque; eight carry it through.
        assert_eq!("#0A0B0C".parse::<Rgba>(), Ok(Rgba([10, 11, 12, 255])));
        assert_eq!("#0A0B0C0D".parse::<Rgba>(), Ok(Rgba([10, 11, 12, 13])));
    }

    #[test]
    fn an_invalid_hex_color_is_rejected() {
        // A color name, a missing `#`, a bad length, and non-hex digits all fail.
        assert!("white".parse::<Rgba>().is_err());
        assert!("ff0000".parse::<Rgba>().is_err());
        assert!("#12345".parse::<Rgba>().is_err());
        assert!("#GG0000".parse::<Rgba>().is_err());
    }
}
