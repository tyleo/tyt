use crate::{Format, MeshFormat, MeshMethod, Result, implementation};
use std::{
    fs,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxsmith::{MeshMethod as VoxsmithMeshMethod, object_to_glb_bytes, object_to_gltf_bytes};

/// Meshes the object at index `object` of the voxel file at `input` into a mesh
/// at `output`, as pure geometry with no hierarchy-node transform. The index is
/// a position into the document's objects, as [`resolve_objects`] returns.
///
/// [`resolve_objects`]: crate::implementation::resolve_objects
#[allow(clippy::too_many_arguments)]
pub fn mesh_object(
    input: &Path,
    from: Option<Format>,
    output: &Path,
    format: MeshFormat,
    scale: f64,
    method: MeshMethod,
    object: usize,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let (_, object) = state.iter_objects().nth(object).ok_or_else(|| {
        IOError::new(
            ErrorKind::InvalidInput,
            format!("object index {object} is out of range"),
        )
    })?;

    let method = mesh_method(method);

    let bytes = match format {
        MeshFormat::Gltf => object_to_gltf_bytes(object, method, scale)?,
        MeshFormat::Glb => object_to_glb_bytes(object, method, scale)?,
    };

    fs::write(output, &bytes)?;

    Ok(())
}

/// Maps a CLI meshing method to the voxsmith method.
fn mesh_method(method: MeshMethod) -> VoxsmithMeshMethod {
    match method {
        MeshMethod::Greedy => VoxsmithMeshMethod::Greedy,
        MeshMethod::Culled => VoxsmithMeshMethod::Culled,
        MeshMethod::Naive => VoxsmithMeshMethod::Naive,
    }
}
