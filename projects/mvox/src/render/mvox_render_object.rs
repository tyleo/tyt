use crate::MVoxDict;

/// A render-settings chunk (`rOBJ`): a global rendering attribute group, such as
/// the ground plane, background, bloom, or edge settings. The format gives these
/// no fixed schema, so the whole `DICT` is kept verbatim.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxRenderObject {
    /// The rendering attributes, as stored.
    pub attributes: MVoxDict,
}
