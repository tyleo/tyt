use voxsmith::operations::mesh::TextureShape;

/// Parses a `--texture-shape` value: the keyword `fit`, `line`, `pot`, or
/// `square`, else a positive side in cells.
pub(crate) fn parse_texture_shape(text: &str) -> Result<TextureShape, String> {
    match text {
        "fit" => Ok(TextureShape::Fit),
        "line" => Ok(TextureShape::Line),
        "pot" => Ok(TextureShape::Pot),
        "square" => Ok(TextureShape::Square),
        other => {
            let side = other.parse::<u32>().map_err(|_| {
                format!("`{other}` is not `fit`, `line`, `pot`, `square`, or a side in cells")
            })?;

            if side == 0 {
                return Err("a texture side must be at least 1".to_owned());
            }

            Ok(TextureShape::Exact(side))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::parse_texture_shape;
    use voxsmith::operations::mesh::TextureShape;

    #[test]
    fn parses_each_keyword_and_a_side() {
        for (text, expected) in [
            ("fit", TextureShape::Fit),
            ("line", TextureShape::Line),
            ("pot", TextureShape::Pot),
            ("square", TextureShape::Square),
            ("256", TextureShape::Exact(256)),
        ] {
            assert_eq!(parse_texture_shape(text), Ok(expected));
        }
    }

    #[test]
    fn rejects_zero_and_non_keywords() {
        assert!(parse_texture_shape("0").is_err());
        assert!(parse_texture_shape("huge").is_err());
        assert!(parse_texture_shape("-8").is_err());
        assert!(parse_texture_shape("").is_err());
    }
}
