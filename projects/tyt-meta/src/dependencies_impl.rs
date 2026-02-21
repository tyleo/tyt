use crate::{Dependencies, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(fs::create_dir_all(path)?)
    }

    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }

    fn write<P: AsRef<Path>>(&self, path: P, contents: &str) -> Result<()> {
        Ok(fs::write(path, contents)?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn workspace_root(&self) -> Result<PathBuf> {
        let mut dir = env::current_dir()?;
        loop {
            let cargo_toml = dir.join("Cargo.toml");
            if cargo_toml.is_file() {
                let contents = fs::read_to_string(&cargo_toml)?;
                if contents.contains("[workspace]") {
                    return Ok(dir);
                }
            }
            if !dir.pop() {
                return Err(crate::Error::Meta(
                    "could not find workspace root (no Cargo.toml with [workspace])".into(),
                ));
            }
        }
    }
}
