use crate::{Dependencies, Result, WriteFile, WriteFormat, write, write_document_files};
use meshdoc::MeshMain;
use std::path::Path;

/// Writes a bare state as the document at `output`: [`write`](crate::write)
/// then [`write_document_files`](crate::write_document_files).
pub fn save<D: Dependencies + WriteFile>(
    dependencies: &D,
    format: &WriteFormat,
    main: MeshMain<()>,
    output: &Path,
) -> Result<()> {
    let files = write(dependencies, format, main)?;

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
        gltf::{GltfContainer, GltfImageStorage, GltfWriteFormat, GltfWriteOptions},
        load, save, test_main,
    };
    use std::path::Path;

    #[test]
    fn a_saved_single_file_loads_back() {
        let memory = MemoryFiles::default();

        save(
            &memory,
            &WriteFormat::from(ReadFormat::Gltf),
            test_main(()),
            Path::new("out/m.glb"),
        )
        .unwrap();

        assert_eq!(memory.0.borrow().len(), 1);

        let loaded = load(&memory, ReadFormat::Gltf, Path::new("out/m.glb")).unwrap();

        assert_eq!(loaded.object_count(), 1);
    }

    #[test]
    fn a_saved_document_with_loose_files_loads_back() {
        let memory = MemoryFiles::default();

        save(
            &memory,
            &WriteFormat::Gltf(GltfWriteFormat {
                container: GltfContainer::Gltf,
                options: GltfWriteOptions {
                    images: GltfImageStorage::Loose,
                },
            }),
            test_main(()),
            Path::new("out/m.gltf"),
        )
        .unwrap();

        assert!(memory.0.borrow().contains_key(Path::new("out/atlas.png")));

        let loaded = load(&memory, ReadFormat::Gltf, Path::new("out/m.gltf")).unwrap();

        assert_eq!(loaded.image_count(), 1);
    }
}
