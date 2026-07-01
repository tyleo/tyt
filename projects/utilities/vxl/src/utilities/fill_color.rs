use std::str::FromStr;

/// The `--fill-color` value: the color of voxels `voxelize` cannot sample from
/// the mesh. `None` is the `none` default, standing for white under `flat` and
/// the nearest surface for a sampled `solid` interior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillColor {
    /// `none`: no explicit color; the mode's own default fills in.
    #[default]
    None,

    /// A straight-RGBA color from a `#RRGGBB` or `#RRGGBBAA` hex.
    Rgba([u8; 4]),
}

impl FillColor {
    /// The explicit color, or `None` for `none`.
    pub fn rgba(self) -> Option<[u8; 4]> {
        match self {
            FillColor::None => Option::None,
            FillColor::Rgba(rgba) => Some(rgba),
        }
    }
}

impl FromStr for FillColor {
    type Err = String;

    /// Parses `none`, or a `#RRGGBB` or `#RRGGBBAA` hex color into straight RGBA
    /// bytes, defaulting a missing alpha to opaque.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("none") {
            return Ok(FillColor::None);
        }

        let hex = value
            .strip_prefix('#')
            .ok_or_else(|| format!("`{value}` is not a color; use none or a #RRGGBBAA hex"))?;

        parse_rgba_hex(hex)
            .map(FillColor::Rgba)
            .ok_or_else(|| format!("invalid color \"#{hex}\"; expected #RRGGBB or #RRGGBBAA"))
    }
}

/// Parses `RRGGBB` or `RRGGBBAA` hex digits (no leading `#`) into straight RGBA
/// bytes, a missing alpha defaulting to opaque. `None` if the string is not 6 or
/// 8 hex digits. Shared by `--fill-color` parsing and reading a stored `rgba`
/// value back.
pub fn parse_rgba_hex(hex: &str) -> Option<[u8; 4]> {
    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }

    let byte = |index: usize| {
        hex.get(index * 2..index * 2 + 2)
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
    };

    Some([
        byte(0)?,
        byte(1)?,
        byte(2)?,
        if hex.len() == 8 { byte(3)? } else { 255 },
    ])
}

#[cfg(test)]
mod tests {
    use crate::FillColor;

    #[test]
    fn parses_none_case_insensitively() {
        assert_eq!("none".parse::<FillColor>().unwrap(), FillColor::None);
        assert_eq!("None".parse::<FillColor>().unwrap(), FillColor::None);
    }

    #[test]
    fn parses_a_six_digit_hex_as_opaque() {
        assert_eq!(
            "#1A2B3C".parse::<FillColor>().unwrap(),
            FillColor::Rgba([0x1A, 0x2B, 0x3C, 0xFF])
        );
    }

    #[test]
    fn parses_an_eight_digit_hex_with_alpha() {
        assert_eq!(
            "#1A2B3C4D".parse::<FillColor>().unwrap(),
            FillColor::Rgba([0x1A, 0x2B, 0x3C, 0x4D])
        );
    }

    #[test]
    fn rejects_a_name_bad_length_or_non_hex() {
        // Color names are gone; only none or a hex parses.
        assert!("white".parse::<FillColor>().is_err());
        assert!("#12345".parse::<FillColor>().is_err());
        assert!("#GG0000".parse::<FillColor>().is_err());
    }

    #[test]
    fn rgba_maps_none_to_no_color() {
        assert_eq!(FillColor::None.rgba(), None);
        assert_eq!(FillColor::Rgba([1, 2, 3, 4]).rgba(), Some([1, 2, 3, 4]));
    }
}
