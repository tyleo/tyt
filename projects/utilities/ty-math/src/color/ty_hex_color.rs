use crate::TySrgbaU8;

/// Hex-string glue for the byte sRGBA color. palette's own `FromStr` takes only
/// 4/8-digit RGBA and drops the `#`, so this preserves tyt's contract: an
/// `Option`, `#RRGGBB` (opaque default) or `#RRGGBBAA`, `#` optional on input
/// and always emitted uppercase.
pub trait TyHexColor: Sized {
    /// Parses a `#RRGGBB` or `#RRGGBBAA` hex string, with or without the leading
    /// `#`. A missing alpha defaults to opaque. Returns `None` when the value is
    /// not six or eight hexadecimal digits.
    fn from_hex(hex: &str) -> Option<Self>;

    /// Formats the color as an uppercase `#RRGGBBAA` hex string. Round-trips
    /// with [`from_hex`](Self::from_hex).
    fn to_hex(self) -> String;
}

impl TyHexColor for TySrgbaU8 {
    fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }

        let byte = |index: usize| u8::from_str_radix(hex.get(index * 2..index * 2 + 2)?, 16).ok();

        Some(TySrgbaU8::new(
            byte(0)?,
            byte(1)?,
            byte(2)?,
            if hex.len() == 8 { byte(3)? } else { 255 },
        ))
    }

    fn to_hex(self) -> String {
        let (r, g, b, a) = (self.red, self.green, self.blue, self.alpha);

        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyHexColor, TySrgbaU8};

    #[test]
    fn hex_round_trips_and_defaults_alpha() {
        // Eight digits carry alpha; six default it to opaque; the `#` is
        // optional either way.
        assert_eq!(
            TySrgbaU8::from_hex("#01020304"),
            Some(TySrgbaU8::new(1, 2, 3, 4))
        );
        assert_eq!(
            TySrgbaU8::from_hex("FF8000"),
            Some(TySrgbaU8::new(255, 128, 0, 255))
        );
        assert_eq!(TySrgbaU8::new(1, 2, 3, 4).to_hex(), "#01020304");
        assert_eq!(
            TySrgbaU8::from_hex(&TySrgbaU8::new(10, 200, 30, 40).to_hex()),
            Some(TySrgbaU8::new(10, 200, 30, 40))
        );
    }

    #[test]
    fn from_hex_rejects_malformed() {
        // Wrong length and non-hex digits are both rejected.
        assert_eq!(TySrgbaU8::from_hex("#12345"), None);
        assert_eq!(TySrgbaU8::from_hex("#GGGGGG"), None);
        assert_eq!(TySrgbaU8::from_hex(""), None);
    }
}
