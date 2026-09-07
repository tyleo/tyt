use crate::{Dependencies, Result, WriteFile, WriteFormat, write, write_document_files};
use std::path::Path;
use voxcore::{VoxMain, ext::VoxExt};

/// Writes a state as the document at `output`: [`write`](crate::write) then
/// [`write_document_files`](crate::write_document_files).
pub fn save<D: Dependencies + WriteFile, T: VoxExt>(
    dependencies: &D,
    format: &WriteFormat,
    state: VoxMain<T>,
    output: &Path,
) -> Result<()> {
    let files = write(dependencies, format, state)?;

    write_document_files(dependencies, output, &files)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        Dependencies, DependenciesImpl, DirectoryEntry, ListDir, ReadFile, ReadFormat, WriteFile,
        WriteFormat, load, save, test_state, vmax::VMaxWriteOptions,
    };
    use std::{
        cell::RefCell,
        collections::BTreeMap,
        io::{Error as IOError, ErrorKind, Result as IOResult},
        path::{Path, PathBuf},
    };
    use voxcore::VoxMain;

    /// An in-memory filesystem over the real codec impls.
    #[derive(Default)]
    struct Memory(RefCell<BTreeMap<PathBuf, Vec<u8>>>);

    impl Dependencies for Memory {
        type Goxl = <DependenciesImpl as Dependencies>::Goxl;

        fn goxl(&self) -> &Self::Goxl {
            DependenciesImpl.goxl()
        }

        type Qbcl = <DependenciesImpl as Dependencies>::Qbcl;

        fn qbcl(&self) -> &Self::Qbcl {
            DependenciesImpl.qbcl()
        }

        type VMax = <DependenciesImpl as Dependencies>::VMax;

        fn vmax(&self) -> &Self::VMax {
            DependenciesImpl.vmax()
        }

        type Voxj = <DependenciesImpl as Dependencies>::Voxj;

        fn voxj(&self) -> &Self::Voxj {
            DependenciesImpl.voxj()
        }
    }

    impl ReadFile for Memory {
        fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
            self.0
                .borrow()
                .get(path)
                .cloned()
                .ok_or_else(|| IOError::from(ErrorKind::NotFound))
        }
    }

    impl ListDir for Memory {
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

    impl WriteFile for Memory {
        fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
            self.0
                .borrow_mut()
                .insert(path.to_path_buf(), bytes.to_vec());

            Ok(())
        }
    }

    #[test]
    fn a_saved_single_file_loads_back() {
        let memory = Memory::default();

        save(
            &memory,
            &WriteFormat::MVox,
            test_state(()),
            Path::new("out/m.vox"),
        )
        .unwrap();

        assert_eq!(memory.0.borrow().len(), 1);

        let loaded: VoxMain<()> = load(&memory, ReadFormat::MVox, Path::new("out/m.vox")).unwrap();

        assert_eq!(loaded.object_count(), 1);
    }

    #[test]
    fn a_saved_package_loads_back() {
        let memory = Memory::default();

        save(
            &memory,
            &WriteFormat::VMax(VMaxWriteOptions::default()),
            test_state(()),
            Path::new("out/p.vmax"),
        )
        .unwrap();

        assert!(
            memory
                .0
                .borrow()
                .contains_key(Path::new("out/p.vmax/scene.json"))
        );

        let loaded: VoxMain<()> = load(&memory, ReadFormat::VMax, Path::new("out/p.vmax")).unwrap();

        assert_eq!(loaded.object_count(), 1);
    }
}
