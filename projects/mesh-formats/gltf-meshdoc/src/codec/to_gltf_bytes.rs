use crate::{EncodeBase64, GltfMeshMain, GltfWriteOptions, Result, codec::GltfBytes, to_gltf_file};

/// Writes a [`GltfMeshMain`] to the bytes of a `.gltf` file through
/// `dependencies`, the JSON container form of [`to_gltf_file`] and the
/// inverse of [`from_gltf_bytes`](crate::codec::from_gltf_bytes). The
/// geometry buffer rides as a data URI, or as the loose file the ext records
/// when the buffer was loaded from one. The images go where `options` says.
pub fn to_gltf_bytes<D: EncodeBase64>(
    dependencies: &D,
    main: &GltfMeshMain,
    options: &GltfWriteOptions,
) -> Result<GltfBytes> {
    let mut file = to_gltf_file(dependencies, main, options)?;

    if let (Some(blob), Some(buffer)) = (file.blob, file.root.buffers.first_mut()) {
        buffer.uri = Some(match &main.ext().buffer_uri {
            Some(uri) => {
                file.loose_files.insert(uri.clone(), blob);
                uri.clone()
            }
            None => format!(
                "data:application/octet-stream;base64,{}",
                dependencies.encode_base64(&blob)
            ),
        });
    }

    let primary = file.root.to_vec().map_err(gltf::Error::Deserialize)?;

    Ok(GltfBytes {
        primary,
        loose_files: file.loose_files,
    })
}
