use crate::{Dependencies, Result};
use std::io::{self, Write};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(io::stdout().write_all(contents)?)
    }
}
