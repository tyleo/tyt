use crate::{Groupings, Value};
use std::collections::HashMap;

/// The values `eval` reads for the names a program never defines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValueEnvironment {
    /// Each name's value.
    pub values: HashMap<String, Value>,

    /// The tables the reductions and climbs walk.
    pub groupings: Groupings,
}
