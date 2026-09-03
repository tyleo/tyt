use crate::{Format, Result, implementation};
use std::{
    fs,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxsmith::{MaterialMeshRequest, MeshFormat, object_to_mesh_files};

/// Loads the voxel file at `input`, meshes its object at index `object_index`
/// with [`object_to_mesh_files`], and writes the mesh to `output` with any
/// loose images beside it. The object index is a position into the document's
/// objects, as [`resolve_objects`] returns.
///
/// [`resolve_objects`]: crate::implementation::resolve_objects
pub fn mesh_object(
    input: &Path,
    from: Option<Format>,
    output: &Path,
    format: MeshFormat,
    object_index: usize,
    request: &MaterialMeshRequest,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let (_, object) = state.iter_objects().nth(object_index).ok_or_else(|| {
        IOError::new(
            ErrorKind::InvalidInput,
            format!("object index {object_index} is out of range"),
        )
    })?;

    let files = object_to_mesh_files(&state, object, format, request)?;

    fs::write(output, &files.mesh)?;

    // Loose images go beside the mesh, named as the document references them.
    let directory = output.parent().unwrap_or_else(|| Path::new("."));

    for (name, bytes) in &files.sidecars {
        fs::write(directory.join(name), bytes)?;
    }

    Ok(())
}
