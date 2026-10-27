use crate::{ReadFile, WriteFile};
use std::{fs, io::Result as IOResult, path::Path};

/// The dependencies over std's filesystem and each enabled format's codec
/// impl. Each format module implements its dependencies trait for it.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl ReadFile for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        fs::read(path)
    }
}

impl WriteFile for DependenciesImpl {
    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, bytes)
    }
}
