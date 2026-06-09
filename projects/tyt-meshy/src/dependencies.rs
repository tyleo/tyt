use crate::{MeshRequest, MeshTask, MeshTaskFile, Result};
use std::path::{Path, PathBuf};

/// Dependencies for this crate's operations.
pub trait Dependencies {
    /// Loads the Meshy API key from the `meshy` section of `.tytusrconfig`, or
    /// `None` if it is not set.
    fn meshy_api_key(&self) -> Result<Option<String>>;

    /// Returns the current working directory.
    fn current_dir(&self) -> Result<PathBuf>;

    /// Reads a UTF-8 text file (e.g. a `--texture-prompt-file`).
    fn read_text(&self, path: &Path) -> Result<String>;

    /// Creates an image-to-3D task and returns its id.
    fn create_task(&self, api_key: &str, request: &MeshRequest) -> Result<String>;

    /// Retrieves a task by id.
    fn get_task(&self, api_key: &str, task_id: &str) -> Result<MeshTask>;

    /// Downloads the bytes at a (pre-signed) URL.
    fn download(&self, url: &str) -> Result<Vec<u8>>;

    /// Writes raw bytes to a file atomically.
    fn write_file(&self, path: &Path, bytes: &[u8]) -> Result<()>;

    /// Writes the `*.meshy.mesh.json` task file atomically.
    fn write_task_file(&self, path: &Path, file: &MeshTaskFile) -> Result<()>;

    /// Sleeps for the given number of seconds, between poll attempts.
    fn sleep(&self, seconds: u64) -> Result<()>;

    /// Writes bytes to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
