use crate::AttributeType;
use clap::ValueEnum;
use std::str::FromStr;

/// A named, typed reference to one palette layer's attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeBinding {
    name: String,
    palette: usize,
    key: String,
    ty: AttributeType,
}

impl AttributeBinding {
    /// Binds `name` to palette `palette`'s attribute `key` of type `ty`.
    pub fn new(name: String, palette: usize, key: String, ty: AttributeType) -> Self {
        AttributeBinding {
            name,
            palette,
            key,
            ty,
        }
    }

    /// The binding's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The palette layer, a position in the object's `paletteRefs`.
    pub fn palette(&self) -> usize {
        self.palette
    }

    /// The voxj attribute key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The value's type.
    pub fn ty(&self) -> AttributeType {
        self.ty
    }
}

impl FromStr for AttributeBinding {
    type Err = String;

    /// Parses the whitespace-separated `name palette key [type]` form; `type`
    /// defaults to `scalar`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let fields: Vec<&str> = value.split_whitespace().collect();
        let (name, palette, key, ty) = match fields.as_slice() {
            [name, palette, key] => (name, palette, key, None),
            [name, palette, key, ty] => (name, palette, key, Some(ty)),
            _ => return Err(format!("`{value}` is not `name palette key [type]`")),
        };
        let palette = palette
            .parse::<usize>()
            .map_err(|_| format!("`{palette}` is not a palette index"))?;
        let ty = match ty {
            Some(ty) => AttributeType::from_str(ty, true)
                .map_err(|_| format!("`{ty}` is not an attribute type; use scalar or color"))?,
            None => AttributeType::Scalar,
        };
        Ok(AttributeBinding::new(
            name.to_string(),
            palette,
            key.to_string(),
            ty,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttributeBinding, AttributeType};

    #[test]
    fn parses_with_explicit_type() {
        assert_eq!(
            "tint 1 tint color".parse::<AttributeBinding>().unwrap(),
            AttributeBinding::new(
                "tint".to_string(),
                1,
                "tint".to_string(),
                AttributeType::Color
            )
        );
    }

    #[test]
    fn type_defaults_to_scalar() {
        assert_eq!(
            "sss 0 subsurface".parse::<AttributeBinding>().unwrap(),
            AttributeBinding::new(
                "sss".to_string(),
                0,
                "subsurface".to_string(),
                AttributeType::Scalar
            )
        );
    }

    #[test]
    fn rejects_bad_bindings() {
        assert!("tint".parse::<AttributeBinding>().is_err());
        assert!("tint x tint".parse::<AttributeBinding>().is_err());
        assert!("tint 1 tint rgba".parse::<AttributeBinding>().is_err());
        assert!(
            "tint 1 tint color extra"
                .parse::<AttributeBinding>()
                .is_err()
        );
    }
}
