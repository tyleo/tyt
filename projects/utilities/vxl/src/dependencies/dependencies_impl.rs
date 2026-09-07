use crate::{DirectoryEntry, ListDir, ReadFile, TerminalColumns, WriteFile, WriteStdout};
#[cfg(unix)]
use libc::{STDOUT_FILENO, TIOCGWINSZ, ioctl, winsize};
#[cfg(unix)]
use std::mem;
use std::{
    fs,
    io::{self, Result as IOResult, Write},
    path::Path,
};

/// The dependencies over std's filesystem and standard output.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl ReadFile for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        fs::read(path)
    }
}

impl ListDir for DependenciesImpl {
    fn list_dir(&self, path: &Path) -> IOResult<Vec<DirectoryEntry>> {
        fs::read_dir(path)?
            .map(|entry| {
                let entry = entry?;

                Ok(DirectoryEntry {
                    path: entry.path(),
                    is_dir: entry.file_type()?.is_dir(),
                })
            })
            .collect()
    }
}

impl WriteFile for DependenciesImpl {
    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, bytes)
    }
}

impl WriteStdout for DependenciesImpl {
    fn write_stdout(&self, contents: &[u8]) -> IOResult<()> {
        io::stdout().write_all(contents)
    }
}

impl TerminalColumns for DependenciesImpl {
    #[cfg(unix)]
    fn terminal_columns(&self) -> Option<usize> {
        // Safety: winsize is plain data; ioctl fills it for the stdout fd, and
        // the result is read only when the call reports success.
        unsafe {
            let mut size: winsize = mem::zeroed();
            if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) == 0 && size.ws_col > 0 {
                Some(size.ws_col as usize)
            } else {
                None
            }
        }
    }

    /// No terminal-width detection off unix; the `rows` layout does not wrap.
    #[cfg(not(unix))]
    fn terminal_columns(&self) -> Option<usize> {
        None
    }
}
