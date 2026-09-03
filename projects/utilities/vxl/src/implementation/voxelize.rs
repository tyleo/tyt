use crate::{Result, VoxjEncoding, VoxjFormat, implementation};
use std::{fs, path::Path};
use voxsmith::{EditStateMode, MeshFormat, VoxelizeOptions, from_gltf_bytes, from_mesh};

/// Voxelizes the glTF or GLB mesh at `input` under `options` into a Voxel Json
/// document at `output`. No `ext` block or edit state is recorded: a voxelized
/// mesh has neither a source `ext` to carry nor an editor build volume.
pub fn voxelize(
    input: &Path,
    _from: Option<MeshFormat>,
    output: &Path,
    options: &VoxelizeOptions,
    encoding: VoxjEncoding,
    format: VoxjFormat,
) -> Result<()> {
    let mesh = from_gltf_bytes(&fs::read(input)?)?;

    // The final fallback when neither `--name` nor the glTF names the object.
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("voxelized");

    let state = from_mesh(&mesh, stem, options)?;

    implementation::write_voxj_document(
        state,
        output,
        encoding,
        format,
        false,
        EditStateMode::Never,
    )
}
