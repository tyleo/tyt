use crate::{
    CreateDirAll, CreateDirLink, DependenciesImpl, DirectoryEntry, ListDir, PathKind, ReadFile,
    ReadPathKind, RemoveDir, RenamePath, WriteFile, WriteStdout,
};
use std::{cell::RefCell, io::Result as IOResult, path::Path};

/// Real files and a captured standard output.
#[derive(Default)]
pub struct CapturedStdout {
    stdout: RefCell<Vec<u8>>,
}

impl CapturedStdout {
    /// Everything written to standard output so far.
    pub fn stdout(&self) -> String {
        String::from_utf8(self.stdout.borrow().clone()).unwrap()
    }
}

impl CreateDirAll for CapturedStdout {
    fn create_dir_all(&self, path: &Path) -> IOResult<()> {
        DependenciesImpl.create_dir_all(path)
    }
}

impl CreateDirLink for CapturedStdout {
    fn create_dir_link(&self, target: &Path, link: &Path) -> IOResult<()> {
        DependenciesImpl.create_dir_link(target, link)
    }
}

impl ListDir for CapturedStdout {
    fn list_dir(&self, path: &Path) -> IOResult<Vec<DirectoryEntry>> {
        ListDir::list_dir(&DependenciesImpl, path)
    }
}

impl ReadFile for CapturedStdout {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        ReadFile::read_file(&DependenciesImpl, path)
    }
}

impl ReadPathKind for CapturedStdout {
    fn read_path_kind(&self, path: &Path) -> IOResult<PathKind> {
        DependenciesImpl.read_path_kind(path)
    }
}

impl RemoveDir for CapturedStdout {
    fn remove_dir(&self, path: &Path) -> IOResult<()> {
        DependenciesImpl.remove_dir(path)
    }
}

impl RenamePath for CapturedStdout {
    fn rename_path(&self, from: &Path, to: &Path) -> IOResult<()> {
        DependenciesImpl.rename_path(from, to)
    }
}

impl WriteFile for CapturedStdout {
    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
        WriteFile::write_file(&DependenciesImpl, path, bytes)
    }
}

impl WriteStdout for CapturedStdout {
    fn write_stdout(&self, contents: &[u8]) -> IOResult<()> {
        self.stdout.borrow_mut().extend_from_slice(contents);

        Ok(())
    }
}
