use std::fmt::Debug;

/// Selects the in-memory representation of a [`VMaxFile`](crate::VMaxFile).
pub trait VMaxBackend {
    /// A `contents*.vmaxb` object.
    type Contents: Clone + Debug + PartialEq;

    /// A `palette*.settings.vmaxpsb` palette.
    type PaletteSettings: Clone + Debug + PartialEq;
}
