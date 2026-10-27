use crate::{ReadFile, ResolvePrefsPaths};
use std::{
    collections::BTreeMap,
    io::{Error as IOError, ErrorKind, Result as IOResult},
    path::{Path, PathBuf},
};
use ty_preferences::{Dependencies as PreferencesDependencies, PrefsPaths};

/// A cascade over in-memory files, the working directory `/repo/sub` under
/// the git root `/repo` and the user's home `/home`. A command reads files
/// beyond the `.vxlconfig` layers through [`ReadFile`].
pub struct Cascade {
    in_repository: bool,

    files: BTreeMap<PathBuf, &'static str>,
}

impl Cascade {
    /// A cascade holding `files` as path and text pairs. Outside a repository
    /// the cascade has no git root.
    pub fn new(in_repository: bool, files: &[(&str, &'static str)]) -> Self {
        Cascade {
            in_repository,
            files: files
                .iter()
                .map(|&(path, text)| (PathBuf::from(path), text))
                .collect(),
        }
    }
}

impl PreferencesDependencies for Cascade {
    fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
        Ok(self.files.get(path).map(|text| text.as_bytes().to_vec()))
    }

    fn write_file(&self, _: &Path, _: &[u8]) -> IOResult<()> {
        unreachable!("loading never writes")
    }
}

impl ReadFile for Cascade {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        match self.files.get(path) {
            Some(text) => Ok(text.as_bytes().to_vec()),
            None => Err(IOError::from(ErrorKind::NotFound)),
        }
    }
}

impl ResolvePrefsPaths for Cascade {
    fn resolve_prefs_paths(&self) -> IOResult<PrefsPaths> {
        Ok(PrefsPaths {
            cwd: PathBuf::from("/repo/sub"),
            git_root: self.in_repository.then(|| PathBuf::from("/repo")),
            user: Some(PathBuf::from("/home")),
        })
    }
}
