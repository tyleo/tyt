use crate::{Dependencies, DependenciesImpl, DirectoryEntry, ListDir, ReadFile, WriteFile};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{Error as IOError, ErrorKind, Result as IOResult},
    path::{Path, PathBuf},
};

/// An in-memory filesystem over the real codec impls.
#[derive(Default)]
pub struct MemoryFiles(pub RefCell<BTreeMap<PathBuf, Vec<u8>>>);

impl Dependencies for MemoryFiles {
    #[cfg(feature = "goxl")]
    type Goxl = <DependenciesImpl as Dependencies>::Goxl;

    #[cfg(feature = "goxl")]
    fn goxl(&self) -> &Self::Goxl {
        DependenciesImpl.goxl()
    }

    #[cfg(feature = "qbcl")]
    type Qbcl = <DependenciesImpl as Dependencies>::Qbcl;

    #[cfg(feature = "qbcl")]
    fn qbcl(&self) -> &Self::Qbcl {
        DependenciesImpl.qbcl()
    }

    #[cfg(feature = "vmax")]
    type VMax = <DependenciesImpl as Dependencies>::VMax;

    #[cfg(feature = "vmax")]
    fn vmax(&self) -> &Self::VMax {
        DependenciesImpl.vmax()
    }

    #[cfg(feature = "voxj")]
    type Voxj = <DependenciesImpl as Dependencies>::Voxj;

    #[cfg(feature = "voxj")]
    fn voxj(&self) -> &Self::Voxj {
        DependenciesImpl.voxj()
    }
}

impl ReadFile for MemoryFiles {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        self.0
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| IOError::from(ErrorKind::NotFound))
    }
}

impl ListDir for MemoryFiles {
    fn list_dir(&self, path: &Path) -> IOResult<Vec<DirectoryEntry>> {
        let mut entries: Vec<DirectoryEntry> = self
            .0
            .borrow()
            .keys()
            .filter_map(|file| {
                let relative = file.strip_prefix(path).ok()?;

                let first = relative.components().next()?;

                Some(DirectoryEntry {
                    path: path.join(first),
                    is_dir: relative.components().count() > 1,
                })
            })
            .collect();

        entries.dedup();

        Ok(entries)
    }
}

impl WriteFile for MemoryFiles {
    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
        self.0
            .borrow_mut()
            .insert(path.to_path_buf(), bytes.to_vec());

        Ok(())
    }
}
