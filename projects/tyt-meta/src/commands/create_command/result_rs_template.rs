pub const RESULT_RS_TEMPLATE: &str = r#"use crate::Error;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;
"#;
