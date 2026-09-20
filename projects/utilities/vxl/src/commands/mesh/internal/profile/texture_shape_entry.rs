use crate::commands::parse_texture_shape;
use serde::Deserialize;
use voxsmith::operations::mesh::TextureShape;

/// A profile's `textureShape`, a keyword or a side in cells.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(try_from = "TextureShapeRepr")]
pub(crate) struct TextureShapeEntry(pub(crate) TextureShape);

#[derive(Deserialize)]
#[serde(untagged)]
enum TextureShapeRepr {
    Keyword(String),
    Side(u32),
}

impl TryFrom<TextureShapeRepr> for TextureShapeEntry {
    type Error = String;

    fn try_from(repr: TextureShapeRepr) -> Result<Self, String> {
        let shape = match repr {
            TextureShapeRepr::Keyword(keyword) => parse_texture_shape(&keyword)?,
            TextureShapeRepr::Side(side) => parse_texture_shape(&side.to_string())?,
        };

        Ok(TextureShapeEntry(shape))
    }
}

#[cfg(test)]
mod tests {
    use super::TextureShapeEntry;
    use voxsmith::operations::mesh::TextureShape;

    #[test]
    fn a_keyword_or_a_side_shapes_the_texture() {
        assert_eq!(
            serde_json::from_str::<TextureShapeEntry>("\"fit\"").unwrap(),
            TextureShapeEntry(TextureShape::Fit)
        );
        assert_eq!(
            serde_json::from_str::<TextureShapeEntry>("64").unwrap(),
            TextureShapeEntry(TextureShape::Exact(64))
        );
        assert!(serde_json::from_str::<TextureShapeEntry>("0").is_err());
        assert!(serde_json::from_str::<TextureShapeEntry>("\"round\"").is_err());
    }
}
