use crate::{
    MaterialMeshRequest, MeshFiles, MeshFormat, Result, object_to_glb_bytes, object_to_gltf_bytes,
    object_to_material_glb, object_to_material_gltf,
};
use voxcore::{VoxMain, VoxObject};

/// Meshes `object` into a `format` file. With no maps in `request` it writes
/// pure geometry: no material and no images, so `request.storage` and
/// `request.shape` have nothing to place. Otherwise it bakes the object's
/// flattened layer materials into the requested maps, which the mesh samples,
/// with any loose images as sidecars.
pub fn object_to_mesh_files<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    format: MeshFormat,
    request: &MaterialMeshRequest,
) -> Result<MeshFiles> {
    if request.maps.is_empty() {
        let mesh = match format {
            MeshFormat::Gltf => object_to_gltf_bytes(object, request.method, request.scale)?,
            MeshFormat::Glb => object_to_glb_bytes(object, request.method, request.scale)?,
        };

        return Ok(MeshFiles {
            mesh,
            sidecars: Vec::new(),
        });
    }

    match format {
        MeshFormat::Gltf => object_to_material_gltf(state, object, request),
        MeshFormat::Glb => object_to_material_glb(state, object, request),
    }
}
