use crate::CliValue;
use voxsmith::ResourceStorage;

impl CliValue for ResourceStorage {
    const VARIANTS: &'static [Self] = &[
        ResourceStorage::Embedded,
        ResourceStorage::External,
        ResourceStorage::Both,
    ];

    fn name(self) -> &'static str {
        match self {
            ResourceStorage::Embedded => "embedded",
            ResourceStorage::External => "external",
            ResourceStorage::Both => "both",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ResourceStorage::Embedded => {
                "Packed into the mesh: a GLB binary chunk or a `.gltf` data URI"
            }
            ResourceStorage::External => {
                "Written as loose files beside the mesh, which references them"
            }
            ResourceStorage::Both => {
                "Both: the mesh references its embedded copy and the loose files are working copies"
            }
        }
    }
}
