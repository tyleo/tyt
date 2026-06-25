use crate::GoxlDict;

/// The `IMG ` chunk: file-level image metadata.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GoxlImage {
    /// Optional `4 x 4` bounding box of the whole image, present only when set.
    pub bounding_box: Option<[[f32; 4]; 4]>,

    /// Dict keys this crate does not model, preserved verbatim.
    pub extra: GoxlDict,
}
