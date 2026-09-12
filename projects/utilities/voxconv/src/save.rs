use crate::{Dependencies, Result, WriteFile, WriteFormat, write, write_document_files};
use std::path::Path;
use voxcore::VoxMain;

/// Writes a bare state as the document at `output`: [`write`](crate::write)
/// then [`write_document_files`](crate::write_document_files).
pub fn save<D: Dependencies + WriteFile>(
    dependencies: &D,
    format: &WriteFormat,
    main: VoxMain<()>,
    output: &Path,
) -> Result<()> {
    let files = write(dependencies, format, main)?;

    write_document_files(dependencies, output, &files)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        MemoryFiles, ReadFormat, WriteFormat, load, save, test_main, vmax::VMaxWriteOptions,
    };
    use std::path::Path;

    #[test]
    fn a_saved_single_file_loads_back() {
        let memory = MemoryFiles::default();

        save(
            &memory,
            &WriteFormat::MVox,
            test_main(()),
            Path::new("out/m.vox"),
        )
        .unwrap();

        assert_eq!(memory.0.borrow().len(), 1);

        let loaded = load(&memory, ReadFormat::MVox, Path::new("out/m.vox")).unwrap();

        assert_eq!(loaded.object_count(), 1);
    }

    #[test]
    fn a_saved_package_loads_back() {
        let memory = MemoryFiles::default();

        save(
            &memory,
            &WriteFormat::VMax(VMaxWriteOptions::default()),
            test_main(()),
            Path::new("out/p.vmax"),
        )
        .unwrap();

        assert!(
            memory
                .0
                .borrow()
                .contains_key(Path::new("out/p.vmax/scene.json"))
        );

        let loaded = load(&memory, ReadFormat::VMax, Path::new("out/p.vmax")).unwrap();

        assert_eq!(loaded.object_count(), 1);
    }
}
