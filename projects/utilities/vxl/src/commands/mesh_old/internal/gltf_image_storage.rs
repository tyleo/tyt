use crate::CliValue;
use meshconv::gltf::GltfImageStorage;

impl CliValue for GltfImageStorage {
    const VARIANTS: &'static [Self] = &[GltfImageStorage::Embedded, GltfImageStorage::Loose];

    fn name(self) -> &'static str {
        match self {
            GltfImageStorage::Embedded => "embedded",
            GltfImageStorage::Loose => "loose",
            GltfImageStorage::AsLoaded => "as-loaded",
        }
    }

    fn help(self) -> &'static str {
        match self {
            GltfImageStorage::Embedded => "In the mesh: a GLB binary chunk or a `.gltf` buffer",
            GltfImageStorage::Loose => "Loose files beside the mesh, which references them",
            GltfImageStorage::AsLoaded => "Where each image was loaded from",
        }
    }
}
