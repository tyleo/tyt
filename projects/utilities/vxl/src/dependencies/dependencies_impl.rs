use crate::{
    CreateDirAll, CreateDirLink, CreateTempDir, DirectoryEntry, DisplayImage, ListDir, PathKind,
    ReadFile, ReadPathKind, RemoveDir, RenamePath, ResolvePrefsPaths, RunProgram, TerminalColumns,
    WriteFile, WriteStderr, WriteStdout,
};
use crossterm::{
    event::{poll, read},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};
use image::{DynamicImage, RgbaImage};
#[cfg(unix)]
use libc::{STDOUT_FILENO, TIOCGWINSZ, ioctl, winsize};
use meshconv::{
    DependenciesImpl as MeshconvDependenciesImpl, DirectoryEntry as MeshDirectoryEntry,
    ForwardDependencies as ForwardMeshDependencies, ListDir as MeshListDir,
    ReadFile as MeshReadFile, WriteFile as MeshWriteFile,
};
use sdfconv::{
    DependenciesImpl as SdfconvDependenciesImpl, ForwardDependencies as ForwardSdfDependencies,
    ReadFile as SdfReadFile,
};
#[cfg(windows)]
use std::os::windows::fs::symlink_dir;
use std::{
    ffi::OsString,
    fs,
    io::{self, Error as IOError, ErrorKind, IsTerminal, Result as IOResult, Write},
    path::Path,
    process::Command,
    time::Duration,
};
#[cfg(unix)]
use std::{mem, os::unix::fs::symlink};
use tempfile::{Builder, TempDir};
use ty_preferences::{
    Dependencies as PreferencesDependencies, DependenciesImpl as PreferencesDependenciesImpl,
    PrefsPaths, resolve_prefs_paths,
};
use viuer::{Config, print};
use voxconv::{DependenciesImpl as VoxconvDependenciesImpl, ForwardDependencies};

/// The dependencies over the real filesystem, processes, and terminal. The
/// codec and config traits forward to their crates' impls.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl ForwardDependencies for DependenciesImpl {
    type Target = VoxconvDependenciesImpl;

    fn target(&self) -> &VoxconvDependenciesImpl {
        &VoxconvDependenciesImpl
    }
}

impl ForwardMeshDependencies for DependenciesImpl {
    type Target = MeshconvDependenciesImpl;

    fn target(&self) -> &MeshconvDependenciesImpl {
        &MeshconvDependenciesImpl
    }
}

impl MeshReadFile for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        ReadFile::read_file(self, path)
    }
}

impl MeshListDir for DependenciesImpl {
    fn list_dir(&self, path: &Path) -> IOResult<Vec<MeshDirectoryEntry>> {
        Ok(ListDir::list_dir(self, path)?
            .into_iter()
            .map(|entry| MeshDirectoryEntry {
                path: entry.path,
                is_dir: entry.is_dir,
            })
            .collect())
    }
}

impl MeshWriteFile for DependenciesImpl {
    fn write_file(&self, path: &Path, bytes: &[u8]) -> IOResult<()> {
        WriteFile::write_file(self, path, bytes)
    }
}

impl ForwardSdfDependencies for DependenciesImpl {
    type Target = SdfconvDependenciesImpl;

    fn target(&self) -> &SdfconvDependenciesImpl {
        &SdfconvDependenciesImpl
    }
}

impl SdfReadFile for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        ReadFile::read_file(self, path)
    }
}

impl PreferencesDependencies for DependenciesImpl {
    fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
        PreferencesDependenciesImpl.read_file(path)
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> IOResult<()> {
        PreferencesDependenciesImpl.write_file(path, contents)
    }
}

impl ResolvePrefsPaths for DependenciesImpl {
    fn resolve_prefs_paths(&self) -> IOResult<PrefsPaths> {
        resolve_prefs_paths()
    }
}

impl CreateTempDir for DependenciesImpl {
    fn create_temp_dir(&self) -> IOResult<TempDir> {
        Builder::new().prefix("vxl-").tempdir()
    }
}

impl RunProgram for DependenciesImpl {
    fn run_program(&self, program: &str, args: &[OsString]) -> IOResult<Option<i32>> {
        Ok(Command::new(program).args(args).status()?.code())
    }
}

impl ReadPathKind for DependenciesImpl {
    fn read_path_kind(&self, path: &Path) -> IOResult<PathKind> {
        let file_type = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata.file_type(),

            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(PathKind::Missing),

            Err(e) => return Err(e),
        };

