use crate::{Dependencies, Error, Result};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn create_temp_dir(&self) -> Result<PathBuf> {
        Ok(tyt_injection::create_temp_dir()?)
    }

    fn exec_blender_script<
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    >(
        &self,
        script_dir: P1,
        script_py_path: P2,
        args: I,
    ) -> Result<Vec<u8>> {
        let blender_args = tyt_injection::Args::new()
            .arg("--background")
            .arg("--python-expr")
            .arg(format!(
                "import sys; sys.path.insert(0, r'{}')",
                script_dir.as_ref().display(),
            ))
            .arg("--python")
            .arg(script_py_path.as_ref())
            .arg("--")
            .args(args);

        tyt_injection::exec_map("blender", blender_args, Error::IO, Error::Blender)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(tyt_injection::remove_dir_all(path.as_ref())?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn write_file<P: AsRef<Path>>(&self, path: P, contents: &[u8]) -> Result<()> {
        let path = path.as_ref();

        // Ensure parent exists (mimics common "write file" ergonomics).
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        // Write to a sibling temp file, then rename over destination.
        // This avoids leaving partial files on crash and is generally atomic on
        // the same filesystem.
        let tmp = tyt_injection::unique_sibling_temp_path(path)?;

        // Use a scope so the file handle is closed before rename (important on
        // Windows).
        {
            let mut f = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&tmp)?;

            f.write_all(contents)?;
            f.sync_all()?; // durability best-effort
        }

        // On Unix, rename over existing is atomic. On Windows, rename fails if
        // dest exists.
        // So: remove dest first on Windows-like behavior, then rename.
        match fs::rename(&tmp, path) {
            Ok(()) => Ok(()),
            Err(e) => {
                // If destination exists, try remove then rename.
                if e.kind() == ErrorKind::AlreadyExists || e.kind() == ErrorKind::PermissionDenied {
                    // PermissionDenied is commonly observed on Windows when
                    // target exists.
                    let _ = fs::remove_file(path);
                    fs::rename(&tmp, path).inspect_err(|_rename_err| {
                        // Cleanup temp if we still failed.
                        let _ = fs::remove_file(&tmp);
                    })?;
                    Ok(())
                } else {
                    let _ = fs::remove_file(&tmp);
                    Err(e.into())
                }
            }
        }
    }
}
