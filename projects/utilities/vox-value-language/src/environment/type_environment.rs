use crate::Type;
use std::collections::HashMap;

/// The types `check` reads for the names a program never defines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TypeEnvironment {
    /// Each name's type.
    pub types: HashMap<String, Type>,
}
