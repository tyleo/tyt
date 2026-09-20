use crate::function::Function;

/// How `any` and `all` fold a comparison's component answers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Fold {
    All,
    Any,
}

impl Fold {
    /// The fold a function names, if it is one.
    pub(crate) fn from_function(function: Function) -> Option<Fold> {
        match function {
            Function::All => Some(Fold::All),
            Function::Any => Some(Fold::Any),
            _ => None,
        }
    }
}
