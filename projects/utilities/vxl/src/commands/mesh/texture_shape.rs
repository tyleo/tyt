use std::str::FromStr;

/// One `--texture-shape` value: how the baked material atlas canvas is shaped,
/// either a keyword or an exact square side in pixels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextureShape {
    /// A single row of texels with no padding.
    Line,

    /// The near-square packing that exactly holds the material texels.
    Fit,

    /// The smallest square that holds the texels.
    Square,

    /// The smallest square power-of-two that holds the texels.
    Pot,

    /// An exact `side` by `side` canvas, rejected when too small to hold the
    /// texels.
    Exact(u32),
}

impl FromStr for TextureShape {
    type Err = String;

    /// Parses the keyword `line`, `fit`, `square`, or `pot`, else a positive
    /// integer square side.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "line" => Ok(Self::Line),
            "fit" => Ok(Self::Fit),
            "square" => Ok(Self::Square),
            "pot" => Ok(Self::Pot),
            other => {
                let side = other.parse::<u32>().map_err(|_| {
                    format!("`{other}` is not `line`, `fit`, `square`, `pot`, or a size")
                })?;

                if side == 0 {
                    return Err("a texture size must be at least 1".to_owned());
                }

                Ok(Self::Exact(side))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::TextureShape;

    #[test]
    fn parses_each_keyword() {
        assert_eq!("line".parse::<TextureShape>(), Ok(TextureShape::Line));
        assert_eq!("fit".parse::<TextureShape>(), Ok(TextureShape::Fit));
        assert_eq!("square".parse::<TextureShape>(), Ok(TextureShape::Square));
        assert_eq!("pot".parse::<TextureShape>(), Ok(TextureShape::Pot));
    }

    #[test]
    fn parses_a_pixel_size() {
        assert_eq!("256".parse::<TextureShape>(), Ok(TextureShape::Exact(256)));
    }

    #[test]
    fn rejects_zero_and_non_keywords() {
        assert!("0".parse::<TextureShape>().is_err());
        assert!("huge".parse::<TextureShape>().is_err());
        assert!("-8".parse::<TextureShape>().is_err());
        assert!("".parse::<TextureShape>().is_err());
    }
}
