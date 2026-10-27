/// Supplies dependencies through another value. A type implementing it
/// gets every format's dependencies trait its target implements, and so
/// [`Dependencies`](crate::Dependencies), from this one impl.
pub trait ForwardDependencies {
    /// The value holding the dependencies.
    type Target: ?Sized;

    /// The value holding the dependencies.
    fn target(&self) -> &Self::Target;
}
