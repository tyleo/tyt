use crate::{
    MeshOutput, MeshRequest, MeshTask, MeshTaskFile, Result, TaskFileHead, TextureRequest,
    TextureTaskFile,
};
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

    /// Reads the `taskId` from a source `*.meshy.mesh.json` task file.
    fn read_input_task_id(&self, path: &Path) -> Result<String>;

    /// Reads the head (`taskId`, `taskKind`, and verbatim `payload.input`) of a
    /// task file, for polling and rewriting it in place.
    fn read_task_file(&self, path: &Path) -> Result<TaskFileHead>;

    /// Creates an image-to-3D task and returns its id.
    fn create_task(&self, api_key: &str, request: &MeshRequest) -> Result<String>;

    /// Creates a retexture task and returns its id.
    fn create_texture_task(&self, api_key: &str, request: &TextureRequest) -> Result<String>;

    /// Retrieves an image-to-3D task by id.
    fn get_task(&self, api_key: &str, task_id: &str) -> Result<MeshTask>;

    /// Retrieves a retexture task by id.
    fn get_texture_task(&self, api_key: &str, task_id: &str) -> Result<MeshTask>;

    /// Downloads the bytes at a (pre-signed) URL.
    fn download(&self, url: &str) -> Result<Vec<u8>>;

    /// Renders an image file inline in the terminal.
    fn display_image_in_terminal(&self, path: &Path) -> Result<()>;

    /// Writes raw bytes to a file atomically.
    fn write_file(&self, path: &Path, bytes: &[u8]) -> Result<()>;

    /// Writes the `*.meshy.mesh.json` task file atomically.
    fn write_task_file(&self, path: &Path, file: &MeshTaskFile) -> Result<()>;

    /// Writes the `*.meshy.texture.json` task file atomically.
    fn write_texture_task_file(&self, path: &Path, file: &TextureTaskFile) -> Result<()>;

    /// Rewrites a polled task file in place, preserving its `payload.input` and
    /// `taskKind` and replacing its `output`.
    fn write_polled_task_file(
        &self,
        path: &Path,
        head: &TaskFileHead,
        output: &MeshOutput,
    ) -> Result<()>;

    /// Sleeps for the given number of seconds, between poll attempts.
    fn sleep(&self, seconds: u64) -> Result<()>;

    /// Writes bytes to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
