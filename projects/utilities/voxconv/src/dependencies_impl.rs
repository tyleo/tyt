use crate::{Dependencies, DirectoryEntry, ListDir, ReadFile, WriteFile};
#[cfg(feature = "goxl")]
use goxl_voxcore::codec::dependencies::DependenciesImpl as GoxlDependenciesImpl;
#[cfg(feature = "qbcl")]
use qbcl_voxcore::codec::dependencies::DependenciesImpl as QbclDependenciesImpl;
use std::{fs, io::Result as IOResult, path::Path};
#[cfg(feature = "vmax")]
use vmax_voxcore::codec::dependencies::DependenciesImpl as VMaxDependenciesImpl;
#[cfg(feature = "voxj")]
use voxj_voxcore::codec::dependencies::DependenciesImpl as VoxjDependenciesImpl;

/// The dependencies over std's filesystem and each enabled format's codec
/// impl.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    #[cfg(feature = "goxl")]
    type Goxl = GoxlDependenciesImpl;

    #[cfg(feature = "goxl")]
    fn goxl(&self) -> &Self::Goxl {
        &GoxlDependenciesImpl
    }

    #[cfg(feature = "qbcl")]
    type Qbcl = QbclDependenciesImpl;

    #[cfg(feature = "qbcl")]
    fn qbcl(&self) -> &Self::Qbcl {
        &QbclDependenciesImpl
    }

    #[cfg(feature = "vmax")]
    type VMax = VMaxDependenciesImpl;

    #[cfg(feature = "vmax")]
    fn vmax(&self) -> &Self::VMax {
        &VMaxDependenciesImpl
    }

    #[cfg(feature = "voxj")]
    type Voxj = VoxjDependenciesImpl;

    #[cfg(feature = "voxj")]
    fn voxj(&self) -> &Self::Voxj {
        &VoxjDependenciesImpl
    }
}

impl ReadFile for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        fs::read(path)
    }
}

impl ListDir for DependenciesImpl {
    fn list_dir(&self, path: &Path) -> IOResult<Vec<DirectoryEntry>> {
        fs::read_dir(path)?
            .map(|entry| {
                let entry = entry?;

                Ok(DirectoryEntry {
                    path: entry.path(),
                    is_dir: entry.file_type()?.is_dir(),
                })
            })
            .collect()
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
