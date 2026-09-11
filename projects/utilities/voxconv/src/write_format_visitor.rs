use crate::InstalledFormat;

/// A computation over the format a [`WriteFormat`](crate::WriteFormat)
/// picks. [`WriteFormat::with`](crate::WriteFormat::with) runs it with that
/// format as `F` and the target's writer options.
pub trait WriteFormatVisitor {
    /// What the visit yields.
    type Output;

    /// Runs the computation for `F` writing with `options`.
    fn visit<F: InstalledFormat>(self, options: &F::WriteOptions) -> Self::Output;
}
