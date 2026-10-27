use crate::EvalResult;

/// A rearrangement of entries that reads the same for every component type.
pub trait EntryTransform {
    /// Rearranges the flattened components.
    fn apply<T: Clone + PartialEq>(&self, components: &[T]) -> EvalResult<Vec<T>>;
}
