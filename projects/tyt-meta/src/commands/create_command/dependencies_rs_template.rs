pub const DEPENDENCIES_RS_TEMPLATE: &str = r#"use crate::Result;

/// Dependencies for this crate's operations.
pub trait Dependencies {
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
"#;
