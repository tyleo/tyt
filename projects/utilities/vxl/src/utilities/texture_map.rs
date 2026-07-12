use crate::{
    AttributeBinding, ChannelPacking, Error, MeshTextureMap, Result, TextureBake, require_file_name,
};

/// A `--texture-map` value: a custom map's file name paired with its channel
/// packing. Pairing the flag's two flat tokens into this typed value at parse
/// keeps the pairing off a re-chunked `Vec<String>` later.
#[derive(Clone, Debug)]
pub struct TextureMap {
    name: String,
    channels: ChannelPacking,
}

impl TextureMap {
    /// Pairs one `--texture-map <file-name> <channels>` occurrence, validating
    /// the file name and parsing the channels.
    pub fn new(name: &str, channels: &str) -> Result<Self> {
        let name = require_file_name(name).map_err(Error::usage)?;

        let channels = channels.parse::<ChannelPacking>().map_err(Error::usage)?;

        Ok(TextureMap { name, channels })
    }

    /// Resolves this custom map against the `--define-attribute` bindings into
    /// the map the writer bakes. A custom packing has no standard glTF slot.
    pub fn resolve(&self, bindings: &[AttributeBinding]) -> Result<MeshTextureMap> {
        let packing = self.channels.resolve(bindings)?;

        Ok(MeshTextureMap {
            name: self.name.clone(),
            slot: None,
            bake: TextureBake::Packing(packing),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{TextureBake, TextureMap};

    #[test]
    fn pairs_a_name_and_channels() {
        let map = TextureMap::new("skin.png", "R=metallicFactor").unwrap();
        let resolved = map.resolve(&[]).unwrap();
        assert_eq!(resolved.name, "skin.png");
        assert_eq!(resolved.slot, None);
        assert!(matches!(resolved.bake, TextureBake::Packing(_)));
    }

    #[test]
    fn rejects_a_path_name_or_bad_channels() {
        assert!(TextureMap::new("textures/skin.png", "R=metallicFactor").is_err());
        assert!(TextureMap::new("skin.png", "X=metallicFactor").is_err());
    }
}
