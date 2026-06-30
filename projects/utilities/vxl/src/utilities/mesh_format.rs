use clap::ValueEnum;
use std::path::Path;

/// A mesh file format that vxl can write. `mesh` infers it from the output
/// extension or takes it from `--to`, mirroring how [`crate::Format`] infers a
/// voxel format. Only `fbx` is supported for now; `obj` and `gltf` are planned.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum MeshFormat {
    /// Autodesk FBX, the `.fbx` file.
    #[value(name = "fbx")]
    Fbx,
}

impl MeshFormat {
    /// Infers the format from `path`'s extension, matched case-insensitively,
    /// or `None` when the extension names no supported mesh format.
    pub fn from_path(path: &Path) -> Option<MeshFormat> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "fbx" => Some(MeshFormat::Fbx),
            _ => None,
        }
    }
}
