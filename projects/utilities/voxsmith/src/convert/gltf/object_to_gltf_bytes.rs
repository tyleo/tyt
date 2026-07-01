use crate::{Error, MeshMethod, Result, object_to_gltf_document};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;
use voxcore::VoxObject;

/// Meshes `object` with `method` and writes it as a text glTF (`.gltf`) with the
/// geometry buffer embedded as a base64 data URI, so the document is a single
/// self-contained file.
///
/// `scale` is the real-world edge length of one voxel in meters, applied as a
/// uniform scale to every vertex; glTF is meter-native, so the mesh opens at
/// that size. Geometry is Y-up, the inverse of the voxelizer's Z-up mapping. An
/// object with no live voxels writes a valid glTF with an empty scene.
pub fn object_to_gltf_bytes(object: &VoxObject, method: MeshMethod, scale: f64) -> Result<Vec<u8>> {
    let (mut document, blob) = object_to_gltf_document(object, method, scale);

    if !blob.is_empty() {
        let uri = format!(
            "data:application/octet-stream;base64,{}",
            STANDARD.encode(&blob)
        );

        document["buffers"][0]["uri"] = Value::String(uri);
    }

    serde_json::to_vec(&document).map_err(Error::invalid)
}

#[cfg(test)]
mod tests {
    use crate::{MeshMethod, object_to_gltf_bytes};
    use gltf::import_slice;
    use ty_math::TyVector3U32;
    use voxcore::VoxObject;

    #[test]
    fn gltf_text_embeds_the_buffer_and_parses() {
        let mut object = VoxObject::new("cube".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(id, &[]).unwrap();

        let bytes = object_to_gltf_bytes(&object, MeshMethod::Culled, 1.0).unwrap();

        // Text JSON with an embedded base64 buffer, not a binary GLB.
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.trim_start().starts_with('{'));
        assert!(text.contains("data:application/octet-stream;base64,"));

        let (document, buffers, _images) = import_slice(text.as_bytes()).unwrap();
        let primitive = document
            .meshes()
            .next()
            .unwrap()
            .primitives()
            .next()
            .unwrap();
        let reader = primitive.reader(|b| Some(&buffers[b.index()][..]));
        let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
        assert_eq!(positions.len(), 24); // six faces, four vertices each
    }
}
