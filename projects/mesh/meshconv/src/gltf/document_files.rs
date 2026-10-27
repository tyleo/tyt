use crate::MeshDocumentFile;
use gltf_meshdoc::codec::GltfBytes;

/// A written document's bytes as its files: the primary, then each loose
/// file at its relative path.
pub fn document_files(bytes: GltfBytes) -> Vec<MeshDocumentFile> {
    let mut files = vec![MeshDocumentFile::primary(bytes.primary)];

    files.extend(
        bytes
            .loose_files
            .into_iter()
            .map(|(path, bytes)| MeshDocumentFile::new(path, bytes)),
    );

    files
}
