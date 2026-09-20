/// One operand's components beside its width, read per entry with a vec1
/// broadcasting its one component across the entry.
#[derive(Clone, Copy)]
pub(crate) struct Operand<'a, T> {
    pub(crate) components: &'a [T],
    pub(crate) width: usize,
}

impl<'a, T> Operand<'a, T> {
    /// The component at the position, a vec1 answering its one component
    /// for every position.
    pub(crate) fn component(&self, entry: usize, position: usize) -> &'a T {
        let position = if self.width == 1 { 0 } else { position };

        &self.components[entry * self.width + position]
    }

    /// The entry's components.
    pub(crate) fn entry(&self, entry: usize) -> &'a [T] {
        &self.components[entry * self.width..(entry + 1) * self.width]
    }
}
