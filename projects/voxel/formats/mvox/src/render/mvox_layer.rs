use crate::MVoxDict;

/// A layer (`LAYR`): a named, optionally hidden grouping that transform nodes
/// assign themselves to by [`id`](Self::id).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxLayer {
    /// The layer id transform nodes reference.
    pub id: i32,

    /// `_name`: the layer's display name.
    pub name: Option<String>,

    /// `_hidden`: whether the layer is hidden.
    pub hidden: Option<bool>,

    /// Any further attribute keys, preserved verbatim.
    pub extra: MVoxDict,
}
