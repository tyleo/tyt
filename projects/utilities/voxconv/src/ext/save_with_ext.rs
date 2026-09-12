use crate::{
    Dependencies, Result, WriteFile, WriteFormat,
    ext::{VoxconvVoxMain, write_with_ext},
    write_document_files,
};
use std::path::Path;

/// Writes a state as the document at `output`:
/// [`write_with_ext`](crate::ext::write_with_ext) then
/// [`write_document_files`](crate::write_document_files).
pub fn save_with_ext<D: Dependencies + WriteFile>(
    dependencies: &D,
    format: &WriteFormat,
    main: VoxconvVoxMain,
    output: &Path,
) -> Result<()> {
    let files = write_with_ext(dependencies, format, main)?;

    write_document_files(dependencies, output, &files)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        MemoryFiles, ReadFormat, WriteFormat,
        ext::{load_with_ext, save_with_ext},
        save, test_main,
        vmax::VMaxWriteOptions,
    };
    use std::path::Path;
    use vmax_voxcore::VMaxExt;

    #[test]
    fn a_saved_package_loads_back_with_its_ext() {
        let memory = MemoryFiles::default();

        let vmax = WriteFormat::VMax(VMaxWriteOptions::default());

        save(&memory, &vmax, test_main(()), Path::new("out/p.vmax")).unwrap();

        let loaded = load_with_ext(&memory, ReadFormat::VMax, Path::new("out/p.vmax")).unwrap();

        save_with_ext(&memory, &vmax, loaded, Path::new("out/q.vmax")).unwrap();

        let loaded = load_with_ext(&memory, ReadFormat::VMax, Path::new("out/q.vmax")).unwrap();

        assert!(loaded.ext().is::<VMaxExt>());

        assert_eq!(loaded.object_count(), 1);
    }
}
