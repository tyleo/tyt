use crate::{Conv, OaiRequest, OaiResponse, Result};
use std::path::{Path, PathBuf};

/// Dependencies for this crate's operations.
pub trait Dependencies {
    /// Loads the OpenAI API key from `.tytusrconfig`, or `None` if it is not set.
    fn oai_api_key(&self) -> Result<Option<String>>;

    /// Resolves the absolute `oai.img.systemPromptsDir`, merged from
    /// `.tytconfig` and `.tytusrconfig`, or `None` if it is not configured.
    fn system_prompts_dir(&self) -> Result<Option<PathBuf>>;

    /// Reads a system prompt file as text, or `None` if it does not exist.
    fn read_system_prompt(&self, path: &Path) -> Result<Option<String>>;

    /// Returns the current working directory.
    fn current_dir(&self) -> Result<PathBuf>;

    /// Returns whether `path` is an existing directory.
    fn is_dir(&self, path: &Path) -> Result<bool>;

    /// Reads the message from stdin, or an empty string if stdin is an
    /// interactive terminal.
    fn read_stdin(&self) -> Result<String>;

    /// Reads and parses the OpenAI image conversation file, or `None` if it does
    /// not exist.
    fn read_conv(&self, path: &Path) -> Result<Option<Conv>>;

    /// Writes the OpenAI image conversation file atomically.
    fn write_conv(&self, path: &Path, conv: &Conv) -> Result<()>;

    /// Writes raw image bytes to a file.
    fn write_image(&self, path: &Path, bytes: &[u8]) -> Result<()>;

    /// Sends an image-generation request to the OpenAI API.
    fn generate_image(&self, api_key: &str, request: &OaiRequest) -> Result<OaiResponse>;

    /// Writes bytes to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;

    /// Displays the image at the given path inline in the terminal.
    fn display_image_in_terminal(&self, path: &Path) -> Result<()>;
}
