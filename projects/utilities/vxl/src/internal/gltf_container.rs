use crate::CliValue;
use meshconv::gltf::GltfContainer;

impl CliValue for GltfContainer {
    const VARIANTS: &'static [Self] = &[GltfContainer::Glb, GltfContainer::Gltf];

    fn name(self) -> &'static str {
        match self {
            GltfContainer::Glb => "glb",
            GltfContainer::Gltf => "gltf",
        }
    }

    fn help(self) -> &'static str {
        match self {
            GltfContainer::Glb => "glTF binary, the `.glb` file",
            GltfContainer::Gltf => "glTF text, the `.gltf` file",
        }
    }
}
