use crate::Dependencies;
use std::{
    fs::{self, OpenOptions},
    io::{Error as IOError, ErrorKind, Result as IOResult, Write},
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

/// Concrete implementation of preference file I/O.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
        match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> IOResult<()> {
        write_file_atomic(path, contents)
    }
}

/// Writes a file atomically by writing to a sibling temp file and renaming over
/// the destination. Creates parent directories as needed.
fn write_file_atomic(path: &Path, contents: &[u8]) -> IOResult<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let tmp = unique_sibling_temp_path(path)?;

    {
        let mut f = OpenOptions::new().create_new(true).write(true).open(&tmp)?;

        f.write_all(contents)?;

        f.sync_all()?;
    }

    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists || e.kind() == ErrorKind::PermissionDenied {
                let _ = fs::remove_file(path);

                fs::rename(&tmp, path).inspect_err(|_| {
                    let _ = fs::remove_file(&tmp);
                })?;

                Ok(())
            } else {
                let _ = fs::remove_file(&tmp);

                Err(e)
            }
        }
    }
}

/// Returns a unique temp-file path in the same directory as `dst`.
fn unique_sibling_temp_path(dst: &Path) -> IOResult<PathBuf> {
    let parent = dst.parent().unwrap_or_else(|| Path::new("."));

    let file_name = dst.file_name().and_then(|s| s.to_str()).unwrap_or("file");

    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(IOError::other)?
        .as_nanos();

    let pid = process::id();

    let n = temp_counter_next();

    let mut tmp = parent.to_path_buf();

    tmp.push(format!(".{}.tmp-{}-{}-{}", file_name, pid, now_ns, n));

    Ok(tmp)
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Returns the next value of a process-wide counter for temp-file names.
fn temp_counter_next() -> u64 {
    TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
}
