use crate::{Dependencies, Error, Prefs, RelConfig, Result};
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    fs::{self},
    io::Result as IOResult,
    path::{Path, PathBuf},
};
use tyt_preferences::{Dependencies as PrefsDependencies, DeserializePrefs as _};

/// Concrete implementation of filesystem operations.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        Ok(fs::create_dir_all(path)?)
    }

    fn current_dir(&self) -> Result<PathBuf> {
        Ok(env::current_dir()?)
    }

    fn exec_rg<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        tyt_injection::exec_map("rg", args, Error::IO, Error::Rg)
    }

    fn fs_prefs(&self) -> Result<Prefs> {
        let Some(layer) =
            tyt_preferences::load_git_prefs::<Prefs>(self, "fs").map_err(Error::IO)?
        else {
            return Ok(Prefs::default());
        };

        let mut prefs = layer.prefs.unwrap_or_default();
        if let Some(move_to_scratch) = prefs.move_to_scratch.as_mut()
            && let Some(scratch_dir) = move_to_scratch.scratch_dir.clone()
        {
            let scratch_path = PathBuf::from(scratch_dir);
            if !scratch_path.is_absolute() {
                move_to_scratch.scratch_dir =
                    Some(layer.dir.join(scratch_path).to_string_lossy().into_owned());
            }
        }
        Ok(prefs)
    }

    fn rel_config(&self) -> Result<Option<RelConfig>> {
        let Some(git_root) = self.git_root_dir().map_err(Error::IO)? else {
            return Ok(None);
        };

        let config = self
            .read_file(&git_root.join(".tytconfig"))
            .map_err(Error::IO)?;
        let usr_config = self
            .read_file(&git_root.join(".tytusrconfig"))
            .map_err(Error::IO)?;

        if config.is_none() && usr_config.is_none() {
            return Ok(None);
        }

        // `.tytconfig` is merged first so that any duplicate key in
        // `.tytusrconfig` overwrites it.
        let mut bases = HashMap::new();
        for bytes in [config, usr_config].into_iter().flatten() {
            let prefs: Option<Prefs> = Prefs::deserialize_prefs(&bytes, "fs").map_err(Error::IO)?;
            let Some(rel) = prefs.and_then(|prefs| prefs.rel) else {
                continue;
            };
            for (name, value) in rel {
                let path = PathBuf::from(&value);
                let resolved = if path.is_absolute() {
                    path
                } else {
                    git_root.join(path)
                };
                bases.insert(name, resolved);
            }
        }

        Ok(Some(RelConfig {
            config_dir: git_root,
            bases,
        }))
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        Ok(fs::rename(from, to)?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}

impl PrefsDependencies for DependenciesImpl {
    fn current_dir(&self) -> IOResult<PathBuf> {
        env::current_dir()
    }

    fn user_home_dir(&self) -> IOResult<Option<PathBuf>> {
        Ok(tyt_injection::user_home_dir())
    }

    fn git_root_dir(&self) -> IOResult<Option<PathBuf>> {
        tyt_injection::git_root_dir()
    }

    fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
        tyt_injection::read_file_optional(path)
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> IOResult<()> {
        tyt_injection::write_file_atomic(path, contents)
    }
}
