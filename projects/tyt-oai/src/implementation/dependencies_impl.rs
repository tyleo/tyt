use crate::{
    Conv, Dependencies, Error, OaiRequest, OaiResponse, Result, implementation::openai,
    utilities::UsrPrefs,
};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use tyt_injection::serde_json;
use tyt_preferences::Dependencies as _;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn oai_api_key(&self) -> Result<Option<String>> {
        let prefs_deps = tyt_preferences::DependenciesImpl;
        let prefs: Option<UsrPrefs> =
            tyt_preferences::load_user_git_prefs(&prefs_deps, "oai").map_err(Error::IO)?;
        Ok(prefs.and_then(|p| p.api_key))
    }

    fn read_conv(&self, path: &Path) -> Result<Option<Conv>> {
        let prefs_deps = tyt_preferences::DependenciesImpl;
        let Some(bytes) = prefs_deps.read_file(path).map_err(Error::IO)? else {
            return Ok(None);
        };
        let conv =
            serde_json::from_slice(&bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        Ok(Some(conv))
    }

    fn write_conv(&self, path: &Path, conv: &Conv) -> Result<()> {
        let bytes =
            serde_json::to_vec_pretty(conv).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn write_image(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_file_atomic(path, bytes)?)
    }

    fn generate_image(&self, api_key: &str, request: &OaiRequest) -> Result<OaiResponse> {
        openai::send_image_request(api_key, request)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn display_image_in_terminal(&self, path: &Path) -> Result<()> {
        Ok(tyt_injection::display_image_in_terminal(path)?)
    }
}
