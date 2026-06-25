use crate::MVoxDict;

/// A model reference inside a shape node: the index of the model in
/// [`MVoxFile::models`](crate::MVoxFile::models) plus its reserved attributes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxShapeModel {
    /// Index into [`MVoxFile::models`](crate::MVoxFile::models).
    pub model: u32,

    /// `_f`: the frame index this model is shown on, counting from `0`.
    pub frame_index: Option<u32>,

    /// Any further model-attribute keys, preserved verbatim.
    pub extra: MVoxDict,
}
