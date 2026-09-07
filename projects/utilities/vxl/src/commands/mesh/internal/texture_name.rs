use crate::{Error, Result, commands::Texture, require_file_name};
use clap::ValueEnum;

/// A `--texture-name` value: a single-map preset paired with the exact file name
/// its map takes, `<preset> <file-name>`. The preset is a [`Texture`], never a
/// bundle, which names several files rather than one.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureName {
    preset: Texture,
    file_name: String,
}

impl TextureName {
    /// Pairs one `--texture-name <preset> <file-name>` occurrence, parsing the
    /// preset and validating the file name.
    pub fn new(preset: &str, file_name: &str) -> Result<Self> {
        let preset = Texture::from_str(preset, true).map_err(|_| {
            Error::usage(format!(
                "`{preset}` is not a single-map --texture preset; a bundle names several files"
            ))
        })?;

        let file_name = require_file_name(file_name).map_err(Error::usage)?;

        Ok(TextureName { preset, file_name })
    }

    /// The preset whose map this names.
    pub fn preset(&self) -> Texture {
        self.preset
    }

    /// The file name the preset's map takes, written beside the mesh.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{Texture, TextureName};

    #[test]
    fn pairs_a_preset_and_file_name() {
        let name = TextureName::new("albedo", "skin.png").unwrap();
        assert_eq!(name.preset(), Texture::Albedo);
        assert_eq!(name.file_name(), "skin.png");
    }

    #[test]
    fn rejects_a_bundle_preset() {
        assert!(TextureName::new("pbr", "skin.png").is_err());
    }

    #[test]
    fn rejects_an_unknown_preset_or_a_path_file_name() {
        assert!(TextureName::new("bogus", "skin.png").is_err());
        assert!(TextureName::new("albedo", "textures/skin.png").is_err());
        assert!(TextureName::new("albedo", "").is_err());
    }
}
