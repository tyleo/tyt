use crate::evaluator::EvalResult;

/// A rearrangement of entries that reads the same for every component type.
pub(crate) trait EntryTransform {
    /// Rearranges the flattened components.
    fn apply<T: Clone + PartialEq>(&self, components: &[T]) -> EvalResult<Vec<T>>;
}
