/// One component of a color attribute a material channel reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorChannel {
    /// The red component.
    R,

    /// The green component.
    G,

    /// The blue component.
    B,

    /// The straight-alpha component.
    A,
}
