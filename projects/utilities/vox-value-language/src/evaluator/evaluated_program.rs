use crate::{Groupings, Value, evaluator::Lengths};
use std::collections::HashMap;

/// A program with every binding's value computed, the input to
/// `eval_expression`.
#[derive(Clone, Debug, PartialEq)]
pub struct EvaluatedProgram {
    pub(crate) values: HashMap<String, Value>,
    pub(crate) groupings: Groupings,
    pub(crate) lengths: Lengths,
}

impl EvaluatedProgram {
    /// The value a name holds at the program's end: its last binding's, or
    /// the environment's where no binding redefines it.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }
}
