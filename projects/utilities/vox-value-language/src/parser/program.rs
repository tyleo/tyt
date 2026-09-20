use crate::parser::SyntaxBinding;

/// A parsed program, the input to `check`.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub(crate) bindings: Vec<SyntaxBinding>,
}
