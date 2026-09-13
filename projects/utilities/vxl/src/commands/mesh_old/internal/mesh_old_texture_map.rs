use crate::{
    Error, Result,
    commands::{MeshOldChannelPacking, MeshOldPropertyBinding},
    require_file_name,
};
use voxsmith::operations::mesh_old::{MaterialMap, MaterialSlot};

/// A `--texture-map` value: a custom map's file name paired with its channel
/// packing. Pairing the flag's two flat tokens into this typed value at parse
/// keeps the pairing off a re-chunked `Vec<String>` later.
#[derive(Clone, Debug)]
pub struct MeshOldTextureMap {
    name: String,
    channels: MeshOldChannelPacking,
}

impl MeshOldTextureMap {
    /// Pairs one `--texture-map <file-name> <channels>` occurrence, validating
    /// the file name and parsing the channels.
    pub fn new(name: &str, channels: &str) -> Result<Self> {
        let name = require_file_name(name).map_err(Error::usage)?;

        let channels = channels
            .parse::<MeshOldChannelPacking>()
            .map_err(Error::usage)?;

        Ok(MeshOldTextureMap { name, channels })
    }

    /// Resolves this custom map against the `--define-property` bindings into
    /// the map the writer bakes. A custom packing has no standard glTF slot.
    pub fn resolve(&self, bindings: &[MeshOldPropertyBinding]) -> Result<MaterialMap> {
        Ok(MaterialMap {
            name: self.name.clone(),
            slot: MaterialSlot::None,
            bake: self.channels.resolve(bindings)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::MeshOldTextureMap;
    use voxsmith::operations::mesh_old::{MaterialBake, MaterialSlot};

    #[test]
    fn pairs_a_name_and_channels() {
        let map = MeshOldTextureMap::new("skin.png", "R=metallic").unwrap();
        let resolved = map.resolve(&[]).unwrap();
        assert_eq!(resolved.name, "skin.png");
        assert_eq!(resolved.slot, MaterialSlot::None);
        assert!(matches!(resolved.bake, MaterialBake::Packing(_)));
    }

    #[test]
    fn rejects_a_path_name_or_bad_channels() {
        assert!(MeshOldTextureMap::new("textures/skin.png", "R=metallic").is_err());
        assert!(MeshOldTextureMap::new("skin.png", "X=metallic").is_err());
    }
}
