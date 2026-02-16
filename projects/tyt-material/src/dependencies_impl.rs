use crate::{Dependencies, Error, Result};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn copy_file<P1: AsRef<Path>, P2: AsRef<Path>>(&self, from: P1, to: P2) -> Result<()> {
        fs::copy(from.as_ref(), to.as_ref())?;
        Ok(())
    }

    fn create_temp_dir(&self) -> Result<PathBuf> {
        Ok(tyt_common::create_temp_dir()?)
    }

    fn exec_magick<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = process::Command::new("magick").args(args).output()?;

        if !output.status.success() {
            return Err(Error::Magick {
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        Ok(output.stdout)
    }

    fn glob_single_match(&self, pattern: &str) -> Result<PathBuf> {
        let mut matches = Vec::new();
        for entry in glob::glob(pattern)
            .map_err(|e| Error::Glob(format!("invalid glob pattern '{pattern}': {e}")))?
        {
            matches
                .push(entry.map_err(|e| Error::Glob(format!("error reading glob result: {e}")))?);
        }

        match matches.len() {
            0 => Err(Error::Glob(format!("missing file matching: {pattern}"))),
            1 => Ok(matches.into_iter().next().unwrap()),
            n => {
                let mut msg = format!("multiple files ({n}) match '{pattern}':");
                for f in &matches {
                    msg.push_str(&format!("\n  {}", f.display()));
                }
                Err(Error::Glob(msg))
            }
        }
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(tyt_common::remove_dir_all(path.as_ref())?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_common::write_stdout(contents)?)
    }
}
