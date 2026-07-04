/// One side of a bounded value pool's range: a finite number, or unbounded.
///
/// Mirrors the wire format's number-or-none bound. This type does not enforce
/// finiteness at construction; [`VoxMain::validate`](crate::VoxMain::validate)
/// rejects a non-finite numeric bound.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoxBound {
    /// A finite numeric bound.
    Number(f64),

    /// Unbounded on this side.
    None,
}
