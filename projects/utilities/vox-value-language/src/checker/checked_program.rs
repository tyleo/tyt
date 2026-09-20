use crate::{Type, TypeEnvironment, checker::CheckedBinding};
use std::collections::HashMap;

/// A program with every binding's type settled, the input to `eval`.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedProgram {
    pub(crate) environment: TypeEnvironment,
    pub(crate) bindings: Vec<CheckedBinding>,
    pub(crate) scope: HashMap<String, Type>,
}

impl CheckedProgram {
    /// The type a name holds at the program's end: its last binding's, or
    /// the environment's where no binding redefines it.
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.scope.get(name)
    }
}
