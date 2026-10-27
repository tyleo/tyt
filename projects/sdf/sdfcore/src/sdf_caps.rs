/// How an arc closes its ends.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SdfCaps {
    /// Square ends.
    Flat,

    /// Half-disk ends.
    Round,
}
