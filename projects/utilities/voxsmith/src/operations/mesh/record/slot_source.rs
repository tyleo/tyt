/// A slot's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotSource {
    /// A file the run writes.
    File(String),

    /// An expression the run evaluates.
    Value(String),
}
