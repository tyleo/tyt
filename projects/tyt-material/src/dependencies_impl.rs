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
    fn copy_file<P1: AsRef<Path>, P2: AsRef<Path>>(&self, from: P1, to: P2) -> Result<()> {
        fs::copy(from.as_ref(), to.as_ref())?;
        Ok(())
    }

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
}
