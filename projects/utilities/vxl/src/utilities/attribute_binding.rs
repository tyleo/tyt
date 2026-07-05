use crate::AttributeType;
use clap::ValueEnum;
use std::str::FromStr;

/// A named, typed binding giving a custom voxel attribute key a name a packing
/// can read. A binding reads the key from the meshed layer's palette, the layer
/// chosen by `mesh`'s `--layer` (the first by default).
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeBinding {
    name: String,
    key: String,
    ty: AttributeType,
}

impl AttributeBinding {
    /// Binds `name` to attribute `key` of type `ty`.
    pub fn new(name: String, key: String, ty: AttributeType) -> Self {
        AttributeBinding { name, key, ty }
    }

    /// The binding's name, as used in `--texture-map` and `--vertex-map`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The voxel attribute key read from the meshed layer's material.
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

    /// Parses the whitespace-separated `name key [type]` form; `type` defaults
    /// to `scalar`.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let fields: Vec<&str> = value.split_whitespace().collect();
        let (name, key, ty) = match fields.as_slice() {
            [name, key] => (name, key, None),
            [name, key, ty] => (name, key, Some(ty)),
            _ => return Err(format!("`{value}` is not `name key [type]`")),
        };
        let ty = match ty {
            Some(ty) => AttributeType::from_str(ty, true).map_err(|_| {
                format!(
                    "`{ty}` is not an attribute type; use a pool kind (srgb, srgba, \
                     linear-rgb, linear-rgba, float, int, bool) or the scalar/color alias"
                )
            })?,
            None => AttributeType::Float,
        };
        Ok(AttributeBinding::new(name.to_string(), key.to_string(), ty))
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttributeBinding, AttributeType};

    #[test]
    fn parses_with_explicit_type() {
        // `color` is the alias for `srgba`.
        assert_eq!(
            "tint tint color".parse::<AttributeBinding>().unwrap(),
            AttributeBinding::new("tint".to_string(), "tint".to_string(), AttributeType::Srgba)
        );
    }

    #[test]
    fn parses_pool_kinds_and_aliases() {
        let ty = |value: &str| value.parse::<AttributeBinding>().unwrap().ty();
        assert_eq!(ty("c c srgb"), AttributeType::Srgb);
        assert_eq!(ty("c c linear-rgba"), AttributeType::LinearRgba);
        assert_eq!(ty("s s float"), AttributeType::Float);
        assert_eq!(ty("s s int"), AttributeType::Int);
        assert_eq!(ty("m m bool"), AttributeType::Bool);
        // The aliases map to their canonical kinds.
        assert_eq!(ty("s s scalar"), AttributeType::Float);
        assert_eq!(ty("c c color"), AttributeType::Srgba);
    }

    #[test]
    fn type_defaults_to_scalar() {
        assert_eq!(
            "sss subsurface".parse::<AttributeBinding>().unwrap(),
            AttributeBinding::new(
                "sss".to_string(),
                "subsurface".to_string(),
                AttributeType::Float
            )
        );
    }

    #[test]
    fn rejects_bad_bindings() {
        assert!("tint".parse::<AttributeBinding>().is_err());
        assert!("tint tint rgba".parse::<AttributeBinding>().is_err());
        assert!("tint tint color extra".parse::<AttributeBinding>().is_err());
    }
}
