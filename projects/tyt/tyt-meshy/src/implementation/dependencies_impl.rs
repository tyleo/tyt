use crate::{
    Dependencies, Error, MeshOutput, MeshRequest, MeshTask, MeshTaskFile, Result, TaskFileHead,
    TextureRequest, TextureTaskFile, UsrPrefs, implementation::meshy,
};
use std::{
    env, fs,
    io::{Error as IOError, ErrorKind, Result as IOResult},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use tyt_preferences::{Dependencies as PrefsDependencies, load_user_git_prefs};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn meshy_api_key(&self) -> Result<Option<String>> {
        let prefs: Option<UsrPrefs> = load_user_git_prefs(self, "meshy").map_err(Error::IO)?;
        Ok(prefs.and_then(|prefs| prefs.api_key))
    }

    fn current_dir(&self) -> Result<PathBuf> {
        Ok(env::current_dir()?)
    }

    fn read_text(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path).map_err(Error::IO)?;
        String::from_utf8(bytes).map_err(|e| Error::IO(IOError::new(ErrorKind::InvalidData, e)))
    }

    fn read_input_task_id(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path).map_err(Error::IO)?;
        meshy::read_task_id(&bytes)
    }

    fn read_task_file(&self, path: &Path) -> Result<TaskFileHead> {
        let bytes = fs::read(path).map_err(Error::IO)?;
        meshy::read_task_file_head(&bytes)
    }

    fn create_task(&self, api_key: &str, request: &MeshRequest) -> Result<String> {
        meshy::create_task(api_key, request)
    }

    fn create_texture_task(&self, api_key: &str, request: &TextureRequest) -> Result<String> {
        meshy::create_texture_task(api_key, request)
    }

    fn get_task(&self, api_key: &str, task_id: &str) -> Result<MeshTask> {
        meshy::get_task(api_key, task_id)
    }

    fn get_texture_task(&self, api_key: &str, task_id: &str) -> Result<MeshTask> {
        meshy::get_texture_task(api_key, task_id)
    }

    fn download(&self, url: &str) -> Result<Vec<u8>> {
        meshy::download(url)
    }

    fn display_image_in_terminal(&self, path: &Path) -> Result<()> {
        Ok(tyt_injection::display_image_in_terminal(path)?)
    }

    fn display_images_in_grid(
        &self,
        paths: &[&Path],
        columns: u32,
        side_percent: u32,
    ) -> Result<()> {
        Ok(tyt_injection::display_images_in_grid(
            paths,
            columns,
            side_percent,
        )?)
    }

    fn write_file(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_file_atomic(path, bytes)?)
    }

    fn write_task_file(&self, path: &Path, file: &MeshTaskFile) -> Result<()> {
        let bytes = meshy::mesh_task_file_bytes(file)?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn write_texture_task_file(&self, path: &Path, file: &TextureTaskFile) -> Result<()> {
        let bytes = meshy::texture_task_file_bytes(file)?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn write_polled_task_file(
        &self,
        path: &Path,
        head: &TaskFileHead,
        output: &MeshOutput,
    ) -> Result<()> {
        let bytes = meshy::polled_task_file_bytes(head, output)?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn sleep(&self, seconds: u64) -> Result<()> {
        thread::sleep(Duration::from_secs(seconds));
        Ok(())
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}

impl PrefsDependencies for DependenciesImpl {
    fn user_home_dir(&self) -> IOResult<Option<PathBuf>> {
        Ok(tyt_injection::user_home_dir())
    }

    fn git_root_dir(&self) -> IOResult<Option<PathBuf>> {
        tyt_injection::git_root_dir()
    }

    fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
        tyt_injection::read_file_optional(path)
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> IOResult<()> {
        tyt_injection::write_file_atomic(path, contents)
    }
}
