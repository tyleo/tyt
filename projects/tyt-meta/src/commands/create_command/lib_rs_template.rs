pub fn lib_rs_template(snake: &str) -> String {
    format!(
        r#"pub mod commands;

mod dependencies;
#[cfg(feature = "impl")]
mod dependencies_impl;
mod error;
mod result;
mod tyt_{snake};

pub use dependencies::*;
#[cfg(feature = "impl")]
pub use dependencies_impl::*;
pub use error::*;
pub use result::*;
pub use tyt_{snake}::*;
"#
    )
}
