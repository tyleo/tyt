use crate::{DecodeBase64, GltfFile, GltfMeshMain, Result, from_gltf_file, is_glb, parse_glb};
use gltf::json::Root;
use std::collections::BTreeMap;

/// Loads the bytes of a `.gltf` or `.glb` file through `dependencies` into a
/// [`GltfMeshMain`], the bytes form of [`from_gltf_file`]. The container is
/// detected from the leading bytes. `loose_files` holds the files beside the
/// primary, keyed by the relative URI the primary references them by, which
/// [`gltf_loose_uris`](crate::codec::gltf_loose_uris) lists.
pub fn from_gltf_bytes<D: DecodeBase64>(
    dependencies: &D,
    bytes: &[u8],
    loose_files: BTreeMap<String, Vec<u8>>,
) -> Result<GltfMeshMain> {
    let (json, blob) = if is_glb(bytes) {
        parse_glb(bytes)?
    } else {
        (bytes.to_vec(), None)
    };

    let root = Root::from_slice(&json).map_err(gltf::Error::Deserialize)?;

    from_gltf_file(
        dependencies,
        &GltfFile {
            root,
            blob,
            loose_files,
        },
    )
}
