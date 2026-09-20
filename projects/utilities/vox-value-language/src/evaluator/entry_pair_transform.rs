/// A merge of two operands' entries that reads the same for every component
/// type.
pub(crate) trait EntryPairTransform {
    /// Merges the two flattened component lists.
    fn apply<T: Clone>(&self, first: &[T], second: &[T]) -> Vec<T>;
}
