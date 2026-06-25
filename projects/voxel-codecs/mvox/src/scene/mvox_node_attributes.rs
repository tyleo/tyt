use crate::MVoxDict;

/// Attributes shared by every scene node (`nTRN` / `nGRP` / `nSHP`): the
/// node-attributes `DICT`. The documented keys are lifted into fields; any other
/// keys are preserved in [`extra`](Self::extra).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxNodeAttributes {
    /// `_name`: the node's display name.
    pub name: Option<String>,

    /// `_hidden`: whether the node is hidden.
    pub hidden: Option<bool>,

    /// Any further attribute keys, preserved verbatim.
    pub extra: MVoxDict,
}
