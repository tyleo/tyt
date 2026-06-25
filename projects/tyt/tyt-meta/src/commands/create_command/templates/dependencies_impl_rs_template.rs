pub const DEPENDENCIES_IMPL_RS_TEMPLATE: &str = r#"use crate::{Dependencies, Result};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
"#;
