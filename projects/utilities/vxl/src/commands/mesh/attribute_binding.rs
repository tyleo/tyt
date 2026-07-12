use std::str::FromStr;

/// A binding aliasing a custom voxel attribute key to a name a packing can read.
/// It renames a key from the meshed layer's palette, the layer `mesh`'s
/// `--layer` selects and the first by default; the value's type is read from
/// that key's value pool at bake, not declared here.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeBinding {
    name: String,
    key: String,
}

impl AttributeBinding {
    /// Binds `name` to attribute `key`.
    pub fn new(name: String, key: String) -> Self {
        AttributeBinding { name, key }
    }

    /// The binding's name, as used in `--texture-map` and the future
    /// `--vertex-map`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The voxel attribute key read from the meshed layer's material.
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl FromStr for AttributeBinding {
    type Err = String;

    /// Parses `name=key`, splitting on the first `=`. Both sides are required.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let malformed = || format!("`{value}` is not `name=key`");

        let (name, key) = value.split_once('=').ok_or_else(malformed)?;

        if name.is_empty() || key.is_empty() {
            return Err(malformed());
        }

        Ok(AttributeBinding::new(name.to_string(), key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::AttributeBinding;

    #[test]
    fn parses_a_name_and_key() {
        assert_eq!(
            "gloss=roughnessFactor".parse::<AttributeBinding>().unwrap(),
            AttributeBinding::new("gloss".to_string(), "roughnessFactor".to_string())
        );
    }

    #[test]
    fn keeps_the_key_verbatim() {
        // No `:type` split any more, so a key keeps every character, colon
        // included.
        assert_eq!(
            "tint=tint:extra".parse::<AttributeBinding>().unwrap(),
            AttributeBinding::new("tint".to_string(), "tint:extra".to_string())
        );
    }

    #[test]
    fn rejects_bad_bindings() {
        assert!("tint".parse::<AttributeBinding>().is_err());
        assert!("=tint".parse::<AttributeBinding>().is_err());
        assert!("tint=".parse::<AttributeBinding>().is_err());
    }
}