        if file_type.is_symlink() {
            Ok(PathKind::Symlink {
                target: fs::read_link(path)?,
            })
        } else if file_type.is_dir() {
            Ok(PathKind::Directory)
        } else {
            Ok(PathKind::File)
        }
    }
}

impl CreateDirAll for DependenciesImpl {
    fn create_dir_all(&self, path: &Path) -> IOResult<()> {
        fs::create_dir_all(path)
    }
}

impl CreateDirLink for DependenciesImpl {
    #[cfg(unix)]
    fn create_dir_link(&self, target: &Path, link: &Path) -> IOResult<()> {
        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent)?;
        }

        symlink(target, link)
    }

    /// Windows lets only an administrator or Developer Mode create a symlink.
    /// Anyone else gets `ERROR_PRIVILEGE_NOT_HELD`.
    #[cfg(windows)]
    fn create_dir_link(&self, target: &Path, link: &Path) -> IOResult<()> {
        const ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;

        if let Some(parent) = link.parent() {
            fs::create_dir_all(parent)?;
        }

        symlink_dir(target, link).map_err(|e| match e.raw_os_error() {
            Some(ERROR_PRIVILEGE_NOT_HELD) => IOError::new(
                e.kind(),
                "creating a symlink needs Developer Mode or an administrator",
            ),

            _ => e,
        })
    }

    #[cfg(not(any(unix, windows)))]
    fn create_dir_link(&self, _target: &Path, _link: &Path) -> IOResult<()> {
        Err(IOError::new(
            ErrorKind::Unsupported,
            "this platform has no directory symlinks",
        ))
    }
}

impl RenamePath for DependenciesImpl {
    fn rename_path(&self, from: &Path, to: &Path) -> IOResult<()> {
        fs::rename(from, to)
    }
}

impl RemoveDir for DependenciesImpl {
    fn remove_dir(&self, path: &Path) -> IOResult<()> {
        fs::remove_dir(path)
    }
}

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

impl WriteStderr for DependenciesImpl {
    fn write_stderr(&self, contents: &[u8]) -> IOResult<()> {
        io::stderr().write_all(contents)
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

    /// No terminal-width detection off unix; the `text-rows` layout does not wrap.
    #[cfg(not(unix))]
    fn terminal_columns(&self) -> Option<usize> {
        None
    }
}

impl DisplayImage for DependenciesImpl {
    /// Probes for a graphics protocol only when standard input and output
    /// are both a terminal, because the probe reads its reply from the
    /// terminal in raw mode. A piped run prints half blocks unprobed.
    fn display_image(&self, width: u32, height: u32, rgba: &[u8]) -> IOResult<()> {
        let image = DynamicImage::ImageRgba8(
            RgbaImage::from_raw(width, height, rgba.to_vec()).expect("the samples fill the image"),
        );

        let interactive = io::stdout().is_terminal() && io::stdin().is_terminal();

        let config = display_config(interactive);

        let show = || print(&image, &config).map(|_| ()).map_err(IOError::other);

        if interactive {
            with_capability_probe_guard(show)
        } else {
            show()
        }
    }
}

/// The viuer config for an inline image: anchored at the cursor over the
/// terminal's background, probing for the Kitty and iTerm2 protocols only
/// under `probe`.
fn display_config(probe: bool) -> Config {
    // No Windows terminal supports the Kitty graphics or iTerm2 inline-image
    // protocols, but viuer still probes for them by writing escape sequences
    // and blocking on the reply, which Windows Terminal never sends.
    let probe = probe && !cfg!(windows);

    Config {
        absolute_offset: false,
        transparent: true,
        use_kitty: probe,
        use_iterm: probe,
        ..Default::default()
    }
}

/// Runs `print`, which writes terminal graphics escape sequences, in raw
/// mode, so the capability-probe replies are not echoed as visible garbage.
/// Because viuer toggles raw mode itself, raw mode is re-asserted afterwards
/// and straggling reply bytes are drained before the original mode returns.
fn with_capability_probe_guard<T>(print: impl FnOnce() -> IOResult<T>) -> IOResult<T> {
    let was_raw = is_raw_mode_enabled()?;
    enable_raw_mode()?;

    let result = print();

    let drained = enable_raw_mode().and_then(|()| drain_replies());
    let restored = if was_raw { Ok(()) } else { disable_raw_mode() };

    let value = result?;
    drained?;
    restored?;

    Ok(value)
}

fn drain_replies() -> IOResult<()> {
    while poll(Duration::from_millis(50))? {
        read()?;
    }

    Ok(())
}
