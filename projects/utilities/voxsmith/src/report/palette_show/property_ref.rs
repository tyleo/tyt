use crate::VectorComponent;

/// The property a [`PropertySelector`](crate::PropertySelector) names: one
/// property key, optionally narrowed to one vector component, or every
/// property of the palette.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum PropertyRef {
    /// Every property of the palette.
    #[default]
    All,

    /// One property by key, optionally narrowed to one vector `component`.
    Key {
        /// The property key.
        key: String,

        /// The vector component to read, or `None` for the whole value.
        component: Option<VectorComponent>,
    },
}
