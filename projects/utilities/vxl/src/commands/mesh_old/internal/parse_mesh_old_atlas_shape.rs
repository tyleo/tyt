use voxsmith::operations::mesh_old::AtlasShape;

/// Parses a `--texture-shape` value: the keyword `line`, `fit`, `square`, or
/// `pot`, else a positive integer square side in pixels.
pub fn parse_mesh_old_atlas_shape(text: &str) -> Result<AtlasShape, String> {
    match text {
        "line" => Ok(AtlasShape::Line),
        "fit" => Ok(AtlasShape::Fit),
        "square" => Ok(AtlasShape::Square),
        "pot" => Ok(AtlasShape::Pot),
        other => {
            let side = other.parse::<u32>().map_err(|_| {
                format!("`{other}` is not `line`, `fit`, `square`, `pot`, or a size")
            })?;

            if side == 0 {
                return Err("a texture size must be at least 1".to_owned());
            }

            Ok(AtlasShape::Exact(side))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::parse_mesh_old_atlas_shape;
    use voxsmith::operations::mesh_old::AtlasShape;

    #[test]
    fn parses_each_keyword() {
        assert_eq!(parse_mesh_old_atlas_shape("line"), Ok(AtlasShape::Line));
        assert_eq!(parse_mesh_old_atlas_shape("fit"), Ok(AtlasShape::Fit));
        assert_eq!(parse_mesh_old_atlas_shape("square"), Ok(AtlasShape::Square));
        assert_eq!(parse_mesh_old_atlas_shape("pot"), Ok(AtlasShape::Pot));
    }

    #[test]
    fn parses_a_pixel_size() {
        assert_eq!(
            parse_mesh_old_atlas_shape("256"),
            Ok(AtlasShape::Exact(256))
        );
    }

    #[test]
    fn rejects_zero_and_non_keywords() {
        assert!(parse_mesh_old_atlas_shape("0").is_err());
        assert!(parse_mesh_old_atlas_shape("huge").is_err());
        assert!(parse_mesh_old_atlas_shape("-8").is_err());
        assert!(parse_mesh_old_atlas_shape("").is_err());
    }
}
