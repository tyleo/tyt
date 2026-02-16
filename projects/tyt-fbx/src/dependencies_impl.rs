use crate::{Dependencies, Error, Result};
use std::{
    ffi::OsStr,
    fs,
    io::{Error as IOError, ErrorKind, Result as IOResult, Write},
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

impl Dependencies for DependenciesImpl {
    fn create_temp_dir(&self) -> Result<PathBuf> {
        // Try a handful of times in the unlikely event of a name collision.
        for _ in 0..16 {
            let path = self.unique_temp_path()?;
            match fs::create_dir(&path) {
                Ok(()) => return Ok(path),
                Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        }

        Err(IOError::new(
            ErrorKind::AlreadyExists,
            "failed to create a unique temp dir after multiple attempts",
        )
        .into())
    }

    fn exec_blender_script<
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    >(
        &self,
        script_dir: P1,
        script_py_path: P2,
        args: I,
    ) -> Result<Vec<u8>> {
        let output = process::Command::new("blender")
            .arg("--background")
            .arg("--python-expr")
            .arg(format!(
                "import sys; sys.path.insert(0, r'{}')",
                script_dir.as_ref().display(),
            ))
            .arg("--python")
            .arg(script_py_path.as_ref())
            .arg("--")
            .args(args)
            .output()?;

        if !output.status.success() {
            return Err(Error::Blender {
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        if !output.stderr.is_empty() {
            return Err(Error::Blender {
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        Ok(output.stdout)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        // If it's already gone, treat as success.
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        std::io::stdout().write_all(contents)?;
        Ok(())
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
        let tmp = self.unique_sibling_temp_path(path)?;

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

impl DependenciesImpl {
    fn unique_temp_path(&self) -> IOResult<PathBuf> {
        let mut base = std::env::temp_dir();

        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(IOError::other)?
            .as_nanos();

        let pid = process::id();
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);

        base.push(format!("tyt-{}-{}-{}", pid, now_ns, n));
        Ok(base)
    }

    fn unique_sibling_temp_path(&self, dst: &Path) -> IOResult<PathBuf> {
        let parent = dst.parent().unwrap_or_else(|| Path::new("."));
        let file_name = dst.file_name().and_then(|s| s.to_str()).unwrap_or("file");

        let now_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(IOError::other)?
            .as_nanos();

        let pid = process::id();
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);

        let mut tmp = parent.to_path_buf();
        tmp.push(format!(".{}.tmp-{}-{}-{}", file_name, pid, now_ns, n));
        Ok(tmp)
    }
}
