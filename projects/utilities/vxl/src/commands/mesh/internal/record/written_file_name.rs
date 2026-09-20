use crate::{Error, Result, require_file_name};

/// The bare file name `origin` writes beside the mesh. A path errors.
pub(crate) fn written_file_name(origin: &str, file: &str) -> Result<String> {
    require_file_name(file).map_err(|reason| Error::usage(format!("{origin}: {reason}")))
}
