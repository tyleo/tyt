/// Which point of a `text` line sits on u = 0.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SdfTextAlign {
    /// The line's middle.
    Center,

    /// The line's end.
    End,

    /// The line's start.
    Start,
}
