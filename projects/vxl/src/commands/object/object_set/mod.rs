// Public API

#[allow(clippy::module_inception)]
mod object_set;
mod object_set_command;
mod object_set_edit_bounds;
mod object_set_name;
mod object_set_origin;

pub use object_set::*;
pub use object_set_command::*;
pub use object_set_edit_bounds::*;
pub use object_set_name::*;
pub use object_set_origin::*;
