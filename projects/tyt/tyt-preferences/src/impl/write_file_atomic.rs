use crate::unique_sibling_temp_path;
use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Result as IOResult, Write},
    path::Path,
};

/// Writes a file atomically by writing to a sibling temp file and renaming
/// over the destination. Creates parent directories as needed.
pub(crate) fn write_file_atomic(path: &Path, contents: &[u8]) -> IOResult<()> {
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
