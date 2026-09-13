use crate::{Error, Result, commands::MeshOldTexture, require_file_name};
use clap::ValueEnum;

/// A `--texture-name` value: a single-map preset paired with the exact file name
/// its map takes, `<preset> <file-name>`. The preset is a [`MeshOldTexture`], never a
/// bundle, which names several files rather than one.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshOldTextureName {
    preset: MeshOldTexture,
    file_name: String,
}

impl MeshOldTextureName {
    /// Pairs one `--texture-name <preset> <file-name>` occurrence, parsing the
    /// preset and validating the file name.
    pub fn new(preset: &str, file_name: &str) -> Result<Self> {
        let preset = MeshOldTexture::from_str(preset, true).map_err(|_| {
            Error::usage(format!(
                "`{preset}` is not a single-map --texture preset; a bundle names several files"
            ))
        })?;

        let file_name = require_file_name(file_name).map_err(Error::usage)?;

        Ok(MeshOldTextureName { preset, file_name })
    }

    /// The preset whose map this names.
    pub fn preset(&self) -> MeshOldTexture {
        self.preset
    }

    /// The file name the preset's map takes, written beside the mesh.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{MeshOldTexture, MeshOldTextureName};

    #[test]
    fn pairs_a_preset_and_file_name() {
        let name = MeshOldTextureName::new("albedo", "skin.png").unwrap();
        assert_eq!(name.preset(), MeshOldTexture::Albedo);
        assert_eq!(name.file_name(), "skin.png");
    }

    #[test]
    fn rejects_a_bundle_preset() {
        assert!(MeshOldTextureName::new("pbr", "skin.png").is_err());
    }

    #[test]
    fn rejects_an_unknown_preset_or_a_path_file_name() {
        assert!(MeshOldTextureName::new("bogus", "skin.png").is_err());
        assert!(MeshOldTextureName::new("albedo", "textures/skin.png").is_err());
        assert!(MeshOldTextureName::new("albedo", "").is_err());
    }
}
