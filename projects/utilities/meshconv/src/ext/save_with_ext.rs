use crate::{
    Dependencies, Result, WriteFile, WriteFormat,
    ext::{MeshconvMeshMain, write_with_ext},
    write_document_files,
};
use std::path::Path;

/// Writes a state as the document at `output`:
/// [`write_with_ext`](crate::ext::write_with_ext) then
/// [`write_document_files`](crate::write_document_files).
pub fn save_with_ext<D: Dependencies + WriteFile>(
    dependencies: &D,
    format: &WriteFormat,
    main: MeshconvMeshMain,
    output: &Path,
) -> Result<()> {
    let files = write_with_ext(dependencies, format, main)?;

    write_document_files(
        dependencies,
        output,
        format.read_format().is_package(),
        &files,
    )
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        MemoryFiles, ReadFormat, WriteFormat,
        ext::{load_with_ext, save_with_ext},
        save, test_main,
    };
    use gltf_meshdoc::GltfExt;
    use std::path::Path;

    #[test]
    fn a_saved_document_loads_back_with_its_ext() {
        let memory = MemoryFiles::default();

        let format = WriteFormat::from(ReadFormat::Gltf);

        save(&memory, &format, test_main(()), Path::new("out/m.glb")).unwrap();

        let loaded = load_with_ext(&memory, ReadFormat::Gltf, Path::new("out/m.glb")).unwrap();

        save_with_ext(&memory, &format, loaded, Path::new("out/n.glb")).unwrap();

        let loaded = load_with_ext(&memory, ReadFormat::Gltf, Path::new("out/n.glb")).unwrap();

        assert!(loaded.ext().is::<GltfExt>());

        assert_eq!(loaded.object_count(), 1);
    }
}
