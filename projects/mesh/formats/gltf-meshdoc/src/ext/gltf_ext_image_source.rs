#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Where an embedded image's bytes were loaded from, so the writer can put
/// them back. An image over one of the document's files records nothing
/// here: the file is where it was loaded from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum GltfExtImageSource {
    /// A buffer view.
    BufferView,

    /// A data URI on the image.
    DataUri,
}
