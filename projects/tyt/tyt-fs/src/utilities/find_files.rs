use crate::{Dependencies, Error, Result};

/// Finds files matching the given glob patterns using `rg --files`.
pub fn find_files(dependencies: &impl Dependencies, patterns: &[String]) -> Result<Vec<u8>> {
    let mut args = vec!["--files".to_owned()];
    for pattern in patterns {
        args.push("-g".to_owned());
        args.push(pattern.clone());
    }

    match dependencies.exec_rg(args) {
        Ok(stdout) => Ok(stdout),
        Err(Error::Rg(ref e)) if e.exit_code == Some(1) => Ok(vec![]),
        Err(e) => Err(e),
    }
}
