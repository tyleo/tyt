use crate::{Error, Result};

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
    /// Pairs one `--define-attribute <attribute> <name>` occurrence, binding the
    /// reference `name` to the voxel attribute `key`. Both are required, and the
    /// reference carries no whitespace.
    pub fn new(name: &str, key: &str) -> Result<Self> {
        if name.is_empty() || key.is_empty() {
            return Err(Error::usage(
                "--define-attribute takes two non-empty tokens, `<attribute> <name>`",
            ));
        }

        if name.chars().any(char::is_whitespace) {
            return Err(Error::usage(format!(
                "--define-attribute's `<attribute>` `{name}` has whitespace; it is the reference \
                 typed in --texture-map, which carries none",
            )));
        }

        Ok(AttributeBinding {
            name: name.to_string(),
            key: key.to_string(),
        })
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

#[cfg(test)]
mod tests {
    use crate::commands::AttributeBinding;

    #[test]
    fn pairs_an_attribute_and_name() {
        let binding = AttributeBinding::new("gloss", "roughnessFactor").unwrap();
        assert_eq!(binding.name(), "gloss");
        assert_eq!(binding.key(), "roughnessFactor");
    }

    #[test]
    fn keeps_the_name_verbatim_including_spaces() {
        // The `<name>` is its own quoted token, so a voxel name keeps every
        // character, spaces and colons included.
        assert_eq!(
            AttributeBinding::new("emissive", "super emissive thing")
                .unwrap()
                .key(),
            "super emissive thing"
        );
        assert_eq!(
            AttributeBinding::new("tint", "tint:extra").unwrap().key(),
            "tint:extra"
        );
    }

    #[test]
    fn rejects_an_empty_side() {
        assert!(AttributeBinding::new("", "tint").is_err());
        assert!(AttributeBinding::new("tint", "").is_err());
    }

    #[test]
    fn rejects_whitespace_in_the_attribute() {
        // The `<attribute>` is typed as a bare token in --texture-map, so it
        // cannot carry spaces even though its bound name may.
        assert!(AttributeBinding::new("super gloss", "roughnessFactor").is_err());
    }
}
