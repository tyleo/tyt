use crate::{Dependencies, Result, WriteFile, WriteFormat, write};
use sdfcore::SdfMain;
use std::path::Path;

/// Writes an [`SdfMain`] as the document at `output`: [`write()`], then the
/// file.
pub fn save<D: Dependencies + WriteFile>(
    dependencies: &D,
    format: &WriteFormat,
    main: &SdfMain,
    output: &Path,
) -> Result<()> {
    Ok(dependencies.write_file(output, &write(dependencies, format, main)?)?)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{LoadOptions, MemoryFiles, ReadFormat, WriteFormat, load, save, test_main};
    use std::path::Path;

    #[test]
    fn a_saved_document_loads_back() {
        let memory = MemoryFiles::default();

        save(
            &memory,
            &WriteFormat::from(ReadFormat::Sdfj),
            &test_main(),
            Path::new("out/m.sdfj"),
        )
        .unwrap();

        assert_eq!(memory.0.borrow().len(), 1);

        let loaded = load(
            &memory,
            ReadFormat::Sdfj,
            Path::new("out/m.sdfj"),
            LoadOptions::default(),
        )
        .unwrap();

        assert_eq!(loaded, test_main());
    }

    #[test]
    fn a_missing_file_errors() {
        assert!(
            load(
                &MemoryFiles::default(),
                ReadFormat::Sdfj,
                Path::new("m.sdfj"),
                LoadOptions::default()
            )
            .is_err()
        );
    }
}
