use crate::{
    Dependencies, Format, MeshDocumentFile, ReadFormat, Result, WriteFormat,
    gltf::{GltfContainer, GltfWriteFormat},
};
use gltf_meshdoc::{
    codec::{GltfBytes, from_gltf_bytes, gltf_loose_uris, to_glb_bytes, to_gltf_bytes},
    to_gltf_mesh_main,
};
use meshdoc::MeshMain;
use std::collections::BTreeMap;

/// glTF 2.0, the `.glb` and `.gltf` files. Both containers are one format:
/// the reader detects the container, and the write options pick it.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Gltf;

impl Format for Gltf {
    type WriteOptions = GltfWriteFormat;

    const NAME: &'static str = "gltf";

    const EXTENSIONS: &'static [&'static str] = &["glb", "gltf"];

    fn read_format() -> ReadFormat {
        ReadFormat::Gltf
    }

    fn write_format(options: GltfWriteFormat) -> WriteFormat {
        WriteFormat::Gltf(options)
    }

    fn extension(options: &GltfWriteFormat) -> &'static str {
        options.container.extension()
    }

    /// The loose files the primary references by relative URI.
    fn loose_paths(primary: &[u8]) -> Result<Vec<String>> {
        Ok(gltf_loose_uris(primary)?)
    }

    fn read<D: Dependencies>(dependencies: &D, files: &[MeshDocumentFile]) -> Result<MeshMain<()>> {
        Ok(from_gltf_bytes(
            dependencies.gltf(),
            MeshDocumentFile::primary_bytes(files)?,
            loose_files(files),
        )?
        .take_ext()
        .main)
    }

    fn write<D: Dependencies>(
        dependencies: &D,
        options: &GltfWriteFormat,
        main: MeshMain<()>,
    ) -> Result<Vec<MeshDocumentFile>> {
        let main = to_gltf_mesh_main(main);

        let bytes = match options.container {
            GltfContainer::Glb => to_glb_bytes(dependencies.gltf(), &main, &options.options)?,
            GltfContainer::Gltf => to_gltf_bytes(dependencies.gltf(), &main, &options.options)?,
        };

        Ok(document_files(bytes))
    }
}

/// The files beside the primary, keyed by their relative paths.
pub(crate) fn loose_files(files: &[MeshDocumentFile]) -> BTreeMap<String, Vec<u8>> {
    MeshDocumentFile::loose_files(files)
        .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()))
        .collect()
}

/// A written document's bytes as its files: the primary, then each loose
/// file at its relative path.
pub(crate) fn document_files(bytes: GltfBytes) -> Vec<MeshDocumentFile> {
    let mut files = vec![MeshDocumentFile::primary(bytes.primary)];

    files.extend(
        bytes
            .loose_files
            .into_iter()
            .map(|(path, bytes)| MeshDocumentFile::new(path, bytes)),
    );

    files
}
