use crate::InstalledFormat;

/// A computation over the format a [`ReadFormat`](crate::ReadFormat) picks.
/// [`ReadFormat::with`](crate::ReadFormat::with) runs it with that format
/// as `F`.
pub trait ReadFormatVisitor {
    /// What the visit yields.
    type Output;

    /// Runs the computation for `F`.
    fn visit<F: InstalledFormat>(self) -> Self::Output;
}
