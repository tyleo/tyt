// Public API

mod completion;
mod completion_verb;
mod dependencies;
mod install_completion;
mod print_completion;

pub use completion::*;
pub use completion_verb::*;
pub use dependencies::*;
pub use install_completion::*;
pub use print_completion::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;
#[cfg(feature = "impl")]
pub use dependencies_impl::*;

// Internal API

mod completion_script;
pub(crate) use completion_script::*;
